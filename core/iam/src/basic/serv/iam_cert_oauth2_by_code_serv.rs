use std::collections::HashMap;

use async_trait::async_trait;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;
use tardis::basic::dto::TardisContext;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::TardisFunsInst;

use crate::basic::dto::iam_account_dto::IamAccountAggAddReq;
use crate::basic::dto::iam_cert_conf_dto::{IamOAuth2CertConfAddOrModifyReq, IamOAuth2CertConfInfo};
use crate::basic::dto::iam_cert_dto::IamOAuth2CertAddOrModifyReq;
use crate::basic::dto::iam_filer_dto::IamTenantFilterReq;
use crate::iam_config::IamBasicConfigApi;
use crate::iam_enumeration::IamCertExtKind;
use bios_basic::rbum::dto::rbum_cert_conf_dto::RbumCertConfAddReq;
use bios_basic::rbum::dto::rbum_cert_dto::{RbumCertAddReq, RbumCertModifyReq};
use bios_basic::rbum::dto::rbum_filer_dto::{RbumCertConfFilterReq, RbumCertFilterReq};
use bios_basic::rbum::rbum_enumeration::{RbumCertRelKind, RbumCertStatusKind};
use bios_basic::rbum::serv::rbum_cert_serv::{RbumCertConfServ, RbumCertServ};
use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;

use super::iam_account_serv::IamAccountServ;
use super::iam_cert_serv::IamCertServ;
use super::iam_tenant_serv::IamTenantServ;
use super::oauth2_spi::iam_cert_oauth2_spi_wechat_mp::IamCertOAuth2SpiWeChatMp;

pub struct IamCertOAuth2ByCodeServ;

impl IamCertOAuth2ByCodeServ {
    pub async fn add_cert_conf(
        cert_kind: IamCertExtKind,
        add_req: &IamOAuth2CertConfAddOrModifyReq,
        rel_iam_item_id: String,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<String> {
        // Add rbum cert conf （for label kind only）
        let conf_id = RbumCertConfServ::add_rbum(
            &mut RbumCertConfAddReq {
                code: TrimString(cert_kind.to_string()),
                name: TrimString(cert_kind.to_string()),
                note: None,
                ak_note: None,
                ak_rule: None,
                sk_note: None,
                sk_rule: None,
                ext: None,
                sk_need: Some(false),
                sk_dynamic: Some(true),
                sk_encrypted: Some(false),
                repeatable: None,
                is_basic: Some(false),
                rest_by_kinds: None,
                expire_sec: None,
                sk_lock_cycle_sec: None,
                sk_lock_err_times: None,
                sk_lock_duration_sec: None,
                coexist_num: Some(1),
                conn_uri: None,
                rel_rbum_domain_id: funs.iam_basic_domain_iam_id(),
                rel_rbum_item_id: Some(rel_iam_item_id.clone()),
            },
            funs,
            ctx,
        )
        .await?;
        // Add real cert conf (contains ak sk）
        RbumCertServ::add_rbum(
            &mut RbumCertAddReq {
                ak: add_req.ak.clone(),
                sk: Some(add_req.sk.clone()),
                vcode: None,
                ext: None,
                start_time: None,
                end_time: None,
                conn_uri: None,
                status: RbumCertStatusKind::Enabled,
                rel_rbum_cert_conf_id: Some(conf_id.clone()),
                rel_rbum_kind: RbumCertRelKind::Item,
                rel_rbum_id: rel_iam_item_id,
                is_outside: false,
            },
            funs,
            ctx,
        )
        .await?;
        Ok(conf_id)
    }

    pub async fn modify_cert_conf(id: &str, modify_req: &IamOAuth2CertConfAddOrModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let conf_id = RbumCertConfServ::get_rbum(id, &RbumCertConfFilterReq::default(), funs, ctx).await?.id;
        let cert_id = RbumCertServ::find_id_rbums(
            &RbumCertFilterReq {
                rel_rbum_cert_conf_ids: Some(vec![conf_id]),
                ..Default::default()
            },
            None,
            None,
            funs,
            ctx,
        )
        .await?;
        let cert_id = cert_id.first().ok_or_else(|| funs.err().not_found("rbum_cert", "modify", "not found oauth2 conf", "404-rbum-cert-conf-not-exist"))?;
        RbumCertServ::modify_rbum(
            cert_id,
            &mut RbumCertModifyReq {
                ak: Some(modify_req.ak.clone()),
                sk: Some(modify_req.sk.clone()),
                ext: None,
                start_time: None,
                end_time: None,
                conn_uri: None,
                status: None,
            },
            funs,
            ctx,
        )
        .await?;
        Ok(())
    }

    pub async fn get_cert_conf(id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<IamOAuth2CertConfInfo> {
        let conf_id = RbumCertConfServ::get_rbum(id, &RbumCertConfFilterReq::default(), funs, ctx).await?.id;
        let certs = RbumCertServ::find_rbums(
            &RbumCertFilterReq {
                rel_rbum_cert_conf_ids: Some(vec![conf_id]),
                ..Default::default()
            },
            None,
            None,
            funs,
            ctx,
        )
        .await?;
        if let Some(cert) = certs.first() {
            let sk = RbumCertServ::show_sk(cert.id.as_str(), &RbumCertFilterReq::default(), funs, ctx).await?;
            Ok(IamOAuth2CertConfInfo { ak: cert.ak.to_string(), sk })
        } else {
            Err(funs.err().not_found("rbum_cert", "get", "not found oauth2 conf", "404-rbum-cert-conf-not-exist"))
        }
    }

    pub async fn add_or_modify_cert(
        add_or_modify_req: &IamOAuth2CertAddOrModifyReq,
        account_id: &str,
        rel_rbum_cert_conf_id: &str,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<()> {
        let cert_id = RbumCertServ::find_id_rbums(
            &RbumCertFilterReq {
                rel_rbum_cert_conf_ids: Some(vec![rel_rbum_cert_conf_id.to_string()]),
                rel_rbum_id: Some(account_id.to_string()),
                ..Default::default()
            },
            None,
            None,
            funs,
            ctx,
        )
        .await?;
        if let Some(cert_id) = cert_id.first() {
            RbumCertServ::modify_rbum(
                cert_id,
                &mut RbumCertModifyReq {
                    ak: Some(add_or_modify_req.ak.clone()),
                    sk: None,
                    ext: None,
                    start_time: None,
                    end_time: None,
                    conn_uri: None,
                    status: None,
                },
                funs,
                ctx,
            )
            .await?;
        } else {
            RbumCertServ::add_rbum(
                &mut RbumCertAddReq {
                    ak: add_or_modify_req.ak.clone(),
                    sk: None,
                    vcode: None,
                    ext: None,
                    start_time: None,
                    end_time: None,
                    conn_uri: None,
                    status: RbumCertStatusKind::Enabled,
                    rel_rbum_cert_conf_id: Some(rel_rbum_cert_conf_id.to_string()),
                    rel_rbum_kind: RbumCertRelKind::Item,
                    rel_rbum_id: account_id.to_string(),
                    is_outside: false,
                },
                funs,
                ctx,
            )
            .await?;
        };
        Ok(())
    }

    pub async fn get_cert_rel_account_by_ak(ak: &str, rel_rbum_cert_conf_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Option<String>> {
        let result = RbumCertServ::find_rbums(
            &RbumCertFilterReq {
                rel_rbum_cert_conf_ids: Some(vec![rel_rbum_cert_conf_id.to_string()]),
                ak: Some(ak.to_string()),
                ..Default::default()
            },
            None,
            None,
            funs,
            ctx,
        )
        .await?
        .first()
        .map(|r| r.rel_rbum_id.to_string());
        Ok(result)
    }

    pub async fn get_or_add_account(cert_kind: IamCertExtKind, code: &str, tenant_id: &str, funs: &TardisFunsInst) -> TardisResult<(String, String)> {
        let cert_conf_id = IamCertServ::get_cert_conf_id_by_code(&cert_kind.to_string(), Some(tenant_id.to_string()), funs).await?;
        let mock_ctx = TardisContext {
            own_paths: tenant_id.to_string(),
            ..Default::default()
        };
        let cert_conf = Self::get_cert_conf(&cert_conf_id, funs, &mock_ctx).await?;
        let oauth_token_info = match cert_kind {
            IamCertExtKind::WechatMp => IamCertOAuth2SpiWeChatMp::get_access_token(code, &cert_conf.ak, &cert_conf.sk, funs).await,
            _ => Err(funs.err().not_found(
                "rbum_cert",
                "get_or_add_account",
                &format!("not found oauth2 kind: {}", cert_kind),
                "404-iam-cert-oauth-kind-not-exist",
            )),
        }?;
        if let Some(account_id) = Self::get_cert_rel_account_by_ak(&oauth_token_info.open_id, &cert_conf_id, funs, &mock_ctx).await? {
            return Ok((account_id, oauth_token_info.access_token));
        }
        if !IamTenantServ::get_item(tenant_id, &IamTenantFilterReq::default(), funs, &mock_ctx).await?.account_self_reg {
            return Err(funs.err().not_found(
                "rbum_cert",
                "get_or_add_account",
                &format!("not found oauth2 cert(openid): {}", &oauth_token_info.open_id),
                "401-rbum-cert-valid-error",
            ));
        }
        // Register
        let account_id = IamAccountServ::add_account_agg(
            &IamAccountAggAddReq {
                id: None,
                name: TrimString("".to_string()),
                cert_user_name: TrimString(format!("{}_user", oauth_token_info.open_id)),
                cert_password: TrimString(oauth_token_info.access_token.to_string()),
                cert_phone: None,
                cert_mail: None,
                role_ids: None,
                org_node_ids: None,
                scope_level: None,
                disabled: None,
                icon: None,
                exts: HashMap::new(),
            },
            funs,
            &mock_ctx,
        )
        .await?;
        Self::add_or_modify_cert(
            &IamOAuth2CertAddOrModifyReq {
                ak: TrimString(oauth_token_info.open_id.to_string()),
            },
            &account_id,
            &cert_conf_id,
            funs,
            &mock_ctx,
        )
        .await?;
        Ok((account_id, oauth_token_info.access_token))
    }
}

#[async_trait]
pub trait IamCertOAuth2ByCodeSpi {
    async fn get_access_token(code: &str, ak: &str, sk: &str, funs: &TardisFunsInst) -> TardisResult<IamCertOAuth2TokenInfo>;
}

pub struct IamCertOAuth2TokenInfo {
    pub open_id: String,
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub token_expires_ms: Option<u32>,
    pub union_id: Option<String>,
}
