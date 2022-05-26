use std::str::FromStr;

use itertools::Itertools;
use tardis::basic::dto::{TardisContext, TardisFunsInst};
use tardis::basic::error::TardisError;
use tardis::basic::result::TardisResult;
use tardis::chrono::Utc;
use tardis::TardisFuns;

use crate::basic::dto::iam_account_dto::AccountInfoResp;
use crate::basic::dto::iam_cert_dto::IamContextFetchReq;
use crate::iam_config::IamConfig;
use crate::iam_enumeration::IamCertTokenKind;

pub struct IamIdentCacheServ;

impl<'a> IamIdentCacheServ {
    pub async fn add_token(token: &str, token_kind: &IamCertTokenKind, rel_iam_item_id: &str, expire_sec: u32, coexist_num: u32, funs: &TardisFunsInst<'a>) -> TardisResult<()> {
        if expire_sec > 0 {
            funs.cache()
                .set_ex(
                    format!("{}{}", funs.conf::<IamConfig>().cache_key_token_info_, token).as_str(),
                    format!("{},{}", token_kind.to_string(), rel_iam_item_id).as_str(),
                    expire_sec as usize,
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
        if coexist_num != 0 {
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
                .skip(coexist_num as usize)
                .map(|(token, _, _)| token)
                .collect::<Vec<String>>();
            for old_token in old_tokens {
                Self::delete_token_by_token(&old_token, funs).await?;
            }
        }
        Ok(())
    }

    pub async fn delete_token_by_token(token: &str, funs: &TardisFunsInst<'a>) -> TardisResult<()> {
        if let Some(token_info) = funs.cache().get(format!("{}{}", funs.conf::<IamConfig>().cache_key_token_info_, token).as_str()).await? {
            let iam_item_id = token_info.split(",").nth(1).unwrap_or("");
            funs.cache().del(format!("{}{}", funs.conf::<IamConfig>().cache_key_token_info_, token).as_str()).await?;
            funs.cache().hdel(format!("{}{}", funs.conf::<IamConfig>().cache_key_account_rel_, iam_item_id).as_str(), &token).await?;
        }
        Ok(())
    }

    pub async fn delete_token_by_account_id(account_id: &str, funs: &TardisFunsInst<'a>) -> TardisResult<()> {
        let tokens = funs.cache().hgetall(format!("{}{}", funs.conf::<IamConfig>().cache_key_account_rel_, account_id).as_str()).await?;
        for (token, _) in tokens.iter() {
            funs.cache().del(format!("{}{}", funs.conf::<IamConfig>().cache_key_token_info_, token).as_str()).await?;
        }
        funs.cache().del(format!("{}{}", funs.conf::<IamConfig>().cache_key_account_rel_, account_id).as_str()).await?;
        funs.cache().del(format!("{}{}", funs.conf::<IamConfig>().cache_key_account_info_, account_id).as_str()).await?;
        Ok(())
    }

    pub async fn add_contexts(account_info: &AccountInfoResp, ak: &str, tenant_id: &str, funs: &TardisFunsInst<'a>) -> TardisResult<()> {
        funs.cache()
            .hset(
                format!("{}{}", funs.conf::<IamConfig>().cache_key_account_info_, account_info.account_id).as_str(),
                "",
                &TardisFuns::json.obj_to_string(&TardisContext {
                    own_paths: tenant_id.to_string(),
                    ak: ak.to_string(),
                    owner: account_info.account_id.to_string(),
                    roles: account_info.roles.iter().map(|(id, _)| id.to_string()).collect(),
                    groups: account_info.groups.iter().map(|(id, _)| id.to_string()).collect(),
                })?,
            )
            .await?;
        for account_app_info in &account_info.apps {
            funs.cache()
                .hset(
                    format!("{}{}", funs.conf::<IamConfig>().cache_key_account_info_, account_info.account_id).as_str(),
                    &account_app_info.app_id,
                    &TardisFuns::json.obj_to_string(&TardisContext {
                        own_paths: format!("{}/{}", tenant_id, account_app_info.app_id).to_string(),
                        ak: ak.to_string(),
                        owner: account_info.account_id.to_string(),
                        roles: account_app_info.roles.iter().map(|(id, _)| id.to_string()).collect(),
                        groups: account_app_info.groups.iter().map(|(id, _)| id.to_string()).collect(),
                    })?,
                )
                .await?;
        }
        Ok(())
    }

    pub async fn fetch_context(fetch_req: &IamContextFetchReq, funs: &TardisFunsInst<'a>) -> TardisResult<TardisContext> {
        if let Some(token_info) = funs.cache().get(format!("{}{}", funs.conf::<IamConfig>().cache_key_token_info_, &fetch_req.token).as_str()).await? {
            let account_id = token_info.split(',').nth(1).unwrap_or("");
            if let Some(context) = funs
                .cache()
                .hget(
                    format!("{}{}", funs.conf::<IamConfig>().cache_key_account_info_, account_id).as_str(),
                    fetch_req.app_id.as_ref().unwrap_or(&"".to_string()),
                )
                .await?
            {
                return TardisFuns::json.str_to_obj(&context);
            }
        }
        Err(TardisError::NotFound("context not found".to_string()))
    }
}
