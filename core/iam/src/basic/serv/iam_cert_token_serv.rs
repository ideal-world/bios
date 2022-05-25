use std::str::FromStr;

use itertools::Itertools;
use tardis::basic::dto::{TardisContext, TardisFunsInst};
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::chrono::Utc;

use bios_basic::rbum::dto::rbum_cert_conf_dto::{RbumCertConfAddReq, RbumCertConfModifyReq};
use bios_basic::rbum::dto::rbum_filer_dto::{RbumBasicFilterReq, RbumCertConfFilterReq};
use bios_basic::rbum::serv::rbum_cert_serv::RbumCertConfServ;
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
        funs: &TardisFunsInst<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<()> {
        let cert_conf = RbumCertConfServ::peek_rbum(
            rel_rbum_cert_conf_id,
            &RbumCertConfFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                ..Default::default()
            },
            funs,
            cxt,
        )
        .await?;
        if cert_conf.expire_sec > 0 {
            funs.cache()
                .set_ex(
                    format!("{}{}", funs.conf::<IamConfig>().cache_key_token_info_, token).as_str(),
                    format!("{},{}", token_kind.to_string(), rel_iam_item_id).as_str(),
                    cert_conf.expire_sec as usize,
                )
                .await?;
        } else {
            funs.cache()
                .set(
                    format!("{}{}", funs.conf::<IamConfig>().cache_key_token_info_, token).as_str(),
                    format!("{},{}", token_kind.to_string(), rel_iam_item_id).as_str(),
                )
                .await?;
        }
        funs.cache()
            .hset(
                format!("{}{}", funs.conf::<IamConfig>().cache_key_account_rel_, rel_iam_item_id).as_str(),
                token,
                &format!("{},{}", token_kind.to_string(), Utc::now().timestamp()),
            )
            .await?;
        // Remove old tokens
        // TODO test
        if cert_conf.coexist_num != 0 {
            let old_tokens = funs.cache().hgetall(format!("{}{}", funs.conf::<IamConfig>().cache_key_account_rel_, rel_iam_item_id).as_str()).await?;
            let old_tokens = old_tokens
                .into_iter()
                .map(|(k, v)| {
                    (
                        k,
                        v.split(",").next().unwrap_or("").to_string(),
                        i64::from_str(v.split(',').nth(1).unwrap_or("")).unwrap_or(0),
                    )
                })
                .filter(|(_, kind, _)| kind == token_kind.to_string().as_str())
                .sorted_by(|(_, _, t1), (_, _, t2)| t2.cmp(t1))
                .skip(cert_conf.coexist_num as usize)
                .map(|(token, _, _)| token)
                .collect::<Vec<String>>();
            for old_token in old_tokens {
                Self::remove_token(&old_token, funs).await?;
            }
        }
        Ok(())
    }

    pub async fn remove_token(token: &str, funs: &TardisFunsInst<'a>) -> TardisResult<()> {
        if let Some(token_info) = funs.cache().get(format!("{}{}", funs.conf::<IamConfig>().cache_key_token_info_, token).as_str()).await? {
            let iam_item_id = token_info.split(",").nth(1).unwrap_or("");
            funs.cache().del(format!("{}{}", funs.conf::<IamConfig>().cache_key_token_info_, token).as_str()).await?;
            funs.cache().hdel(format!("{}{}", funs.conf::<IamConfig>().cache_key_account_rel_, iam_item_id).as_str(), &token).await?;
        }
        Ok(())
    }
}
