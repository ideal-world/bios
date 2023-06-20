use std::collections::HashMap;

use async_trait::async_trait;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;
use tardis::basic::dto::TardisContext;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::{TardisFuns, TardisFunsInst};

use crate::basic::dto::iam_account_dto::IamAccountAggAddReq;
use crate::basic::dto::iam_cert_conf_dto::{IamCertConfOAuth2AddOrModifyReq, IamCertConfOAuth2Resp};
use crate::basic::dto::iam_cert_dto::IamCertOAuth2AddOrModifyReq;
use crate::basic::dto::iam_filer_dto::IamTenantFilterReq;
use crate::basic::serv::iam_cert_user_pwd_serv::IamCertUserPwdServ;
use crate::basic::serv::oauth2_spi::iam_cert_oauth2_spi_github::IamCertOAuth2SpiGithub;
use crate::iam_config::IamBasicConfigApi;
use crate::iam_enumeration::{IamCertExtKind, IamCertOAuth2Supplier};
use bios_basic::rbum::dto::rbum_cert_conf_dto::{RbumCertConfAddReq, RbumCertConfModifyReq};
use bios_basic::rbum::dto::rbum_cert_dto::{RbumCertAddReq, RbumCertModifyReq};
use bios_basic::rbum::dto::rbum_filer_dto::{RbumCertConfFilterReq, RbumCertFilterReq};
use bios_basic::rbum::rbum_enumeration::RbumCertStatusKind::Pending;
use bios_basic::rbum::rbum_enumeration::{RbumCertConfStatusKind, RbumCertRelKind, RbumCertStatusKind};
use bios_basic::rbum::serv::rbum_cert_serv::{RbumCertConfServ, RbumCertServ};
use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;

use super::iam_account_serv::IamAccountServ;
use super::iam_cert_serv::IamCertServ;
use super::iam_tenant_serv::IamTenantServ;
use super::oauth2_spi::iam_cert_oauth2_spi_wechat_mp::IamCertOAuth2SpiWeChatMp;

pub struct IamCertOAuth2Serv;

impl IamCertOAuth2Serv {
    pub async fn add_cert_conf(
        cert_supplier: IamCertOAuth2Supplier,
        add_req: &IamCertConfOAuth2AddOrModifyReq,
        rel_iam_item_id: &str,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<String> {
        RbumCertConfServ::add_rbum(
            &mut RbumCertConfAddReq {
                kind: TrimString(IamCertExtKind::OAuth2.to_string()),
                supplier: Some(TrimString(cert_supplier.to_string())),
                name: TrimString(format!("{}{}", IamCertExtKind::OAuth2, cert_supplier)),
                note: None,
                ak_note: None,
                ak_rule: None,
                sk_note: None,
                sk_rule: None,
                ext: Some(TardisFuns::json.obj_to_string(&add_req)?),
                sk_need: Some(false),
                sk_dynamic: Some(false),
                sk_encrypted: Some(false),
                repeatable: None,
                is_basic: Some(false),
                is_ak_repeatable: None,
                rest_by_kinds: None,
                expire_sec: None,
                sk_lock_cycle_sec: None,
                sk_lock_err_times: None,
                sk_lock_duration_sec: None,
                coexist_num: Some(1),
                conn_uri: None,
                status: RbumCertConfStatusKind::Enabled,
                rel_rbum_domain_id: funs.iam_basic_domain_iam_id(),
                rel_rbum_item_id: Some(rel_iam_item_id.to_string()),
            },
            funs,
            ctx,
        )
        .await
    }

    pub async fn modify_cert_conf(id: &str, modify_req: &IamCertConfOAuth2AddOrModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        RbumCertConfServ::modify_rbum(
            id,
            &mut RbumCertConfModifyReq {
                name: None,
                note: None,
                ak_note: None,
                ak_rule: None,
                sk_note: None,
                sk_rule: None,
                ext: Some(TardisFuns::json.obj_to_string(&modify_req)?),
                sk_need: None,
                sk_encrypted: None,
                repeatable: None,
                is_basic: None,
                rest_by_kinds: None,
                expire_sec: None,
                sk_lock_cycle_sec: None,
                sk_lock_err_times: None,
                sk_lock_duration_sec: None,
                coexist_num: None,
                conn_uri: None,
                status: None,
            },
            funs,
            ctx,
        )
        .await
    }

    pub async fn get_cert_conf(id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<IamCertConfOAuth2Resp> {
        RbumCertConfServ::get_rbum(id, &RbumCertConfFilterReq::default(), funs, ctx)
            .await
            .map(|i: bios_basic::rbum::dto::rbum_cert_conf_dto::RbumCertConfDetailResp| TardisFuns::json.str_to_obj(&i.ext))?
    }

    pub async fn add_or_modify_cert(
        add_or_modify_req: &IamCertOAuth2AddOrModifyReq,
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
                    ak: Some(add_or_modify_req.open_id.clone()),
                    sk: None,
                    is_ignore_check_sk: false,
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
                    ak: add_or_modify_req.open_id.clone(),
                    sk: None,
                    kind: None,
                    supplier: None,
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
                    is_ignore_check_sk: false,
                },
                funs,
                ctx,
            )
            .await?;
        };
        Ok(())
    }

    pub async fn get_cert_rel_account_by_open_id(open_id: &str, rel_rbum_cert_conf_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Option<String>> {
        let result = RbumCertServ::find_rbums(
            &RbumCertFilterReq {
                rel_rbum_cert_conf_ids: Some(vec![rel_rbum_cert_conf_id.to_string()]),
                ak: Some(open_id.to_string()),
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

    pub async fn get_or_add_account(cert_supplier: IamCertOAuth2Supplier, code: &str, tenant_id: &str, funs: &TardisFunsInst) -> TardisResult<(String, String)> {
        let cert_conf_id =
            IamCertServ::get_cert_conf_id_by_kind_supplier(&IamCertExtKind::OAuth2.to_string(), &cert_supplier.to_string(), Some(tenant_id.to_string()), funs).await?;
        let mut mock_ctx = TardisContext {
            own_paths: tenant_id.to_string(),
            ..Default::default()
        };
        let cert_conf = Self::get_cert_conf(&cert_conf_id, funs, &mock_ctx).await?;
        let client = Self::get_access_token_func(cert_supplier);
        let oauth_token_info = client.get_access_token(code, &cert_conf.ak, &cert_conf.sk, funs).await?;
        if let Some(account_id) = Self::get_cert_rel_account_by_open_id(&oauth_token_info.open_id, &cert_conf_id, funs, &mock_ctx).await? {
            return Ok((account_id, oauth_token_info.access_token));
        }
        if !IamTenantServ::get_item(tenant_id, &IamTenantFilterReq::default(), funs, &mock_ctx).await?.account_self_reg {
            return Err(funs.err().not_found(
                "rbum_cert",
                "get_or_add_account",
                &format!("not found oauth2 cert(openid): {} and self-registration disabled", &oauth_token_info.open_id),
                "401-rbum-cert-valid-error",
            ));
        }
        // Register
        mock_ctx.owner = TardisFuns::field.nanoid();
        let account_id = IamAccountServ::add_account_agg(
            &IamAccountAggAddReq {
                id: Some(TrimString(mock_ctx.owner.clone())),
                name: TrimString(client.get_account_name(oauth_token_info.clone(), funs).await?),
                cert_user_name: IamCertUserPwdServ::rename_ak_if_duplicate(&TardisFuns::field.nanoid_len(8).to_lowercase(), funs, &mock_ctx).await?,
                // FIXME 临时密码
                cert_password: Some(TrimString(format!("{}0Pw$", TardisFuns::field.nanoid_len(6)))),
                cert_phone: None,
                cert_mail: None,
                role_ids: None,
                org_node_ids: None,
                scope_level: None,
                disabled: None,
                icon: None,
                exts: HashMap::new(),
                status: Some(Pending),
                temporary: None,
                lock_status: None,
            },
            false,
            funs,
            &mock_ctx,
        )
        .await?;
        Self::add_or_modify_cert(
            &IamCertOAuth2AddOrModifyReq {
                open_id: TrimString(oauth_token_info.open_id.to_string()),
            },
            &account_id,
            &cert_conf_id,
            funs,
            &mock_ctx,
        )
        .await?;
        IamAccountServ::async_add_or_modify_account_search(account_id.clone(), Box::new(false), "".to_string(), funs, &mock_ctx).await?;
        mock_ctx.execute_task().await?;
        Ok((account_id, oauth_token_info.access_token))
    }

    fn get_access_token_func(supplier: IamCertOAuth2Supplier) -> Box<dyn IamCertOAuth2Spi> {
        match supplier {
            // IamCertOAuth2Supplier::Weibo => {}
            IamCertOAuth2Supplier::Github => Box::new(IamCertOAuth2SpiGithub),
            IamCertOAuth2Supplier::WechatMp => Box::new(IamCertOAuth2SpiWeChatMp),
        }
    }

    pub async fn add_or_enable_cert_conf(
        supplier: IamCertOAuth2Supplier,
        add_req: &IamCertConfOAuth2AddOrModifyReq,
        rel_iam_item_id: &str,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<String> {
        let cert_result = RbumCertConfServ::do_find_one_rbum(
            &RbumCertConfFilterReq {
                kind: Some(TrimString(IamCertExtKind::OAuth2.to_string())),
                supplier: Some(supplier.clone().to_string()),
                rel_rbum_item_id: Some(rel_iam_item_id.into()),
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?;
        let result = if let Some(cert_result) = cert_result {
            IamCertServ::enabled_cert_conf(&cert_result.id, funs, ctx).await?;
            cert_result.id
        } else {
            Self::add_cert_conf(supplier, add_req, rel_iam_item_id, funs, ctx).await?
        };
        Ok(result)
    }
}

#[async_trait]
pub trait IamCertOAuth2Spi: Send + Sync {
    async fn get_access_token(&self, code: &str, ak: &str, sk: &str, funs: &TardisFunsInst) -> TardisResult<IamCertOAuth2TokenInfo>;
    async fn get_account_name(&self, oauth2_info: IamCertOAuth2TokenInfo, funs: &TardisFunsInst) -> TardisResult<String>;
}

#[derive(Clone)]
pub struct IamCertOAuth2TokenInfo {
    pub open_id: String,
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub token_expires_ms: Option<u32>,
    pub union_id: Option<String>,
}
