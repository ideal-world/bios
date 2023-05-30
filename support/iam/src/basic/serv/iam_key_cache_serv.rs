use std::collections::HashMap;
use std::default::Default;
use std::str::FromStr;

use bios_basic::process::task_processor::TaskProcessor;
use bios_basic::rbum::rbum_config::RbumConfigApi;
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
use crate::basic::serv::clients::spi_log_client::{LogParamTag, SpiLogClient};
use crate::basic::serv::iam_account_serv::IamAccountServ;
use crate::basic::serv::iam_app_serv::IamAppServ;
use crate::basic::serv::iam_rel_serv::IamRelServ;
use crate::iam_config::IamConfig;
use crate::iam_constants;
use crate::iam_enumeration::{IamCertTokenKind, IamRelKind};
use tardis::tokio::{self, task};
pub struct IamIdentCacheServ;

impl IamIdentCacheServ {
    pub async fn add_token(
        token: &str,
        token_kind: &IamCertTokenKind,
        rel_iam_item_id: &str,
        renewal_expire_sec: Option<i64>,
        expire_sec: i64,
        coexist_num: i16,
        funs: &TardisFunsInst,
    ) -> TardisResult<()> {
        let token_value = if let Some(renewal_expire_sec) = renewal_expire_sec {
            format!("{token_kind},{rel_iam_item_id},{}", renewal_expire_sec.to_string())
        } else {
            format!("{token_kind},{rel_iam_item_id}")
        };
        log::trace!("add token: token={}", token);
        if expire_sec > 0 {
            funs.cache()
                .set_ex(
                    format!("{}{}", funs.conf::<IamConfig>().cache_key_token_info_, token).as_str(),
                    token_value.as_str(),
                    expire_sec as usize,
                )
                .await?;
        } else {
            funs.cache().set(format!("{}{}", funs.conf::<IamConfig>().cache_key_token_info_, token).as_str(), token_value.as_str()).await?;
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
            Self::delete_double_auth(iam_item_id, funs).await?;
            funs.cache().hdel(format!("{}{}", funs.conf::<IamConfig>().cache_key_account_rel_, iam_item_id).as_str(), token).await?;

            let mock_ctx = TardisContext { ..Default::default() };
            let _ = SpiLogClient::add_ctx_task(
                LogParamTag::IamAccount,
                Some(token.to_string()),
                "下线账号".to_string(),
                Some("OfflineAccount".to_string()),
                &mock_ctx,
            )
            .await;
            let _ = SpiLogClient::add_ctx_task(LogParamTag::SecurityVisit, Some(token.to_string()), "退出".to_string(), Some("Quit".to_string()), &mock_ctx).await;

            let task_handle = task::spawn_blocking(move || tokio::runtime::Runtime::new().unwrap().block_on(mock_ctx.execute_task()));
            let _ = task_handle.await;
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
                        let account_context = Self::get_account_context(&id, "", &funs).await;
                        if let Ok(account_context) = account_context {
                            if account_context.own_paths == ctx_clone.own_paths {
                                Self::delete_tokens_and_contexts_by_account_id(&id, &funs).await.unwrap();
                            }
                        }
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
                            let account_context = Self::get_account_context(&id, "", &funs).await;
                            if let Ok(account_context) = account_context {
                                if account_context.own_paths == ctx_clone.own_paths {
                                    Self::delete_tokens_and_contexts_by_account_id(&id, &funs).await.unwrap();
                                }
                            }
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

        let mock_ctx = TardisContext { ..Default::default() };
        let _ = SpiLogClient::add_ctx_task(
            LogParamTag::IamAccount,
            Some(account_id.to_string()),
            "下线账号".to_string(),
            Some("OfflineAccount".to_string()),
            &mock_ctx,
        )
        .await;

        let task_handle = task::spawn_blocking(move || tokio::runtime::Runtime::new().unwrap().block_on(mock_ctx.execute_task()));
        let _ = task_handle.await;

        Ok(())
    }

    pub async fn exist_token_by_account_id(account_id: &str, funs: &TardisFunsInst) -> TardisResult<bool> {
        log::trace!("exist tokens: account_id={}", account_id);
        let tokens = funs.cache().hgetall(format!("{}{}", funs.conf::<IamConfig>().cache_key_account_rel_, account_id).as_str()).await?;
        for (token, _) in tokens.iter() {
            if funs.cache().exists(format!("{}{}", funs.conf::<IamConfig>().cache_key_token_info_, token).as_str()).await? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    pub async fn delete_lock_by_account_id(account_id: &str, funs: &TardisFunsInst) -> TardisResult<()> {
        log::trace!("delete lock: account_id={}", account_id);
        funs.cache().del(&format!("{}{}", funs.rbum_conf_cache_key_cert_locked_(), &account_id)).await?;
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
                    roles: account_info.roles.keys().map(|id| id.to_string()).collect(),
                    groups: account_info.groups.keys().map(|id| id.to_string()).collect(),
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
                        roles: account_app_info.roles.keys().map(|id| id.to_string()).collect(),
                        groups: account_app_info.groups.keys().map(|id| id.to_string()).collect(),
                        ..Default::default()
                    })?,
                )
                .await?;
        }
        Ok(())
    }

    pub async fn get_account_context(account_id: &str, field: &str, funs: &TardisFunsInst) -> TardisResult<TardisContext> {
        if let Some(context) = funs.cache().hget(format!("{}{}", funs.conf::<IamConfig>().cache_key_account_info_, account_id).as_str(), field).await? {
            return TardisFuns::json.str_to_obj(&context);
        }
        Err(funs.err().not_found("get_account_context", "get", "not found context", "404-iam-cache-context-not-exist"))
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

    pub async fn add_aksk(ak: &str, sk: &str, tenant_id: &str, app_id: Option<String>, expire_sec: i64, funs: &TardisFunsInst) -> TardisResult<()> {
        log::trace!("add aksk: ak={},sk={}", ak, sk);
        if expire_sec > 0 {
            funs.cache()
                .set_ex(
                    format!("{}{}", funs.conf::<IamConfig>().cache_key_aksk_info_, ak).as_str(),
                    format!("{},{},{}", sk, tenant_id, app_id.unwrap_or_default()).as_str(),
                    expire_sec as usize,
                )
                .await?;
        } else {
            funs.cache()
                .set(
                    format!("{}{}", funs.conf::<IamConfig>().cache_key_aksk_info_, ak).as_str(),
                    format!("{},{},{}", sk, tenant_id, app_id.unwrap_or_default()).as_str(),
                )
                .await?;
        }
        Ok(())
    }

    pub async fn delete_aksk(ak: &str, funs: &TardisFunsInst) -> TardisResult<()> {
        log::trace!("delete aksk: ak={}", ak);

        funs.cache().del(format!("{}{}", funs.conf::<IamConfig>().cache_key_aksk_info_, ak).as_str()).await?;

        Ok(())
    }

    pub async fn add_double_auth(account_id: &str, funs: &TardisFunsInst) -> TardisResult<()> {
        log::trace!("add double auth: account_id={}", account_id);

        funs.cache()
            .set_ex(
                format!("{}{}", funs.conf::<IamConfig>().cache_key_double_auth_info, account_id).as_str(),
                "",
                funs.conf::<IamConfig>().cache_key_double_auth_expire_sec,
            )
            .await?;

        Ok(())
    }
    pub async fn delete_double_auth(account_id: &str, funs: &TardisFunsInst) -> TardisResult<()> {
        log::trace!("delete double auth: account_id={}", account_id);
        if (funs.cache().get(format!("{}{}", funs.conf::<IamConfig>().cache_key_double_auth_info, account_id).as_str()).await?).is_some() {
            funs.cache().del(format!("{}{}", funs.conf::<IamConfig>().cache_key_double_auth_info, account_id).as_str()).await?;
        }
        Ok(())
    }
}

pub struct IamResCacheServ;

impl IamResCacheServ {
    pub async fn add_res(item_code: &str, action: &str, crypto_req: bool, crypto_resp: bool, double_auth: bool, funs: &TardisFunsInst) -> TardisResult<()> {
        let uri_mixed = Self::package_uri_mixed(item_code, action);
        log::trace!("add res: uri_mixed={}", uri_mixed);
        let add_res_dto = IamCacheResRelAddOrModifyDto {
            need_crypto_req: crypto_req,
            need_crypto_resp: crypto_resp,
            need_double_auth: double_auth,
            ..Default::default()
        };
        funs.cache().hset(&funs.conf::<IamConfig>().cache_key_res_info, &uri_mixed, &TardisFuns::json.obj_to_string(&add_res_dto)?).await?;
        Self::add_change_trigger(&uri_mixed, funs).await
    }

    pub async fn delete_res(item_code: &str, action: &str, funs: &TardisFunsInst) -> TardisResult<()> {
        let uri_mixed = Self::package_uri_mixed(item_code, action);
        log::trace!("delete res: uri_mixed={}", uri_mixed);
        funs.cache().hdel(&funs.conf::<IamConfig>().cache_key_res_info, &uri_mixed).await?;
        Self::add_change_trigger(&uri_mixed, funs).await
    }

    // add anonymous access permissions
    pub async fn add_anonymous_res_rel(item_code: &str, action: &str, st: Option<i64>, et: Option<i64>, funs: &TardisFunsInst) -> TardisResult<()> {
        let res_auth = IamCacheResAuth {
            tenants: "#*#".to_string(),
            st,
            et,
            ..Default::default()
        };
        let mut res_dto = IamCacheResRelAddOrModifyDto {
            auth: Some(res_auth),
            need_crypto_req: false,
            need_crypto_resp: false,
            need_double_auth: false,
        };
        let uri_mixed = Self::package_uri_mixed(item_code, action);
        let rels = funs.cache().hget(&funs.conf::<IamConfig>().cache_key_res_info, &uri_mixed).await?;
        if let Some(rels) = rels {
            let old_res_dto = TardisFuns::json.str_to_obj::<IamCacheResRelAddOrModifyDto>(&rels)?;
            res_dto.need_crypto_req = old_res_dto.need_crypto_req;
            res_dto.need_crypto_resp = old_res_dto.need_crypto_resp;
            res_dto.need_double_auth = old_res_dto.need_double_auth;
        }
        funs.cache().hset(&funs.conf::<IamConfig>().cache_key_res_info, &uri_mixed, &TardisFuns::json.obj_to_string(&res_dto)?).await?;
        Self::add_change_trigger(&uri_mixed, funs).await
    }

    pub async fn add_or_modify_res_rel(item_code: &str, action: &str, add_or_modify_req: &IamCacheResRelAddOrModifyReq, funs: &TardisFunsInst) -> TardisResult<()> {
        if (add_or_modify_req.st.is_some() || add_or_modify_req.et.is_some()) && add_or_modify_req.pwd.is_none() {
            // TODO support time range
            return Err(funs.err().conflict("iam_cache_res", "add_or_modify", "st and et must be none", "409-iam-cache-res-date-not-none"));
        }
        // TODO improve PWD
        let pwd = add_or_modify_req.pwd.clone().map(|pwd| {
            HashMap::from([(
                pwd,
                if let (Some(st), Some(et)) = (add_or_modify_req.st, add_or_modify_req.et) {
                    Some((st, et))
                } else {
                    None
                },
            )])
        });
        let mut res_auth = IamCacheResAuth {
            accounts: format!("#{}#", add_or_modify_req.accounts.join("#")),
            roles: format!("#{}#", add_or_modify_req.roles.join("#")),
            groups: format!("#{}#", add_or_modify_req.groups.join("#")),
            apps: format!("#{}#", add_or_modify_req.apps.join("#")),
            tenants: format!("#{}#", add_or_modify_req.tenants.join("#")),
            pwd,
            ..Default::default()
        };
        let mut res_dto = IamCacheResRelAddOrModifyDto {
            auth: None,
            need_crypto_req: false,
            need_crypto_resp: false,
            need_double_auth: false,
        };
        let uri_mixed = Self::package_uri_mixed(item_code, action);
        log::trace!("add or modify res rel: uri_mixed={}", uri_mixed);
        let rels = funs.cache().hget(&funs.conf::<IamConfig>().cache_key_res_info, &uri_mixed).await?;
        if let Some(rels) = rels {
            let old_res_dto = TardisFuns::json.str_to_obj::<IamCacheResRelAddOrModifyDto>(&rels)?;
            if let Some(old_auth) = old_res_dto.auth {
                res_auth.accounts = format!("{}{}", res_auth.accounts, old_auth.accounts);
                res_auth.roles = format!("{}{}", res_auth.roles, old_auth.roles);
                res_auth.groups = format!("{}{}", res_auth.groups, old_auth.groups);
                res_auth.apps = format!("{}{}", res_auth.apps, old_auth.apps);
                res_auth.tenants = format!("{}{}", res_auth.tenants, old_auth.tenants);
                if let Some(old_pwd) = old_auth.pwd {
                    res_auth.pwd = if let Some(mut res_auth_pwd) = res_auth.pwd.clone() {
                        res_auth_pwd.extend(old_pwd);
                        res_auth.pwd
                    } else {
                        Some(old_pwd)
                    };
                }
            }

            if let Some(need_crypto_req) = add_or_modify_req.need_crypto_req {
                res_dto.need_crypto_req = need_crypto_req
            } else {
                res_dto.need_crypto_req = old_res_dto.need_crypto_req
            }
            if let Some(need_crypto_resp) = add_or_modify_req.need_crypto_resp {
                res_dto.need_crypto_resp = need_crypto_resp
            } else {
                res_dto.need_crypto_resp = old_res_dto.need_crypto_resp
            }
            if let Some(need_double_auth) = add_or_modify_req.need_double_auth {
                res_dto.need_double_auth = need_double_auth
            } else {
                res_dto.need_double_auth = old_res_dto.need_double_auth
            }
        }
        res_auth.accounts = res_auth.accounts.replace("##", "#");
        res_auth.roles = res_auth.roles.replace("##", "#");
        res_auth.groups = res_auth.groups.replace("##", "#");
        res_auth.apps = res_auth.apps.replace("##", "#");
        res_auth.tenants = res_auth.tenants.replace("##", "#");

        if (res_auth.accounts == "#" || res_auth.accounts == "##")
            && (res_auth.roles == "#" || res_auth.roles == "##")
            && (res_auth.groups == "#" || res_auth.groups == "##")
            && (res_auth.apps == "#" || res_auth.apps == "##")
            && (res_auth.tenants == "#" || res_auth.tenants == "##")
        {
            res_dto.auth = None;
        } else {
            res_dto.auth = Some(res_auth);
        }

        funs.cache().hset(&funs.conf::<IamConfig>().cache_key_res_info, &uri_mixed, &TardisFuns::json.obj_to_string(&res_dto)?).await?;
        Self::add_change_trigger(&uri_mixed, funs).await
    }

    pub async fn delete_res_rel(item_code: &str, action: &str, delete_req: &IamCacheResRelDeleteReq, funs: &TardisFunsInst) -> TardisResult<()> {
        let uri_mixed = Self::package_uri_mixed(item_code, action);
        log::trace!("delete res rel: uri_mixed={}", uri_mixed);
        let rels = funs.cache().hget(&funs.conf::<IamConfig>().cache_key_res_info, &uri_mixed).await?;
        if let Some(rels) = rels {
            let mut res_dto = TardisFuns::json.str_to_obj::<IamCacheResRelAddOrModifyDto>(&rels)?;
            if let Some(res_auth) = res_dto.auth {
                let mut auth = res_auth;
                for account in &delete_req.accounts {
                    auth.accounts = auth.accounts.replace(&format!("#{account}#"), "#");
                }
                for role in &delete_req.roles {
                    auth.roles = auth.roles.replace(&format!("#{role}#"), "#");
                }
                for group in &delete_req.groups {
                    auth.groups = auth.groups.replace(&format!("#{group}#"), "#");
                }
                for app in &delete_req.apps {
                    auth.apps = auth.apps.replace(&format!("#{app}#"), "#");
                }
                for tenant in &delete_req.tenants {
                    auth.tenants = auth.tenants.replace(&format!("#{tenant}#"), "#");
                }
                res_dto.auth = Some(auth);
            }

            funs.cache().hset(&funs.conf::<IamConfig>().cache_key_res_info, &uri_mixed, &TardisFuns::json.obj_to_string(&res_dto)?).await?;
            return Self::add_change_trigger(&uri_mixed, funs).await;
        }
        Err(funs.err().not_found("iam_cache_res", "delete", "not found res rel", "404-iam-cache-res-rel-not-exist"))
    }

    async fn add_change_trigger(uri: &str, funs: &TardisFunsInst) -> TardisResult<()> {
        funs.cache()
            .set_ex(
                &format!("{}{}", funs.conf::<IamConfig>().cache_key_res_changed_info_, uri),
                "",
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
    pub auth: Option<IamCacheResAuth>,
    pub need_crypto_req: bool,
    pub need_crypto_resp: bool,
    pub need_double_auth: bool,
}
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(default)]
struct IamCacheResAuth {
    pub accounts: String,
    pub roles: String,
    pub groups: String,
    pub apps: String,
    pub tenants: String,
    // pwd->(st,et)
    pub pwd: Option<HashMap<String, Option<(i64, i64)>>>,
    pub st: Option<i64>,
    pub et: Option<i64>,
}

pub struct IamCacheResRelAddOrModifyReq {
    pub st: Option<i64>,
    pub et: Option<i64>,
    pub accounts: Vec<String>,
    pub roles: Vec<String>,
    pub groups: Vec<String>,
    pub apps: Vec<String>,
    pub tenants: Vec<String>,
    //pwd first
    pub pwd: Option<String>,
    pub need_crypto_req: Option<bool>,
    pub need_crypto_resp: Option<bool>,
    pub need_double_auth: Option<bool>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct IamCacheResRelDeleteReq {
    pub accounts: Vec<String>,
    pub roles: Vec<String>,
    pub groups: Vec<String>,
    pub apps: Vec<String>,
    pub tenants: Vec<String>,
}
