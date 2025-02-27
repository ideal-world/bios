use bios_basic::rbum::{
    dto::{
        rbum_cert_conf_dto::RbumCertConfAddReq,
        rbum_cert_dto::RbumCertAddReq,
        rbum_filer_dto::{RbumBasicFilterReq, RbumCertConfFilterReq},
    },
    rbum_enumeration::{RbumCertConfStatusKind, RbumCertRelKind},
    serv::{
        rbum_cert_serv::{RbumCertConfServ, RbumCertServ},
        rbum_crud_serv::RbumCrudOperation as _,
    },
};
use serde::{Deserialize, Serialize};
use tardis::{
    basic::{dto::TardisContext, field::TrimString, result::TardisResult},
    TardisFuns, TardisFunsInst,
};

use crate::{
    basic::dto::{
        iam_cert_conf_dto::IamCertConfOAuth2ServiceAddOrModifyReq,
        iam_cert_dto::{IamCertOAuth2ServiceCodeAddReq, IamCertOAuth2ServiceCodeVerifyReq},
    },
    iam_config::IamBasicConfigApi as _,
    iam_enumeration::IamCertExtKind,
};

const REDIS_CODE_KEY: &str = "iam:oauth2:code:";

pub struct IamCertOAuth2ServiceServ;

#[derive(Debug, Serialize, Deserialize)]
pub struct IamCertOAuth2ServiceCode {
    pub ctx: TardisContext,
}

impl IamCertOAuth2ServiceServ {
    pub async fn add_cert_conf(add_req: &IamCertConfOAuth2ServiceAddOrModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<String> {
        RbumCertConfServ::add_rbum(
            &mut RbumCertConfAddReq {
                id: Some(add_req.client_id.to_string()),
                kind: TrimString(IamCertExtKind::OAuth2Service.to_string()),
                supplier: None,
                name: add_req.name.clone(),
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
                rest_by_kinds: None,
                expire_sec: Some(add_req.access_token_expire_sec.unwrap_or(60 * 60 * 24 * 7)),
                sk_lock_cycle_sec: None,
                sk_lock_err_times: None,
                sk_lock_duration_sec: None,
                coexist_num: Some(1),
                conn_uri: Some(add_req.redirect_uri.to_string()),
                status: RbumCertConfStatusKind::Enabled,
                rel_rbum_domain_id: funs.iam_basic_domain_iam_id(),
                rel_rbum_item_id: None,
            },
            funs,
            ctx,
        )
        .await
    }

    /// 生成code
    pub async fn generate_code(add_req: &IamCertOAuth2ServiceCodeAddReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<String> {
        let code = TardisFuns::field.nanoid();
        let conf = RbumCertConfServ::get_rbum(
            &add_req.client_id,
            &RbumCertConfFilterReq {
                basic: RbumBasicFilterReq::default(),
                kind: Some(TrimString(IamCertExtKind::OAuth2Service.to_string())),
                supplier: None,
                status: Some(RbumCertConfStatusKind::Enabled),
                rel_rbum_domain_id: Some(funs.iam_basic_domain_iam_id()),
                rel_rbum_item_id: None,
            },
            funs,
            ctx,
        )
        .await
        .map_err(|_| funs.err().unauthorized("oauth2", "get", &format!("not unauthorized {}", add_req.client_id), "401-iam-cert-code-not-exist"))?;
        let ext = TardisFuns::json.str_to_obj::<IamCertConfOAuth2ServiceAddOrModifyReq>(&conf.ext)?;
        if add_req.redirect_uri == ext.redirect_uri {
            funs.cache()
                .set_ex(
                    &format!("{}{}", REDIS_CODE_KEY, code),
                    &TardisFuns::json.obj_to_string(&IamCertOAuth2ServiceCode { ctx: ctx.clone() })?,
                    60 * 10,
                )
                .await?;
            Ok(code)
        } else {
            Err(funs.err().bad_request("oauth2", "get", &format!("not unauthorized {}", add_req.client_id), "401-iam-cert-code-not-exist"))
        }
    }

    /// verify code to get access_token
    ///
    /// 验证client_secret code 来获取access_token
    pub async fn verify_code(add_req: &IamCertOAuth2ServiceCodeVerifyReq, funs: &TardisFunsInst) -> TardisResult<String> {
        let global_ctx = TardisContext::default();
        let conf = RbumCertConfServ::get_rbum(
            &add_req.client_id,
            &RbumCertConfFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                kind: Some(TrimString(IamCertExtKind::OAuth2Service.to_string())),
                supplier: None,
                status: Some(RbumCertConfStatusKind::Enabled),
                rel_rbum_domain_id: Some(funs.iam_basic_domain_iam_id()),
                rel_rbum_item_id: None,
            },
            funs,
            &global_ctx,
        )
        .await
        .map_err(|_| funs.err().unauthorized("oauth2", "get", &format!("not unauthorized {}", add_req.client_id), "401-iam-cert-code-not-exist"))?;
        let ext = TardisFuns::json.str_to_obj::<IamCertConfOAuth2ServiceAddOrModifyReq>(&conf.ext)?;
        if add_req.client_secret == ext.client_secret {
            // todo get code from redis
            let code = funs.cache().get(&format!("{}{}", REDIS_CODE_KEY, add_req.code)).await?;
            if code.is_some() {
                let code_value = TardisFuns::json.str_to_obj::<IamCertOAuth2ServiceCode>(&code.unwrap())?;
                let new_ak = TardisFuns::field.nanoid();
                let _ = RbumCertServ::add_rbum(
                    &mut RbumCertAddReq {
                        kind: None,
                        supplier: None,
                        ak: TrimString(new_ak),
                        sk: None,
                        sk_invisible: None,
                        ignore_check_sk: false,
                        ext: None,
                        start_time: None,
                        end_time: None,
                        conn_uri: None,
                        status: todo!(),
                        vcode: None,
                        rel_rbum_cert_conf_id: Some(conf.id),
                        rel_rbum_kind: RbumCertRelKind::Item,
                        rel_rbum_id: code_value.ctx.owner.clone(),
                        is_outside: false,
                    },
                    funs,
                    &code_value.ctx,
                );

                Ok(new_ak)
            } else {
                Err(funs.err().bad_request("oauth2", "get", &format!("not unauthorized or code expired"), "401-iam-cert-code-not-exist"))
            }
        } else {
            Err(funs.err().bad_request("oauth2", "get", &format!("not unauthorized {}", add_req.client_id), "401-iam-cert-code-not-exist"))
        }
    }
}
