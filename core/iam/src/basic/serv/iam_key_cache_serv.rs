use std::default::Default;
use std::str::FromStr;

use bios_basic::process::task_processor::TaskProcessor;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::chrono::Utc;
use tardis::{log, TardisFuns, TardisFunsInst};

use bios_basic::rbum::dto::rbum_filer_dto::RbumBasicFilterReq;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;

use crate::basic::dto::iam_account_dto::IamAccountInfoResp;
use crate::basic::dto::iam_cert_dto::IamContextFetchReq;
use crate::basic::dto::iam_filer_dto::{IamAccountFilterReq, IamAppFilterReq};
use crate::basic::serv::iam_account_serv::IamAccountServ;
use crate::basic::serv::iam_app_serv::IamAppServ;
use crate::basic::serv::iam_rel_serv::IamRelServ;
use crate::iam_config::IamConfig;
use crate::iam_constants;
use crate::iam_enumeration::{IamCertTokenKind, IamRelKind};

use super::iam_cert_serv::IamCertServ;

pub struct IamIdentCacheServ;

impl IamIdentCacheServ {
    pub async fn add_token(token: &str, token_kind: &IamCertTokenKind, rel_iam_item_id: &str, expire_sec: u32, coexist_num: u32, funs: &TardisFunsInst) -> TardisResult<()> {
        log::trace!("add token: token={}", token);
        if expire_sec > 0 {
            funs.cache()
                .set_ex(
                    format!("{}{}", funs.conf::<IamConfig>().cache_key_token_info_, token).as_str(),
                    format!("{},{}", token_kind, rel_iam_item_id).as_str(),
                    expire_sec as usize,
                )
                .await?;
        } else {
            funs.cache()
                .set(
                    format!("{}{}", funs.conf::<IamConfig>().cache_key_token_info_, token).as_str(),
                    format!("{},{}", token_kind, rel_iam_item_id).as_str(),
                )
                .await?;
        }
        funs.cache()
            .hset(
                format!("{}{}", funs.conf::<IamConfig>().cache_key_account_rel_, rel_iam_item_id).as_str(),
                token,
                &format!("{},{}", token_kind, Utc::now().timestamp_nanos()),
            )
            .await?;
        // Remove old tokens
        if coexist_num != 0 {
            let old_tokens = funs.cache().hgetall(format!("{}{}", funs.conf::<IamConfig>().cache_key_account_rel_, rel_iam_item_id).as_str()).await?;
            let old_tokens = old_tokens
                .into_iter()
                .map(|(k, v)| {
                    (
                        k,
                        v.split(',').next().unwrap_or("").to_string(),
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

    pub async fn delete_token_by_token(token: &str, funs: &TardisFunsInst) -> TardisResult<()> {
        log::trace!("delete token: token={}", token);
        if let Some(token_info) = funs.cache().get(format!("{}{}", funs.conf::<IamConfig>().cache_key_token_info_, token).as_str()).await? {
            let iam_item_id = token_info.split(',').nth(1).unwrap_or("");
            funs.cache().del(format!("{}{}", funs.conf::<IamConfig>().cache_key_token_info_, token).as_str()).await?;
            funs.cache().hdel(format!("{}{}", funs.conf::<IamConfig>().cache_key_account_rel_, iam_item_id).as_str(), token).await?;
        }
        Ok(())
    }

    pub async fn delete_tokens_and_contexts_by_tenant_or_app(tenant_or_app_id: &str, is_app: bool, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let tenant_or_app_id = tenant_or_app_id.to_string();
        let own_paths = if is_app {
            IamAppServ::peek_item(
                &tenant_or_app_id,
                &IamAppFilterReq {
                    basic: RbumBasicFilterReq {
                        with_sub_own_paths: true,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                funs,
                ctx,
            )
            .await?
            .own_paths
        } else {
            tenant_or_app_id.clone()
        };
        let ctx_clone = ctx.clone();
        TaskProcessor::execute_task_with_ctx(
            &funs.conf::<IamConfig>().cache_key_async_task_status,
            move || async move {
                let funs = iam_constants::get_tardis_inst();
                let filter = IamAccountFilterReq {
                    basic: RbumBasicFilterReq {
                        own_paths: Some(own_paths),
                        with_sub_own_paths: true,
                        ..Default::default()
                    },
                    ..Default::default()
                };
                let mut count = IamAccountServ::count_items(&filter, &funs, &ctx_clone).await.unwrap() as isize;
                let mut page_number = 1;
                while count > 0 {
                    let ids = IamAccountServ::paginate_id_items(&filter, page_number, 100, None, None, &funs, &ctx_clone).await.unwrap().records;
                    for id in ids {
                        // Self::delete_tokens_and_contexts_by_account_id(&id, &funs).await.unwrap();
                        // TODO reset account cache
                        let tenant_ctx = IamCertServ::use_sys_or_tenant_ctx_unsafe(ctx_clone.clone())?;
                        IamCertServ::package_tardis_account_context_and_resp(&id, &tenant_ctx.own_paths, "".to_string(), None, &funs, &tenant_ctx).await?;
                    }
                    page_number += 1;
                    count -= 100;
                }
                if is_app {
                    let mut count = IamRelServ::count_to_rels(&IamRelKind::IamAccountApp, &tenant_or_app_id, &funs, &ctx_clone).await.unwrap() as isize;
                    let mut page_number = 1;
                    while count > 0 {
                        let ids =
                            IamRelServ::paginate_to_id_rels(&IamRelKind::IamAccountApp, &tenant_or_app_id, page_number, 100, None, None, &funs, &ctx_clone).await.unwrap().records;
                        for id in ids {
                            // IamIdentCacheServ::delete_tokens_and_contexts_by_account_id(&id, &funs).await.unwrap();
                            // TODO reset account cache
                            let tenant_ctx = IamCertServ::use_sys_or_tenant_ctx_unsafe(ctx_clone.clone())?;
                            IamCertServ::package_tardis_account_context_and_resp(&id, &tenant_ctx.own_paths, "".to_string(), None, &funs, &tenant_ctx).await?;
                        }
                        page_number += 1;
                        count -= 100;
                    }
                }
                Ok(())
            },
            funs,
            ctx,
        )
        .await?;
        Ok(())
    }

    pub async fn delete_tokens_and_contexts_by_account_id(account_id: &str, funs: &TardisFunsInst) -> TardisResult<()> {
        log::trace!("delete tokens and contexts: account_id={}", account_id);
        let tokens = funs.cache().hgetall(format!("{}{}", funs.conf::<IamConfig>().cache_key_account_rel_, account_id).as_str()).await?;
        for (token, _) in tokens.iter() {
            funs.cache().del(format!("{}{}", funs.conf::<IamConfig>().cache_key_token_info_, token).as_str()).await?;
        }
        funs.cache().del(format!("{}{}", funs.conf::<IamConfig>().cache_key_account_rel_, account_id).as_str()).await?;
        funs.cache().del(format!("{}{}", funs.conf::<IamConfig>().cache_key_account_info_, account_id).as_str()).await?;
        Ok(())
    }

    pub async fn add_contexts(account_info: &IamAccountInfoResp, tenant_id: &str, funs: &TardisFunsInst) -> TardisResult<()> {
        log::trace!("add contexts: account_id={:?}", account_info);
        funs.cache()
            .hset(
                format!("{}{}", funs.conf::<IamConfig>().cache_key_account_info_, account_info.account_id).as_str(),
                "",
                &TardisFuns::json.obj_to_string(&TardisContext {
                    own_paths: tenant_id.to_string(),
                    owner: account_info.account_id.to_string(),
                    roles: account_info.roles.iter().map(|(id, _)| id.to_string()).collect(),
                    groups: account_info.groups.iter().map(|(id, _)| id.to_string()).collect(),
                    ..Default::default()
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
                        owner: account_info.account_id.to_string(),
                        roles: account_app_info.roles.iter().map(|(id, _)| id.to_string()).collect(),
                        groups: account_app_info.groups.iter().map(|(id, _)| id.to_string()).collect(),
                        ..Default::default()
                    })?,
                )
                .await?;
        }
        Ok(())
    }

    pub async fn get_context(fetch_req: &IamContextFetchReq, funs: &TardisFunsInst) -> TardisResult<TardisContext> {
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
        Err(funs.err().not_found("iam_cache_context", "get", "not found context", "404-iam-cache-context-not-exist"))
    }
}

pub struct IamResCacheServ;

impl IamResCacheServ {
    pub async fn add_res(item_code: &str, action: &str, funs: &TardisFunsInst) -> TardisResult<()> {
        let uri_mixed = Self::package_uri_mixed(item_code, action);
        log::trace!("add res: uri_mixed={}", uri_mixed);
        funs.cache()
            .hset(
                &funs.conf::<IamConfig>().cache_key_res_info,
                &uri_mixed,
                &TardisFuns::json.obj_to_string(&IamCacheResRelAddOrModifyDto::default())?,
            )
            .await?;
        Self::add_change_trigger(&uri_mixed, funs).await
    }

    pub async fn delete_res(item_code: &str, action: &str, funs: &TardisFunsInst) -> TardisResult<()> {
        let uri_mixed = Self::package_uri_mixed(item_code, action);
        log::trace!("delete res: uri_mixed={}", uri_mixed);
        funs.cache().hdel(&funs.conf::<IamConfig>().cache_key_res_info, &uri_mixed).await?;
        Self::add_change_trigger(&uri_mixed, funs).await
    }

    pub async fn add_or_modify_res_rel(item_code: &str, action: &str, add_or_modify_req: &IamCacheResRelAddOrModifyReq, funs: &TardisFunsInst) -> TardisResult<()> {
        if add_or_modify_req.st.is_some() || add_or_modify_req.et.is_some() {
            // TODO support time range
            return Err(funs.err().conflict("iam_cache_res", "add_or_modify", "st and et must be none", "409-iam-cache-res-date-not-none"));
        }
        let mut res_dto = IamCacheResRelAddOrModifyDto {
            accounts: format!("#{}#", add_or_modify_req.accounts.join("#")),
            roles: format!("#{}#", add_or_modify_req.roles.join("#")),
            groups: format!("#{}#", add_or_modify_req.groups.join("#")),
            apps: format!("#{}#", add_or_modify_req.apps.join("#")),
            tenants: format!("#{}#", add_or_modify_req.tenants.join("#")),
        };
        let uri_mixed = Self::package_uri_mixed(item_code, action);
        log::trace!("add or modify res rel: uri_mixed={}", uri_mixed);
        let rels = funs.cache().hget(&funs.conf::<IamConfig>().cache_key_res_info, &uri_mixed).await?;
        if let Some(rels) = rels {
            let old_res_dto = TardisFuns::json.str_to_obj::<IamCacheResRelAddOrModifyDto>(&rels)?;
            res_dto.accounts = format!("{}{}", res_dto.accounts, old_res_dto.accounts);
            res_dto.roles = format!("{}{}", res_dto.roles, old_res_dto.roles);
            res_dto.groups = format!("{}{}", res_dto.groups, old_res_dto.groups);
            res_dto.apps = format!("{}{}", res_dto.apps, old_res_dto.apps);
            res_dto.tenants = format!("{}{}", res_dto.tenants, old_res_dto.tenants);
        }
        res_dto.accounts = res_dto.accounts.replace("##", "#");
        res_dto.roles = res_dto.roles.replace("##", "#");
        res_dto.groups = res_dto.groups.replace("##", "#");
        res_dto.apps = res_dto.apps.replace("##", "#");
        res_dto.tenants = res_dto.tenants.replace("##", "#");
        funs.cache().hset(&funs.conf::<IamConfig>().cache_key_res_info, &uri_mixed, &TardisFuns::json.obj_to_string(&res_dto)?).await?;
        Self::add_change_trigger(&uri_mixed, funs).await
    }

    pub async fn delete_res_rel(item_code: &str, action: &str, delete_req: &IamCacheResRelDeleteReq, funs: &TardisFunsInst) -> TardisResult<()> {
        let uri_mixed = Self::package_uri_mixed(item_code, action);
        log::trace!("delete res rel: uri_mixed={}", uri_mixed);
        let rels = funs.cache().hget(&funs.conf::<IamConfig>().cache_key_res_info, &uri_mixed).await?;
        if let Some(rels) = rels {
            let mut res_dto = TardisFuns::json.str_to_obj::<IamCacheResRelAddOrModifyDto>(&rels)?;
            for account in &delete_req.accounts {
                res_dto.accounts = res_dto.accounts.replace(&format!("#{}#", account), "#");
            }
            for role in &delete_req.roles {
                res_dto.roles = res_dto.roles.replace(&format!("#{}#", role), "#");
            }
            for group in &delete_req.groups {
                res_dto.groups = res_dto.groups.replace(&format!("#{}#", group), "#");
            }
            for app in &delete_req.apps {
                res_dto.apps = res_dto.apps.replace(&format!("#{}#", app), "#");
            }
            for tenant in &delete_req.tenants {
                res_dto.tenants = res_dto.tenants.replace(&format!("#{}#", tenant), "#");
            }
            funs.cache().hset(&funs.conf::<IamConfig>().cache_key_res_info, &uri_mixed, &TardisFuns::json.obj_to_string(&res_dto)?).await?;
            return Self::add_change_trigger(&uri_mixed, funs).await;
        }
        Err(funs.err().not_found("iam_cache_res", "delete", "not found res rel", "404-iam-cache-res-rel-not-exist"))
    }

    async fn add_change_trigger(uri: &str, funs: &TardisFunsInst) -> TardisResult<()> {
        funs.cache()
            .set_ex(
                &format!("{}{}", funs.conf::<IamConfig>().cache_key_res_changed_info_, Utc::now().timestamp_nanos()),
                uri,
                funs.conf::<IamConfig>().cache_key_res_changed_expire_sec,
            )
            .await?;
        Ok(())
    }

    pub fn package_uri_mixed(item_code: &str, action: &str) -> String {
        let domain_idx = item_code.find('/').unwrap_or(item_code.len());
        let domain = &item_code[0..domain_idx];
        let path_and_query = if domain_idx != item_code.len() { &item_code[domain_idx + 1..] } else { "" };
        format!(
            "{}://{}/{}##{}",
            iam_constants::RBUM_KIND_CODE_IAM_RES.to_lowercase(),
            domain.to_lowercase(),
            path_and_query,
            action
        )
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(default)]
struct IamCacheResRelAddOrModifyDto {
    pub accounts: String,
    pub roles: String,
    pub groups: String,
    pub apps: String,
    pub tenants: String,
}

pub struct IamCacheResRelAddOrModifyReq {
    pub st: Option<i64>,
    pub et: Option<i64>,
    pub accounts: Vec<String>,
    pub roles: Vec<String>,
    pub groups: Vec<String>,
    pub apps: Vec<String>,
    pub tenants: Vec<String>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct IamCacheResRelDeleteReq {
    pub accounts: Vec<String>,
    pub roles: Vec<String>,
    pub groups: Vec<String>,
    pub apps: Vec<String>,
    pub tenants: Vec<String>,
}
