use std::default::Default;
use std::str::FromStr;

use bios_basic::helper::request_helper::{add_ip, get_remote_ip};
use bios_basic::process::task_processor::TaskProcessor;
use bios_basic::rbum::rbum_config::RbumConfigApi;
use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::chrono::Utc;
use tardis::{log, TardisFuns, TardisFunsInst};

use bios_basic::rbum::dto::rbum_filer_dto::RbumBasicFilterReq;
use bios_basic::rbum::serv::rbum_item_serv::{RbumItemCrudOperation, RbumItemServ};

use crate::basic::dto::iam_account_dto::IamAccountInfoResp;
use crate::basic::dto::iam_cert_dto::IamContextFetchReq;
use crate::basic::dto::iam_filer_dto::{IamAccountFilterReq, IamAppFilterReq};
use crate::basic::serv::clients::iam_log_client::{IamLogClient, LogParamTag};
use crate::basic::serv::iam_account_serv::IamAccountServ;
use crate::basic::serv::iam_app_serv::IamAppServ;
use crate::basic::serv::iam_cert_serv::IamCertServ;
use crate::basic::serv::iam_rel_serv::IamRelServ;
use crate::iam_config::IamConfig;
use crate::iam_constants;
use crate::iam_enumeration::{IamCertTokenKind, IamRelKind};
use crate::iam_initializer::{default_iam_send_avatar, ws_iam_send_client};
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
            format!("{token_kind},{rel_iam_item_id},{}", renewal_expire_sec)
        } else {
            format!("{token_kind},{rel_iam_item_id}")
        };
        log::trace!("add token: token={}", token);
        if expire_sec > 0 {
            funs.cache()
                .set_ex(
                    format!("{}{}", funs.conf::<IamConfig>().cache_key_token_info_, token).as_str(),
                    token_value.as_str(),
                    expire_sec as u64,
                )
                .await?;
        } else {
            funs.cache().set(format!("{}{}", funs.conf::<IamConfig>().cache_key_token_info_, token).as_str(), token_value.as_str()).await?;
        }
        funs.cache()
            .hset(
                format!("{}{}", funs.conf::<IamConfig>().cache_key_account_rel_, rel_iam_item_id).as_str(),
                token,
                &format!("{},{}", token_kind, Utc::now().timestamp_nanos_opt().expect("maybe in 23rd centery")),
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
                Self::delete_token_by_token(&old_token, None, funs).await?;
            }
        }
        Ok(())
    }

    pub async fn delete_token_by_token(token: &str, ip: Option<String>, funs: &TardisFunsInst) -> TardisResult<()> {
        log::trace!("delete token: token={}", token);
        if let Some(token_info) = funs.cache().get(format!("{}{}", funs.conf::<IamConfig>().cache_key_token_info_, token).as_str()).await? {
            let iam_item_id = token_info.split(',').nth(1).unwrap_or("");
            funs.cache().del(format!("{}{}", funs.conf::<IamConfig>().cache_key_token_info_, token).as_str()).await?;
            Self::delete_double_auth(iam_item_id, funs).await?;
            funs.cache().hdel(format!("{}{}", funs.conf::<IamConfig>().cache_key_account_rel_, iam_item_id).as_str(), token).await?;

            let mut mock_ctx = TardisContext::default();
            add_ip(ip, &mock_ctx).await?;
            if let Ok(account_context) = Self::get_account_context(iam_item_id, "", funs).await {
                mock_ctx = account_context;
            } else {
                mock_ctx.owner = iam_item_id.to_string();
                let own_paths = RbumItemServ::get_rbum(
                    iam_item_id,
                    &RbumBasicFilterReq {
                        ignore_scope: true,
                        with_sub_own_paths: true,
                        own_paths: Some("".to_string()),
                        ..Default::default()
                    },
                    funs,
                    &mock_ctx,
                )
                .await?
                .own_paths;
                mock_ctx.own_paths = own_paths;
            }

            let _ = IamLogClient::add_ctx_task(
                LogParamTag::IamAccount,
                Some(iam_item_id.to_string()),
                "下线账号".to_string(),
                Some("OfflineAccount".to_string()),
                &mock_ctx,
            )
            .await;
            let _ = IamLogClient::add_ctx_task(
                LogParamTag::SecurityVisit,
                Some(iam_item_id.to_string()),
                "退出".to_string(),
                Some("Quit".to_string()),
                &mock_ctx,
            )
            .await;

            mock_ctx.execute_task().await?;
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
            move |_task_id| async move {
                let funs = iam_constants::get_tardis_inst();
                let filter = IamAccountFilterReq {
                    basic: RbumBasicFilterReq {
                        own_paths: Some(own_paths),
                        with_sub_own_paths: true,
                        ..Default::default()
                    },
                    ..Default::default()
                };
                let mut count = IamAccountServ::count_items(&filter, &funs, &ctx_clone).await.unwrap_or_default() as isize;
                let mut page_number = 1;
                while count > 0 {
                    let mut ids = Vec::new();
                    if let Ok(page) = IamAccountServ::paginate_id_items(&filter, page_number, 100, None, None, &funs, &ctx_clone).await {
                        ids = page.records;
                    }
                    for id in ids {
                        let account_context = Self::get_account_context(&id, "", &funs).await;
                        if let Ok(account_context) = account_context {
                            if account_context.own_paths == ctx_clone.own_paths {
                                Self::delete_tokens_and_contexts_by_account_id(&id, get_remote_ip(&ctx_clone).await?, &funs).await?;
                            }
                        }
                    }
                    page_number += 1;
                    count -= 100;
                }
                if is_app {
                    let mut count = IamRelServ::count_to_rels(&IamRelKind::IamAccountApp, &tenant_or_app_id, &funs, &ctx_clone).await.unwrap_or_default() as isize;
                    let mut page_number = 1;
                    while count > 0 {
                        let mut ids = Vec::new();
                        if let Ok(page) = IamRelServ::paginate_to_id_rels(&IamRelKind::IamAccountApp, &tenant_or_app_id, page_number, 100, None, None, &funs, &ctx_clone).await {
                            ids = page.records;
                        }
                        for id in ids {
                            let account_context = Self::get_account_context(&id, "", &funs).await;
                            if let Ok(account_context) = account_context {
                                if account_context.own_paths == ctx_clone.own_paths {
                                    Self::delete_tokens_and_contexts_by_account_id(&id, get_remote_ip(&ctx_clone).await?, &funs).await?;
                                }
                            }
                        }
                        page_number += 1;
                        count -= 100;
                    }
                }
                Ok(())
            },
            &funs.cache(),
            ws_iam_send_client().await.clone(),
            default_iam_send_avatar().await.clone(),
            Some(vec![format!("account/{}", ctx.owner)]),
            ctx,
        )
        .await?;
        Ok(())
    }

    pub async fn delete_tokens_and_contexts_by_account_id(account_id: &str, ip: Option<String>, funs: &TardisFunsInst) -> TardisResult<()> {
        log::trace!("delete tokens and contexts: account_id={}", account_id);
        let tokens = funs.cache().hgetall(format!("{}{}", funs.conf::<IamConfig>().cache_key_account_rel_, account_id).as_str()).await?;
        for (token, _) in tokens.iter() {
            funs.cache().del(format!("{}{}", funs.conf::<IamConfig>().cache_key_token_info_, token).as_str()).await?;
        }
        funs.cache().del(format!("{}{}", funs.conf::<IamConfig>().cache_key_account_rel_, account_id).as_str()).await?;
        funs.cache().del(format!("{}{}", funs.conf::<IamConfig>().cache_key_account_info_, account_id).as_str()).await?;

        let mock_ctx = TardisContext { ..Default::default() };
        add_ip(ip, &mock_ctx).await?;
        let _ = IamLogClient::add_ctx_task(
            LogParamTag::IamAccount,
            Some(account_id.to_string()),
            "下线账号".to_string(),
            Some("OfflineAccount".to_string()),
            &mock_ctx,
        )
        .await;

        mock_ctx.execute_task().await?;

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

    pub async fn refresh_account_info_by_account_id(account_id: &str, funs: &TardisFunsInst) -> TardisResult<()> {
        log::trace!("refresh account info: account_id={}", account_id);
        let tenant_info = funs.cache().hget(format!("{}{}", funs.conf::<IamConfig>().cache_key_account_info_, account_id).as_str(), "").await?;
        if tenant_info.is_none() {
            return Ok(());
        }

        let tenant_id = TardisFuns::json.str_to_obj::<TardisContext>(&tenant_info.unwrap())?.own_paths;
        let mock_ctx = TardisContext {
            own_paths: tenant_id.clone(),
            ..Default::default()
        };
        let _ = IamCertServ::package_tardis_account_context_and_resp(account_id, &tenant_id, "".to_string(), None, funs, &mock_ctx).await;
        Ok(())
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
        let mut context = if let Some(context) = funs.cache().hget(format!("{}{}", funs.conf::<IamConfig>().cache_key_account_info_, account_id).as_str(), field).await? {
            TardisFuns::json.str_to_obj::<TardisContext>(&context)?
        } else {
            return Err(funs.err().not_found("get_account_context", "get", "not found context", "404-iam-cache-context-not-exist"));
        };
        if !field.is_empty() {
            if let Some(tenant_context) = funs.cache().hget(&format!("{}{}", funs.conf::<IamConfig>().cache_key_account_info_, account_id), "").await? {
                let tenant_context = TardisFuns::json.str_to_obj::<TardisContext>(&tenant_context)?;
                if !tenant_context.roles.is_empty() {
                    context.roles.extend(tenant_context.roles);
                }
                if !tenant_context.groups.is_empty() {
                    context.groups.extend(tenant_context.groups);
                }
            }
        }
        Ok(context)
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
                    expire_sec as u64,
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

    pub async fn add_gateway_rule_info(ak: &str, rule_name: &str, match_method: Option<&str>, value: &str, funs: &TardisFunsInst) -> TardisResult<()> {
        log::trace!(
            "add gateway_rule_info: ak={},rule_name={},match_method={},value={}",
            ak,
            rule_name,
            match_method.unwrap_or("*"),
            value
        );
        let match_path = &funs.conf::<IamConfig>().gateway_openapi_path;
        funs.cache()
            .set(
                format!(
                    "{}{}:{}:{}:{}",
                    funs.conf::<IamConfig>().cache_key_gateway_rule_info_,
                    rule_name,
                    match_method.unwrap_or("*"),
                    match_path,
                    ak
                )
                .as_str(),
                value,
            )
            .await?;
        Ok(())
    }

    pub async fn get_gateway_cumulative_count(ak: &str, match_method: Option<&str>, funs: &TardisFunsInst) -> TardisResult<Option<String>> {
        let match_path = &funs.conf::<IamConfig>().gateway_openapi_path;
        let result = funs
            .cache()
            .get(&format!(
                "{}{}:{}:{}:{}:cumulative-count",
                funs.conf::<IamConfig>().cache_key_gateway_rule_info_,
                iam_constants::OPENAPI_GATEWAY_PLUGIN_COUNT,
                match_method.unwrap_or("*"),
                match_path,
                ak
            ))
            .await?;
        Ok(result)
    }

    pub async fn get_gateway_rule_info(ak: &str, rule_name: &str, match_method: Option<&str>, funs: &TardisFunsInst) -> TardisResult<Option<String>> {
        let match_path = &funs.conf::<IamConfig>().gateway_openapi_path;
        let result = funs
            .cache()
            .get(
                format!(
                    "{}{}:{}:{}:{}",
                    funs.conf::<IamConfig>().cache_key_gateway_rule_info_,
                    rule_name,
                    match_method.unwrap_or("*"),
                    match_path,
                    ak
                )
                .as_str(),
            )
            .await?;
        Ok(result)
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
                funs.conf::<IamConfig>().cache_key_double_auth_expire_sec as u64,
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
    pub async fn add_res(item_code: &str, action: &str, crypto_req: bool, crypto_resp: bool, double_auth: bool, need_login: bool, funs: &TardisFunsInst) -> TardisResult<()> {
        let uri_mixed = Self::package_uri_mixed(item_code, action);
        log::trace!("add res: uri_mixed={}", uri_mixed);
        let add_res_dto = IamCacheResRelAddOrModifyDto {
            need_crypto_req: crypto_req,
            need_crypto_resp: crypto_resp,
            need_double_auth: double_auth,
            need_login,
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
            need_login: false,
        };
        let uri_mixed = Self::package_uri_mixed(item_code, action);
        let rels = funs.cache().hget(&funs.conf::<IamConfig>().cache_key_res_info, &uri_mixed).await?;
        if let Some(rels) = rels {
            let old_res_dto = TardisFuns::json.str_to_obj::<IamCacheResRelAddOrModifyDto>(&rels)?;
            res_dto.need_crypto_req = old_res_dto.need_crypto_req;
            res_dto.need_crypto_resp = old_res_dto.need_crypto_resp;
            res_dto.need_double_auth = old_res_dto.need_double_auth;
            res_dto.need_login = old_res_dto.need_login;
        }
        funs.cache().hset(&funs.conf::<IamConfig>().cache_key_res_info, &uri_mixed, &TardisFuns::json.obj_to_string(&res_dto)?).await?;
        Self::add_change_trigger(&uri_mixed, funs).await
    }

    pub async fn add_or_modify_res_rel(item_code: &str, action: &str, add_or_modify_req: &IamCacheResRelAddOrModifyReq, funs: &TardisFunsInst) -> TardisResult<()> {
        if add_or_modify_req.st.is_some() || add_or_modify_req.et.is_some() {
            // TODO supports time range
            return Err(funs.err().conflict("iam_cache_res", "add_or_modify", "st and et must be none", "409-iam-cache-res-date-not-none"));
        }
        let mut res_auth = IamCacheResAuth {
            accounts: format!("#{}#", add_or_modify_req.accounts.join("#")),
            roles: format!("#{}#", add_or_modify_req.roles.join("#")),
            groups: format!("#{}#", add_or_modify_req.groups.join("#")),
            apps: format!("#{}#", add_or_modify_req.apps.join("#")),
            tenants: format!("#{}#", add_or_modify_req.tenants.join("#")),
            aks: format!("#{}#", add_or_modify_req.aks.join("#")),
            ..Default::default()
        };
        let mut res_dto = IamCacheResRelAddOrModifyDto {
            auth: None,
            need_crypto_req: false,
            need_crypto_resp: false,
            need_double_auth: false,
            need_login: false,
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
                res_auth.aks = format!("{}{}", res_auth.aks, old_auth.aks);
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
            if let Some(need_login) = add_or_modify_req.need_login {
                res_dto.need_login = need_login
            } else {
                res_dto.need_login = old_res_dto.need_login
            }
        }
        res_auth.accounts = res_auth.accounts.replace("##", "#");
        res_auth.roles = res_auth.roles.replace("##", "#");
        res_auth.groups = res_auth.groups.replace("##", "#");
        res_auth.apps = res_auth.apps.replace("##", "#");
        res_auth.tenants = res_auth.tenants.replace("##", "#");
        res_auth.aks = res_auth.aks.replace("##", "#");

        if (res_auth.accounts == "#" || res_auth.accounts == "##")
            && (res_auth.roles == "#" || res_auth.roles == "##")
            && (res_auth.groups == "#" || res_auth.groups == "##")
            && (res_auth.apps == "#" || res_auth.apps == "##")
            && (res_auth.tenants == "#" || res_auth.tenants == "##")
            && (res_auth.aks == "#" || res_auth.aks == "##")
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
                    while auth.accounts.contains(&format!("#{account}#")) {
                        auth.accounts = auth.accounts.replacen(&format!("#{account}#"), "#", 1);
                    }
                }
                for role in &delete_req.roles {
                    while auth.roles.contains(&format!("#{role}#")) {
                        auth.roles = auth.roles.replacen(&format!("#{role}#"), "#", 1);
                    }
                }
                for group in &delete_req.groups {
                    while auth.groups.contains(&format!("#{group}#")) {
                        auth.groups = auth.groups.replacen(&format!("#{group}#"), "#", 1);
                    }
                }
                for app in &delete_req.apps {
                    while auth.apps.contains(&format!("#{app}#")) {
                        auth.apps = auth.apps.replacen(&format!("#{app}#"), "#", 1);
                    }
                }
                for tenant in &delete_req.tenants {
                    while auth.tenants.contains(&format!("#{tenant}#")) {
                        auth.tenants = auth.tenants.replacen(&format!("#{tenant}#"), "#", 1);
                    }
                }
                for ak in &delete_req.aks {
                    while auth.aks.contains(&format!("#{ak}#")) {
                        auth.aks = auth.aks.replacen(&format!("#{ak}#"), "#", 1);
                    }
                }
                if (auth.accounts == "#" || auth.accounts == "##")
                    && (auth.roles == "#" || auth.roles == "##")
                    && (auth.groups == "#" || auth.groups == "##")
                    && (auth.apps == "#" || auth.apps == "##")
                    && (auth.tenants == "#" || auth.tenants == "##")
                {
                    res_dto.auth = None;
                } else {
                    res_dto.auth = Some(auth);
                }
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
                funs.conf::<IamConfig>().cache_key_res_changed_expire_sec as u64,
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
    pub need_login: bool,
}
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(default)]
struct IamCacheResAuth {
    pub accounts: String,
    pub roles: String,
    pub groups: String,
    pub apps: String,
    pub tenants: String,
    pub aks: String,
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
    pub aks: Vec<String>,
    pub need_crypto_req: Option<bool>,
    pub need_crypto_resp: Option<bool>,
    pub need_double_auth: Option<bool>,
    pub need_login: Option<bool>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct IamCacheResRelDeleteReq {
    pub accounts: Vec<String>,
    pub roles: Vec<String>,
    pub groups: Vec<String>,
    pub apps: Vec<String>,
    pub tenants: Vec<String>,
    pub aks: Vec<String>,
}
