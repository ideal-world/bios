use crate::{
    basic::{
        dto::{
            iam_account_dto::{IamAccountModifyReq, IamAccountSummaryResp},
            iam_config_dto::IamConfigSummaryResp,
            iam_filer_dto::IamAccountFilterReq,
        },
        serv::{
            clients::{iam_log_client::LogParamTag, iam_search_client::IamSearchClient},
            iam_account_serv::IamAccountServ,
            iam_platform_serv::IamPlatformServ,
            iam_rel_serv::IamRelServ,
            iam_tenant_serv::IamTenantServ,
        },
    },
    iam_config::{IamBasicConfigApi, IamConfig},
    iam_constants::{self, IAM_AVATAR},
    iam_enumeration::{IamAccountLockStateKind, IamRelKind},
};
use bios_basic::{
    process::task_processor::TaskProcessor,
    rbum::{dto::rbum_filer_dto::RbumBasicFilterReq, serv::rbum_item_serv::RbumItemCrudOperation},
};
use bios_sdk_invoke::clients::spi_log_client::{LogItemFindReq, SpiLogClient};
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    chrono::{DateTime, Duration, Utc},
    TardisFunsInst,
};

use crate::iam_enumeration::IamAccountStatusKind;

pub struct IamCcAccountTaskServ;

impl IamCcAccountTaskServ {
    pub async fn execute_account_search_task(funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Option<String>> {
        let task_ctx = ctx.clone();
        TaskProcessor::execute_task_with_ctx(
            &funs.conf::<IamConfig>().cache_key_async_task_status,
            move |_task_id| async move {
                let funs = iam_constants::get_tardis_inst();
                let account_list = IamAccountServ::find_id_items(
                    &IamAccountFilterReq {
                        basic: RbumBasicFilterReq {
                            ignore_scope: false,
                            rel_ctx_owner: false,
                            own_paths: Some(task_ctx.own_paths.clone()),
                            with_sub_own_paths: true,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    None,
                    None,
                    &funs,
                    &task_ctx,
                )
                .await?;
                let mut num = 0;
                for account in account_list {
                    let id = account;
                    num += 1;
                    if num % 100 == 0 {
                        tardis::tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                    }
                    IamSearchClient::async_add_or_modify_account_search(&id, Box::new(true), "", &funs, &task_ctx).await?;
                    task_ctx.execute_task().await?;
                }
                Ok(())
            },
            &funs.cache(),
            IAM_AVATAR.to_owned(),
            Some(vec![format!("account/{}", ctx.owner)]),
            ctx,
        )
        .await?;
        Ok(None)
    }

    pub async fn execute_account_task(funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Option<String>> {
        let task_ctx = ctx.clone();
        TaskProcessor::execute_task_with_ctx(
            &funs.conf::<IamConfig>().cache_key_async_task_status,
            move |_task_id| async move {
                let mut funs = iam_constants::get_tardis_inst();
                funs.begin().await?;
                let account_list = IamAccountServ::find_items(
                    &IamAccountFilterReq {
                        basic: RbumBasicFilterReq {
                            ignore_scope: false,
                            rel_ctx_owner: false,
                            own_paths: Some(task_ctx.own_paths.clone()),
                            with_sub_own_paths: true,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    None,
                    None,
                    &funs,
                    &task_ctx,
                )
                .await?;
                let admin_account_list = IamRelServ::find_to_simple_rels(&IamRelKind::IamAccountRole, &funs.iam_basic_role_sys_admin_id(), None, None, &funs, &task_ctx)
                    .await?
                    .iter()
                    .map(|r| r.rel_id.clone())
                    .collect::<Vec<String>>();
                let platform_config = IamPlatformServ::get_platform_config_agg(&funs, &task_ctx).await?;
                let mut num = 0;
                for account in account_list {
                    let id = account.id.clone();
                    if admin_account_list.contains(&id) {
                        continue;
                    }
                    num += 1;
                    if num % 100 == 0 {
                        tardis::tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                    }
                    match account.scope_level.clone() {
                        bios_basic::rbum::rbum_enumeration::RbumScopeLevelKind::Private => {
                            if !account.own_paths.is_empty() {
                                let tenant_config = IamTenantServ::get_tenant_config_agg(&account.own_paths, &funs, &task_ctx).await?;
                                Self::task_modify_account_agg(account, tenant_config.config, &funs, &task_ctx).await?;
                            } else {
                                Self::task_modify_account_agg(account, platform_config.config.clone(), &funs, &task_ctx).await?;
                            }
                        }
                        bios_basic::rbum::rbum_enumeration::RbumScopeLevelKind::Root => {
                            Self::task_modify_account_agg(account, platform_config.config.clone(), &funs, &task_ctx).await?;
                        }
                        bios_basic::rbum::rbum_enumeration::RbumScopeLevelKind::L1 => {}
                        bios_basic::rbum::rbum_enumeration::RbumScopeLevelKind::L2 => {}
                        bios_basic::rbum::rbum_enumeration::RbumScopeLevelKind::L3 => {}
                    }
                    IamSearchClient::async_add_or_modify_account_search(&id, Box::new(true), "", &funs, &task_ctx).await?;
                }
                funs.commit().await?;
                task_ctx.execute_task().await?;
                Ok(())
            },
            &funs.cache(),
            IAM_AVATAR.to_owned(),
            Some(vec![format!("account/{}", ctx.owner)]),
            ctx,
        )
        .await?;
        Ok(None)
    }

    async fn task_modify_account_agg(account: IamAccountSummaryResp, configs: Vec<IamConfigSummaryResp>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let (account_temporary_expire, account_temporary_sleep_expire, account_temporary_sleep_logout_expire, account_inactivity_lock) = Self::config(configs);
        let tag: String = LogParamTag::Token.into();
        let token_log_resp = SpiLogClient::findv2(
            LogItemFindReq {
                tag: tag.clone(),
                page_number: 1,
                page_size: 1,
                owners: Some(vec![account.id.clone()]),
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?;
        let account_log = if let Some(log_page) = token_log_resp {
            if !log_page.records.is_empty() {
                Some(log_page.records[0].clone())
            } else {
                None
            }
        } else {
            None
        };
        match account.status {
            IamAccountStatusKind::Active => {
                if account.temporary {
                    if let Some(account_temporary_sleep_expire) = account_temporary_sleep_expire {
                        let expire = account_temporary_sleep_expire.value1.parse().unwrap_or(0);
                        if account_log.is_none() {
                            Self::account_modify_status(&account.id, account.update_time, expire * 30, IamAccountStatusKind::Dormant, funs, ctx).await?;
                        } else if let Some(account_log) = account_log.clone() {
                            Self::account_modify_status(&account.id, account_log.ts, expire * 30, IamAccountStatusKind::Dormant, funs, ctx).await?;
                        }
                    }
                }
            }
            IamAccountStatusKind::Dormant => {
                if account.temporary {
                    if let Some(account_temporary_sleep_logout_expire) = account_temporary_sleep_logout_expire {
                        let expire = account_temporary_sleep_logout_expire.value1.parse().unwrap_or(0);
                        if account_log.is_none() {
                            Self::account_modify_status(&account.id, account.update_time, expire * 30, IamAccountStatusKind::Logout, funs, ctx).await?;
                        } else if let Some(account_log) = account_log.clone() {
                            Self::account_modify_status(&account.id, account_log.ts, expire * 30, IamAccountStatusKind::Logout, funs, ctx).await?;
                        }
                    }
                }
            }
            IamAccountStatusKind::Logout => {}
        }
        if let Some(account_temporary_expire) = account_temporary_expire {
            let expire = account_temporary_expire.value1.parse().unwrap_or(0);
            Self::account_modify_status(&account.id, account.effective_time, expire * 30, IamAccountStatusKind::Dormant, funs, ctx).await?;
        }
        if let Some(account_inactivity_lock) = account_inactivity_lock {
            let expire = account_inactivity_lock.value1.parse().unwrap_or(0);
            if account_log.is_none() {
                Self::account_lock(&account.id, account.update_time, expire * 30, funs, ctx).await?;
            } else if let Some(account_log) = account_log.clone() {
                Self::account_lock(&account.id, account_log.ts, expire * 30, funs, ctx).await?;
            }
        }
        Ok(())
    }

    async fn account_modify_status(
        account_id: &str,
        old_time: DateTime<Utc>,
        expire_day: i64,
        next_status: IamAccountStatusKind,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<()> {
        let current_time = Utc::now();
        if let Some(new_time) = old_time.checked_add_signed(Duration::try_days(expire_day).expect("TimeDelta::days out of bounds")) {
            if current_time < new_time {
                IamAccountServ::modify_item(
                    account_id,
                    &mut IamAccountModifyReq {
                        status: Some(next_status),
                        name: None,
                        scope_level: None,
                        disabled: None,
                        lock_status: None,
                        is_auto: None,
                        icon: None,
                        temporary: None,
                        logout_type: None,
                        labor_type: None,
                    },
                    funs,
                    ctx,
                )
                .await?;
            }
        }
        Ok(())
    }

    async fn account_lock(account_id: &str, old_time: DateTime<Utc>, expire_day: i64, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let current_time = Utc::now();
        if let Some(new_time) = old_time.checked_add_signed(Duration::try_days(expire_day).expect("TimeDelta::days out of bounds")) {
            if current_time < new_time {
                IamAccountServ::modify_item(
                    account_id,
                    &mut IamAccountModifyReq {
                        status: None,
                        name: None,
                        scope_level: None,
                        disabled: None,
                        lock_status: Some(IamAccountLockStateKind::LongTimeNoLoginLocked),
                        is_auto: None,
                        icon: None,
                        temporary: None,
                        logout_type: None,
                        labor_type: None,
                    },
                    funs,
                    ctx,
                )
                .await?;
            }
        }
        Ok(())
    }

    fn config(
        configs: Vec<IamConfigSummaryResp>,
    ) -> (
        Option<IamConfigSummaryResp>,
        Option<IamConfigSummaryResp>,
        Option<IamConfigSummaryResp>,
        Option<IamConfigSummaryResp>,
    ) {
        // 临时账号使用期限
        let account_temporary_expire = configs.iter().find(|x| !x.disabled && x.code == "AccountTemporaryExpire").cloned();
        // 休眠配置
        let account_temporary_sleep_expire = configs.iter().find(|x| !x.disabled && x.code == "AccountTemporarySleepExpire").cloned();
        // 注销配置
        let account_temporary_sleep_logout_expire = configs.iter().find(|x| !x.disabled && x.code == "AccountTemporarySleepRemoveExpire").cloned();
        // 锁定配置
        let account_inactivity_lock = configs.iter().find(|x| !x.disabled && x.code == "AccountInactivityLock").cloned();
        (
            account_temporary_expire,
            account_temporary_sleep_expire,
            account_temporary_sleep_logout_expire,
            account_inactivity_lock,
        )
    }
}
