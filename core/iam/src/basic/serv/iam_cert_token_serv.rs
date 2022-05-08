use tardis::basic::dto::{TardisContext, TardisFunsInst};
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;

use bios_basic::rbum::dto::rbum_cert_conf_dto::{RbumCertConfAddReq, RbumCertConfModifyReq};
use bios_basic::rbum::dto::rbum_cert_dto::RbumCertAddReq;
use bios_basic::rbum::dto::rbum_filer_dto::RbumCertFilterReq;
use bios_basic::rbum::rbum_enumeration::{RbumCertRelKind, RbumCertStatusKind};
use bios_basic::rbum::serv::rbum_cert_serv::{RbumCertConfServ, RbumCertServ};
use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;

use crate::basic::dto::iam_cert_conf_dto::{IamTokenCertConfAddReq, IamTokenCertConfModifyReq};
use crate::iam_config::{IamBasicInfoManager, IamConfig};
use crate::iam_enumeration::IamCertTokenKind;

pub struct IamCertTokenServ;

impl<'a> IamCertTokenServ {
    pub async fn add_cert_conf(
        add_req: &IamTokenCertConfAddReq,
        token_kind: IamCertTokenKind,
        rel_iam_item_id: Option<String>,
        funs: &TardisFunsInst<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<()> {
        RbumCertConfServ::add_rbum(
            &mut RbumCertConfAddReq {
                code: TrimString(token_kind.to_string()),
                name: TrimString(add_req.name.to_string()),
                note: None,
                ak_note: None,
                ak_rule: None,
                sk_note: None,
                sk_rule: None,
                sk_need: Some(false),
                sk_dynamic: None,
                sk_encrypted: Some(false),
                repeatable: None,
                is_basic: Some(false),
                rest_by_kinds: None,
                expire_sec: add_req.expire_sec,
                coexist_num: Some(add_req.coexist_num),
                conn_uri: None,
                rel_rbum_domain_id: IamBasicInfoManager::get().domain_iam_id.to_string(),
                rel_rbum_item_id: rel_iam_item_id,
            },
            funs,
            cxt,
        )
        .await?;
        Ok(())
    }

    pub async fn modify_cert_conf(id: &str, modify_req: &IamTokenCertConfModifyReq, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<()> {
        RbumCertConfServ::modify_rbum(
            id,
            &mut RbumCertConfModifyReq {
                name: modify_req.name.clone(),
                note: None,
                ak_note: None,
                ak_rule: None,
                sk_note: None,
                sk_rule: None,
                sk_need: None,
                sk_encrypted: None,
                repeatable: None,
                is_basic: None,
                rest_by_kinds: None,
                expire_sec: modify_req.expire_sec,
                coexist_num: modify_req.coexist_num,
                conn_uri: None,
            },
            funs,
            cxt,
        )
        .await?;
        Ok(())
    }

    pub async fn add_cert(
        token: &str,
        token_kind: &IamCertTokenKind,
        rel_iam_item_id: &str,
        rel_rbum_cert_conf_id: &str,
        from_cert_id: &str,
        funs: &TardisFunsInst<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<()> {
        let id = RbumCertServ::add_rbum(
            &mut RbumCertAddReq {
                ak: TrimString(token.to_string()),
                sk: None,
                vcode: None,
                ext: Some(from_cert_id.to_string()),
                start_time: None,
                end_time: None,
                conn_uri: None,
                status: RbumCertStatusKind::Enabled,
                rel_rbum_cert_conf_id: Some(rel_rbum_cert_conf_id.to_string()),
                rel_rbum_kind: RbumCertRelKind::Item,
                rel_rbum_id: rel_iam_item_id.to_string(),
            },
            funs,
            cxt,
        )
        .await?;
        let cert = RbumCertServ::get_rbum(&id, &RbumCertFilterReq::default(), funs, cxt).await?;
        let expire_sec = (cert.end_time - cert.start_time).num_seconds() as usize;
        funs.cache()
            .set_ex(
                format!("{}{}", funs.conf::<IamConfig>().cache_key_token_info_, token).as_str(),
                format!("{},{}", token_kind.to_string(), rel_iam_item_id).as_str(),
                expire_sec,
            )
            .await?;
        funs.cache().hset(format!("{}{}", funs.conf::<IamConfig>().cache_key_account_rel_, rel_iam_item_id).as_str(), &token, "").await?;
        Ok(())
    }

    pub async fn remove_cert(token: &str, funs: &TardisFunsInst<'a>) -> TardisResult<()> {
        if let Some(token_info) = funs.cache().get(format!("{}{}", funs.conf::<IamConfig>().cache_key_token_info_, token).as_str()).await? {
            let iam_item_id = token_info.split(",").nth(1).unwrap_or("");
            funs.cache().del(format!("{}{}", funs.conf::<IamConfig>().cache_key_token_info_, token).as_str()).await?;
            funs.cache().hdel(format!("{}{}", funs.conf::<IamConfig>().cache_key_account_rel_, iam_item_id).as_str(), &token).await?;
        }
        Ok(())
    }
}
