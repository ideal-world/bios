use async_trait::async_trait;
use bios_basic::helper::request_helper::get_remote_ip;
use bios_basic::rbum::rbum_config::RbumConfigApi;
use bios_basic::rbum::rbum_enumeration::RbumRelFromKind;
use bios_sdk_invoke::clients::spi_kv_client::SpiKvClient;
use itertools::Itertools;
use tardis::chrono::Utc;

use std::collections::HashMap;

use tardis::basic::dto::TardisContext;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;

use tardis::db::sea_orm::sea_query::{Alias, Expr, SelectStatement};
use tardis::db::sea_orm::*;

use tardis::web::web_resp::{TardisPage, Void};
use tardis::TardisFunsInst;

use bios_basic::rbum::dto::rbum_filer_dto::{RbumBasicFilterReq, RbumCertFilterReq, RbumItemRelFilterReq};
use bios_basic::rbum::dto::rbum_item_dto::{RbumItemKernelAddReq, RbumItemKernelModifyReq};
use bios_basic::rbum::dto::rbum_rel_dto::RbumRelBoneResp;
use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;
use bios_basic::rbum::serv::rbum_item_serv::{RbumItemCrudOperation, RbumItemServ};

use crate::basic::domain::iam_account;
use crate::basic::dto::iam_account_dto::{
    AccountTenantInfo, AccountTenantInfoResp, IamAccountAddReq, IamAccountAggAddReq, IamAccountAggModifyReq, IamAccountAppInfoResp, IamAccountAttrResp, IamAccountDetailAggResp,
    IamAccountDetailResp, IamAccountModifyReq, IamAccountSelfModifyReq, IamAccountSummaryAggResp, IamAccountSummaryResp,
};
use crate::basic::dto::iam_cert_dto::{IamCertMailVCodeAddReq, IamCertPhoneVCodeAddReq, IamCertUserPwdAddReq};
use crate::basic::dto::iam_filer_dto::{IamAccountFilterReq, IamAppFilterReq, IamTenantFilterReq};
use crate::basic::dto::iam_set_dto::IamSetItemAddReq;
use crate::basic::serv::iam_attr_serv::IamAttrServ;
use crate::basic::serv::iam_cert_mail_vcode_serv::IamCertMailVCodeServ;
use crate::basic::serv::iam_cert_phone_vcode_serv::IamCertPhoneVCodeServ;
use crate::basic::serv::iam_cert_serv::IamCertServ;
use crate::basic::serv::iam_cert_user_pwd_serv::IamCertUserPwdServ;
use crate::basic::serv::iam_key_cache_serv::IamIdentCacheServ;
use crate::basic::serv::iam_rel_serv::IamRelServ;
use crate::basic::serv::iam_role_serv::IamRoleServ;
use crate::basic::serv::iam_set_serv::IamSetServ;
use crate::basic::serv::iam_tenant_serv::IamTenantServ;
use crate::iam_config::{IamBasicInfoManager, IamConfig};
use crate::iam_enumeration::{IamAccountLockStateKind, IamAccountStatusKind, IamCertKernelKind, IamRelKind, IamSetKind};

use super::clients::iam_log_client::{IamLogClient, LogParamTag};
use super::clients::iam_search_client::IamSearchClient;
use super::clients::mail_client::MailClient;
use super::clients::sms_client::SmsClient;
use super::iam_app_serv::IamAppServ;

pub struct IamAccountServ;

#[async_trait]
impl RbumItemCrudOperation<iam_account::ActiveModel, IamAccountAddReq, IamAccountModifyReq, IamAccountSummaryResp, IamAccountDetailResp, IamAccountFilterReq> for IamAccountServ {
    fn get_ext_table_name() -> &'static str {
        iam_account::Entity.table_name()
    }

    fn get_rbum_kind_id() -> Option<String> {
        Some(IamBasicInfoManager::get_config(|conf| conf.kind_account_id.clone()))
    }

    fn get_rbum_domain_id() -> Option<String> {
        Some(IamBasicInfoManager::get_config(|conf| conf.domain_iam_id.clone()))
    }

    async fn package_item_add(add_req: &IamAccountAddReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<RbumItemKernelAddReq> {
        Ok(RbumItemKernelAddReq {
            id: add_req.id.clone(),
            name: add_req.name.clone(),
            disabled: add_req.disabled,
            scope_level: add_req.scope_level.clone(),
            ..Default::default()
        })
    }

    async fn package_ext_add(id: &str, add_req: &IamAccountAddReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<iam_account::ActiveModel> {
        Ok(iam_account::ActiveModel {
            id: Set(id.to_string()),
            icon: Set(add_req.icon.as_ref().unwrap_or(&"".to_string()).to_string()),
            status: Set(add_req.status.as_ref().unwrap_or(&IamAccountStatusKind::Active).to_int()),
            temporary: Set(add_req.temporary.unwrap_or(false)),
            lock_status: Set(add_req.lock_status.as_ref().unwrap_or(&IamAccountLockStateKind::Unlocked).to_int()),
            ext1_idx: Set("".to_string()),
            ext2_idx: Set("".to_string()),
            ext3_idx: Set("".to_string()),
            ext4: Set("".to_string()),
            ext5: Set("".to_string()),
            ext6: Set("".to_string()),
            ext7: Set("".to_string()),
            ext8: Set("".to_string()),
            ext9: Set("".to_string()),
            ..Default::default()
        })
    }

    async fn package_item_modify(_: &str, modify_req: &IamAccountModifyReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<Option<RbumItemKernelModifyReq>> {
        if modify_req.name.is_none() && modify_req.scope_level.is_none() && modify_req.status.is_none() && modify_req.disabled.is_none() {
            return Ok(None);
        }
        let disabled = if let Some(status) = modify_req.status.as_ref() {
            match status {
                IamAccountStatusKind::Active => Some(false),
                IamAccountStatusKind::Dormant => Some(true),
                IamAccountStatusKind::Logout => Some(true),
            }
        } else {
            None
        };
        Ok(Some(RbumItemKernelModifyReq {
            code: None,
            name: modify_req.name.clone(),
            scope_level: modify_req.scope_level.clone(),
            disabled: modify_req.disabled.or(disabled),
        }))
    }

    async fn package_ext_modify(id: &str, modify_req: &IamAccountModifyReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<Option<iam_account::ActiveModel>> {
        if modify_req.icon.is_none() && modify_req.status.is_none() && modify_req.lock_status.is_none() && modify_req.temporary.is_none() {
            return Ok(None);
        }
        let mut iam_account = iam_account::ActiveModel {
            id: Set(id.to_string()),
            ..Default::default()
        };
        if let Some(icon) = &modify_req.icon {
            iam_account.icon = Set(icon.to_string());
        }
        if let Some(status) = &modify_req.status {
            iam_account.status = Set(status.to_int());
            if status == &IamAccountStatusKind::Active {
                iam_account.effective_time = Set(Utc::now());
            }
        }
        if let Some(lock_status) = &modify_req.lock_status {
            iam_account.lock_status = Set(lock_status.to_int());
        }
        if let Some(temporary) = &modify_req.temporary {
            iam_account.temporary = Set(*temporary);
        }
        Ok(Some(iam_account))
    }

    async fn after_modify_item(id: &str, modify_req: &mut IamAccountModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        if modify_req.disabled.is_some() || modify_req.scope_level.is_some() || modify_req.status.is_some() {
            IamIdentCacheServ::delete_tokens_and_contexts_by_account_id(id, get_remote_ip(ctx).await?, funs).await?;
        }

        let mut tasks = vec![];
        if modify_req.status == Some(IamAccountStatusKind::Logout) {
            tasks.push(("注销账号".to_string(), "Logout".to_string()));
        }
        if modify_req.status == Some(IamAccountStatusKind::Dormant) {
            tasks.push(("休眠账号".to_string(), "DormantAccount".to_string()));
        }
        if modify_req.status == Some(IamAccountStatusKind::Active) {
            tasks.push(("激活账号".to_string(), "ActivateAccount".to_string()));
        }
        if modify_req.lock_status == Some(IamAccountLockStateKind::Unlocked) {
            tasks.push(("解锁账号".to_string(), "UnlockAccount".to_string()));
        }
        if modify_req.icon.is_some() {
            tasks.push(("修改账号头像".to_string(), "ModifyAccountIcon".to_string()));
        }
        if let Some(name) = modify_req.name.as_ref() {
            tasks.push((format!("修改姓名为{}", name), "ModifyName".to_string()));
        }
        for (op_describe, op_kind) in tasks {
            let _ = IamLogClient::add_ctx_task(LogParamTag::IamAccount, Some(id.to_string()), op_describe, Some(op_kind), ctx).await;
        }
        #[cfg(feature = "spi_kv")]
        Self::add_or_modify_account_kv(id, funs, ctx).await?;

        Ok(())
    }

    async fn after_add_item(id: &str, add_req: &mut IamAccountAddReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let mut op_describe = "添加长期账号".to_string();
        let mut op_kind = "AddLongTermAccount".to_string();
        if add_req.temporary == Some(true) {
            op_describe = "添加临时账号".to_string();
            op_kind = "AddTempAccount".to_string();
        }
        let _ = IamLogClient::add_ctx_task(LogParamTag::IamAccount, Some(id.to_string()), op_describe, Some(op_kind), ctx).await;

        #[cfg(feature = "spi_kv")]
        Self::add_or_modify_account_kv(id, funs, ctx).await?;

        Ok(())
    }
    async fn before_delete_item(id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Option<IamAccountDetailResp>> {
        if id == ctx.owner {
            return Err(funs.err().conflict(&Self::get_obj_name(), "delete", "account invalid", "409-iam-current-can-not-account-delete"));
        }
        Ok(None)
    }

    async fn after_delete_item(id: &str, _: &Option<IamAccountDetailResp>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        IamIdentCacheServ::delete_tokens_and_contexts_by_account_id(id, get_remote_ip(ctx).await?, funs).await?;
        IamSearchClient::async_delete_account_search(id.to_string(), funs, ctx.clone()).await?;
        Ok(())
    }

    async fn package_ext_query(query: &mut SelectStatement, _: bool, filter: &IamAccountFilterReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<()> {
        query.column((iam_account::Entity, iam_account::Column::Icon));
        query.column((iam_account::Entity, iam_account::Column::Status));
        query.column((iam_account::Entity, iam_account::Column::Temporary));
        query.column((iam_account::Entity, iam_account::Column::LockStatus));
        query.column((iam_account::Entity, iam_account::Column::Ext1Idx));
        query.column((iam_account::Entity, iam_account::Column::Ext2Idx));
        query.column((iam_account::Entity, iam_account::Column::Ext3Idx));
        query.column((iam_account::Entity, iam_account::Column::Ext4));
        query.column((iam_account::Entity, iam_account::Column::Ext5));
        query.column((iam_account::Entity, iam_account::Column::Ext6));
        query.column((iam_account::Entity, iam_account::Column::Ext7));
        query.column((iam_account::Entity, iam_account::Column::Ext8));
        query.column((iam_account::Entity, iam_account::Column::Ext9));
        query.column((iam_account::Entity, iam_account::Column::EffectiveTime));
        if let Some(icon) = &filter.icon {
            query.and_where(Expr::col(iam_account::Column::Icon).eq(icon.as_str()));
        }
        if let Some(set_rel) = &filter.set_rel {
            Self::package_set_rel(query, Alias::new("rbum_set_rel"), set_rel);
        }
        Ok(())
    }
}

impl IamAccountServ {
    /// if add_req.status is None.default is RbumCertStatusKind::Enabled
    pub async fn add_account_agg(add_req: &IamAccountAggAddReq, is_ignore_check_sk: bool, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<String> {
        let attrs = IamAttrServ::find_account_attrs(funs, ctx).await?;
        if attrs.iter().any(|i| i.required && !add_req.exts.contains_key(&i.name)) {
            return Err(funs.err().bad_request(&Self::get_obj_name(), "add", "missing required field", "400-iam-account-field-missing"));
        }
        let mut is_ignore_check_sk = is_ignore_check_sk;
        let pwd: String = if let Some(cert_password) = &add_req.cert_password {
            cert_password.to_string()
        } else {
            is_ignore_check_sk = true;
            IamCertServ::get_new_pwd()
        };
        let account_id = IamAccountServ::add_item(
            &mut IamAccountAddReq {
                id: add_req.id.clone(),
                name: add_req.name.clone(),
                scope_level: add_req.scope_level.clone(),
                disabled: add_req.disabled,
                icon: add_req.icon.clone(),
                temporary: add_req.temporary,
                status: None,
                lock_status: add_req.lock_status.clone(),
            },
            funs,
            ctx,
        )
        .await?;
        if let Some(cert_conf) = IamCertServ::get_cert_conf_id_and_ext_opt_by_kind(&IamCertKernelKind::UserPwd.to_string(), Some(ctx.own_paths.clone()), funs).await? {
            IamCertUserPwdServ::add_cert(
                &IamCertUserPwdAddReq {
                    ak: add_req.cert_user_name.clone(),
                    sk: TrimString(pwd.clone()),
                    status: add_req.status.clone(),
                    is_ignore_check_sk,
                },
                &account_id,
                Some(cert_conf.id),
                funs,
                ctx,
            )
            .await?;
        }
        if let Some(cert_phone) = &add_req.cert_phone {
            if let Some(cert_conf) = IamCertServ::get_cert_conf_id_and_ext_opt_by_kind(&IamCertKernelKind::PhoneVCode.to_string(), Some(ctx.own_paths.clone()), funs).await? {
                IamCertPhoneVCodeServ::add_cert(
                    &IamCertPhoneVCodeAddReq {
                        phone: TrimString(cert_phone.to_string()),
                    },
                    &account_id,
                    &cert_conf.id,
                    funs,
                    ctx,
                )
                .await?;
            }
            let _ = SmsClient::async_send_pwd(cert_phone, &pwd, funs, ctx).await;
        }
        if let Some(cert_mail) = &add_req.cert_mail {
            if let Some(cert_conf) = IamCertServ::get_cert_conf_id_and_ext_opt_by_kind(&IamCertKernelKind::MailVCode.to_string(), Some(ctx.own_paths.clone()), funs).await? {
                IamCertMailVCodeServ::add_cert(&IamCertMailVCodeAddReq { mail: cert_mail.to_string() }, &account_id, &cert_conf.id, funs, ctx).await?;
            }
            let _ = MailClient::async_send_pwd(cert_mail, &pwd, funs, ctx).await;
        }
        if let Some(role_ids) = &add_req.role_ids {
            for role_id in role_ids {
                IamRoleServ::add_rel_account(role_id, &account_id, None, funs, ctx).await?;
            }
        }
        if let Some(org_cate_ids) = &add_req.org_node_ids {
            let set_id = IamSetServ::get_default_set_id_by_ctx(&IamSetKind::Org, funs, ctx).await?;
            for org_node_id in org_cate_ids {
                IamSetServ::add_set_item(
                    &IamSetItemAddReq {
                        set_id: set_id.clone(),
                        set_cate_id: org_node_id.to_string(),
                        sort: 0,
                        rel_rbum_item_id: account_id.to_string(),
                    },
                    funs,
                    ctx,
                )
                .await?;
            }
        }
        IamAttrServ::add_or_modify_account_attr_values(&account_id, add_req.exts.clone(), funs, ctx).await?;
        Ok(account_id)
    }

    pub async fn modify_account_agg(id: &str, modify_req: &IamAccountAggModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        IamAccountServ::modify_item(
            id,
            &mut IamAccountModifyReq {
                name: modify_req.name.clone(),
                scope_level: modify_req.scope_level.clone(),
                disabled: modify_req.disabled,
                temporary: modify_req.temporary,
                icon: modify_req.icon.clone(),
                status: modify_req.status.clone(),
                is_auto: Some(false),
                lock_status: None,
            },
            funs,
            ctx,
        )
        .await?;
        if let Some(input_role_ids) = &modify_req.role_ids {
            let stored_roles = Self::find_simple_rel_roles(id, true, None, None, funs, ctx).await?;
            let stored_role_ids: Vec<String> = stored_roles.into_iter().map(|r| r.rel_id).collect();
            for input_role_id in input_role_ids {
                if !stored_role_ids.contains(input_role_id) {
                    IamRoleServ::add_rel_account(input_role_id, id, None, funs, ctx).await?;
                }
            }
            for stored_role_id in stored_role_ids {
                if !input_role_ids.contains(&stored_role_id) {
                    IamRoleServ::delete_rel_account(&stored_role_id, id, None, funs, ctx).await?;
                }
            }
        }
        // TODO test
        if let Some(input_org_cate_ids) = &modify_req.org_cate_ids {
            let set_id = IamSetServ::get_default_set_id_by_ctx(&IamSetKind::Org, funs, ctx).await?;
            let stored_cates = IamSetServ::find_set_items(Some(set_id.clone()), None, Some(id.to_string()), None, false, None, funs, ctx).await?;
            let mut stored_cate_ids: Vec<String> = stored_cates.iter().map(|r| r.rel_rbum_set_cate_id.clone().unwrap_or_default()).collect();
            stored_cate_ids.dedup();
            for input_org_cate_id in input_org_cate_ids {
                if !stored_cate_ids.contains(input_org_cate_id) {
                    IamSetServ::add_set_item(
                        &IamSetItemAddReq {
                            set_id: set_id.clone(),
                            set_cate_id: input_org_cate_id.to_string(),
                            sort: 0,
                            rel_rbum_item_id: id.to_string(),
                        },
                        funs,
                        ctx,
                    )
                    .await?;
                }
            }
            let deleted_item_ids: Vec<String> =
                stored_cates.into_iter().filter(|r| !input_org_cate_ids.contains(&r.rel_rbum_set_cate_id.clone().unwrap_or_default())).map(|r| r.id).unique().collect();
            for deleted_item_id in deleted_item_ids {
                IamSetServ::delete_set_item(&deleted_item_id, funs, ctx).await?;
            }
        }
        if let Some(cert_phone) = &modify_req.cert_phone {
            if let Some(cert_conf) = IamCertServ::get_cert_conf_id_and_ext_opt_by_kind(&IamCertKernelKind::PhoneVCode.to_string(), Some(ctx.own_paths.clone()), funs).await? {
                IamCertPhoneVCodeServ::add_or_modify_cert(cert_phone, id, &cert_conf.id, funs, ctx).await?;
            }
        }
        if let Some(cert_mail) = &modify_req.cert_mail {
            if let Some(cert_conf) = IamCertServ::get_cert_conf_id_and_ext_opt_by_kind(&IamCertKernelKind::MailVCode.to_string(), Some(ctx.own_paths.clone()), funs).await? {
                IamCertMailVCodeServ::add_or_modify_cert(cert_mail, id, &cert_conf.id, funs, ctx).await?;
            }
        }
        if let Some(exts) = &modify_req.exts {
            IamAttrServ::add_or_modify_account_attr_values(id, exts.clone(), funs, ctx).await?;
        }
        Ok(())
    }

    pub async fn self_modify_account(modify_req: &mut IamAccountSelfModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let id = &ctx.owner;
        let account = IamAccountServ::peek_item(
            id,
            &IamAccountFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?;
        let mock_ctx = TardisContext {
            owner: account.id.clone(),
            own_paths: account.own_paths.clone(),
            ..ctx.clone()
        };
        IamAccountServ::modify_item(
            id,
            &mut IamAccountModifyReq {
                name: modify_req.name.clone(),
                icon: modify_req.icon.clone(),
                disabled: modify_req.disabled,
                scope_level: None,
                status: None,
                lock_status: None,
                is_auto: None,
                temporary: None,
            },
            funs,
            &mock_ctx,
        )
        .await?;
        IamAttrServ::add_or_modify_account_attr_values(id, modify_req.exts.clone(), funs, &mock_ctx).await?;
        Ok(())
    }

    // TODO To optimize
    pub async fn get_account_detail_aggs(
        account_id: &str,
        filter: &IamAccountFilterReq,
        use_sys_org: bool,
        use_sys_cert: bool,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<IamAccountDetailAggResp> {
        let account = IamAccountServ::get_item(account_id, filter, funs, ctx).await?;
        let mut mock_tenant_ctx = ctx.clone();
        mock_tenant_ctx.own_paths = IamTenantServ::get_id_by_ctx(ctx, funs)?;
        let set_id = if use_sys_org {
            IamSetServ::get_set_id_by_code(&IamSetServ::get_default_code(&IamSetKind::Org, ""), true, funs, ctx).await?
        } else {
            IamSetServ::get_set_id_by_code(&IamSetServ::get_default_code(&IamSetKind::Org, &IamTenantServ::get_id_by_ctx(ctx, funs)?), true, funs, ctx).await?
            // IamSetServ::get_default_set_id_by_ctx(&IamSetKind::Org, funs, ctx).await?
        };
        let raw_roles = Self::find_simple_rel_roles(&account.id, true, Some(true), None, funs, ctx).await?;
        let mut roles: Vec<RbumRelBoneResp> = vec![];
        for role in raw_roles {
            if !IamRoleServ::is_disabled(&role.rel_id, funs).await? {
                roles.push(role)
            }
        }

        let enabled_apps = IamAppServ::find_items(
            &IamAppFilterReq {
                basic: RbumBasicFilterReq {
                    ignore_scope: false,
                    rel_ctx_owner: false,
                    with_sub_own_paths: true,
                    enabled: Some(true),
                    ..Default::default()
                },
                rel: Some(RbumItemRelFilterReq {
                    rel_by_from: false,
                    is_left: false,
                    tag: Some(IamRelKind::IamAccountApp.to_string()),
                    from_rbum_kind: Some(RbumRelFromKind::Item),
                    rel_item_id: Some(account.id.clone()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            None,
            None,
            funs,
            ctx,
        )
        .await?;
        let mut apps: Vec<IamAccountAppInfoResp> = vec![];
        for app in enabled_apps {
            let mut mock_app_ctx = ctx.clone();
            mock_app_ctx.own_paths = app.own_paths.clone();
            let set_id = IamSetServ::get_set_id_by_code(&IamSetServ::get_default_code(&IamSetKind::Org, &app.own_paths), true, funs, &mock_app_ctx).await?;
            let groups = IamSetServ::find_flat_set_items(&set_id, account_id, true, funs, &mock_app_ctx).await?;
            apps.push(IamAccountAppInfoResp {
                app_id: app.id,
                app_name: app.name,
                app_icon: app.icon,
                roles: roles.iter().filter(|r| r.rel_own_paths == app.own_paths).map(|r| (r.rel_id.to_string(), r.rel_name.to_string())).collect(),
                groups,
            });
        }
        let account_attrs = IamAttrServ::find_account_attrs(funs, ctx).await?;
        let account_attr_values = IamAttrServ::find_account_attr_values(&account.id, funs, ctx).await?;

        let org_set_id = IamSetServ::get_set_id_by_code(&IamSetServ::get_default_code(&IamSetKind::Org, &ctx.own_paths), false, funs, ctx).await?;
        let groups = IamSetServ::find_flat_set_items(&org_set_id, &account.id, false, funs, ctx).await?;
        let account = IamAccountDetailAggResp {
            id: account.id.clone(),
            name: account.name,
            own_paths: account.own_paths,
            owner: account.owner,
            owner_name: account.owner_name,
            create_time: account.create_time,
            update_time: account.update_time,
            effective_time: account.effective_time,
            scope_level: account.scope_level,
            disabled: account.disabled,
            is_locked: funs.cache().exists(&format!("{}{}", funs.rbum_conf_cache_key_cert_locked_(), &account.id.clone())).await?,
            is_online: IamIdentCacheServ::exist_token_by_account_id(&account.id, funs).await?,
            status: account.status,
            temporary: account.temporary,
            lock_status: account.lock_status,
            icon: account.icon,
            roles: roles.iter().filter(|r| r.rel_own_paths == ctx.own_paths).map(|r| (r.rel_id.to_string(), r.rel_name.to_string())).collect(),
            apps,
            groups,
            certs: IamCertServ::find_certs(
                &RbumCertFilterReq {
                    basic: RbumBasicFilterReq {
                        own_paths: Some(if use_sys_cert { "".to_string() } else { IamTenantServ::get_id_by_ctx(ctx, funs)? }),
                        with_sub_own_paths: true,
                        ..Default::default()
                    },
                    rel_rbum_id: Some(account.id.clone()),
                    ..Default::default()
                },
                None,
                None,
                funs,
                ctx,
            )
            .await?
            .into_iter()
            .map(|r| (r.rel_rbum_cert_conf_code.unwrap_or("".to_string()), r.ak))
            .collect(),
            orgs: IamSetServ::find_set_paths(&account.id, &set_id, funs, &mock_tenant_ctx).await?.into_iter().map(|r| r.into_iter().map(|rr| rr.name).join("/")).collect(),
            exts: account_attrs
                .into_iter()
                .map(|r| IamAccountAttrResp {
                    name: r.name.clone(),
                    label: r.label,
                    value: account_attr_values.get(&r.name).unwrap_or(&("".to_string())).to_string(),
                })
                .collect(),
        };
        Ok(account)
    }

    pub async fn paginate_account_summary_aggs(
        filter: &IamAccountFilterReq,
        use_sys_org: bool,
        use_sys_cert: bool,
        page_number: u32,
        page_size: u32,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<TardisPage<IamAccountSummaryAggResp>> {
        let accounts = IamAccountServ::paginate_items(filter, page_number, page_size, desc_sort_by_create, desc_sort_by_update, funs, ctx).await?;
        let mut account_aggs = Vec::with_capacity(accounts.total_size as usize);
        let mock_ctx = if use_sys_org {
            ctx.clone()
        } else {
            TardisContext {
                own_paths: IamTenantServ::get_id_by_ctx(ctx, funs)?,
                ..ctx.clone()
            }
        };
        let set_id = if use_sys_org {
            IamSetServ::get_set_id_by_code(&IamSetServ::get_default_code(&IamSetKind::Org, ""), true, funs, ctx).await?
        } else {
            IamSetServ::get_set_id_by_code(
                &IamSetServ::get_default_code(&IamSetKind::Org, &IamTenantServ::get_id_by_ctx(ctx, funs)?),
                true,
                funs,
                &mock_ctx,
            )
            .await?
            // IamSetServ::get_default_set_id_by_ctx(&IamSetKind::Org, funs, ctx).await?
        };
        for account in accounts.records {
            account_aggs.push(IamAccountSummaryAggResp {
                id: account.id.clone(),
                name: account.name,
                own_paths: account.own_paths,
                owner: account.owner,
                create_time: account.create_time,
                update_time: account.update_time,
                effective_time: account.effective_time,
                scope_level: account.scope_level,
                disabled: account.disabled,
                is_locked: funs.cache().exists(&format!("{}{}", funs.rbum_conf_cache_key_cert_locked_(), &account.id.clone())).await?,
                is_online: IamIdentCacheServ::exist_token_by_account_id(&account.id, funs).await?,
                status: account.status,
                temporary: account.temporary,
                lock_status: account.lock_status,
                icon: account.icon,
                roles: Self::find_simple_rel_roles(&account.id, true, None, None, funs, ctx).await?.into_iter().map(|r| (r.rel_id, r.rel_name)).collect(),
                certs: IamCertServ::find_certs(
                    &RbumCertFilterReq {
                        basic: RbumBasicFilterReq {
                            own_paths: if use_sys_cert {
                                Some("".to_string())
                            } else {
                                Some(IamTenantServ::get_id_by_ctx(ctx, funs)?)
                            },
                            with_sub_own_paths: use_sys_cert,
                            ..Default::default()
                        },
                        rel_rbum_id: Some(account.id.clone()),
                        ..Default::default()
                    },
                    None,
                    None,
                    funs,
                    ctx,
                )
                .await?
                .into_iter()
                .map(|r| (r.rel_rbum_cert_conf_code.unwrap_or("".to_string()), r.ak))
                .collect(),
                orgs: IamSetServ::find_set_paths(&account.id, &set_id, funs, ctx).await?.into_iter().map(|r| r.into_iter().map(|rr| rr.name).join("/")).collect(),
            });
        }
        Ok(TardisPage {
            page_size: accounts.page_size,
            page_number: accounts.page_number,
            total_size: accounts.total_size,
            records: account_aggs,
        })
    }

    pub async fn get_account_tenant_info(id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<AccountTenantInfoResp> {
        let mut tenant_ids = Vec::new();
        let raw_roles = Self::find_simple_rel_roles(id, true, Some(true), None, funs, ctx).await?;
        let mut roles: Vec<RbumRelBoneResp> = vec![];
        for role in raw_roles {
            if !IamRoleServ::is_disabled(&role.rel_id, funs).await? {
                let rel_own_path = role.rel_own_paths.clone();
                let tenant_id = rel_own_path.split('/').collect::<Vec<_>>();
                if !tenant_ids.contains(&tenant_id[0].to_string()) && !tenant_id[0].is_empty() {
                    tenant_ids.push(tenant_id[0].to_string());
                }
                roles.push(role);
            }
        }

        let mut tenant_info = HashMap::new();
        for tenant_id in tenant_ids {
            let tenant_ctx = IamCertServ::use_tenant_ctx(ctx.clone(), &tenant_id)?;
            let tenant_result = IamAccountServ::get_account_detail_aggs(
                id,
                &IamAccountFilterReq {
                    basic: RbumBasicFilterReq {
                        with_sub_own_paths: true,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                false,
                true,
                funs,
                &tenant_ctx,
            )
            .await?;
            let account_tenant_info = AccountTenantInfo {
                roles: tenant_result.roles,
                orgs: tenant_result.orgs,
                groups: tenant_result.groups,
                apps: tenant_result.apps,
            };
            tenant_info.insert(
                IamTenantServ::peek_item(&tenant_id, &IamTenantFilterReq::default(), funs, ctx).await?.name,
                account_tenant_info,
            );
        }

        let tenant_info = AccountTenantInfoResp { tenant_info };
        Ok(tenant_info)
    }

    pub async fn find_name_by_ids(ids: Vec<String>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Vec<String>> {
        IamAccountServ::find_items(
            &IamAccountFilterReq {
                basic: RbumBasicFilterReq {
                    ids: Some(ids),
                    with_sub_own_paths: true,
                    own_paths: Some("".to_string()),
                    ..Default::default()
                },
                ..Default::default()
            },
            None,
            None,
            funs,
            ctx,
        )
        .await
        .map(|r| r.into_iter().map(|r| format!("{},{},{}", r.id, r.name, r.icon)).collect())
    }

    pub async fn find_account_online_by_ids(ids: Vec<String>, funs: &TardisFunsInst, _ctx: &TardisContext) -> TardisResult<Vec<String>> {
        let mut online_accounts = vec![];
        for id in ids {
            online_accounts.push(format!("{},{}", id, IamIdentCacheServ::exist_token_by_account_id(&id, funs).await?));
        }
        Ok(online_accounts)
    }

    pub async fn find_account_lock_state_by_ids(ids: Vec<String>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Vec<String>> {
        let mut lock_accounts: Vec<String> = vec![];
        let accounts = IamAccountServ::find_items(
            &IamAccountFilterReq {
                basic: RbumBasicFilterReq {
                    ids: Some(ids.clone()),
                    with_sub_own_paths: true,
                    own_paths: Some("".to_string()),
                    ..Default::default()
                },
                ..Default::default()
            },
            None,
            None,
            funs,
            ctx,
        )
        .await?;
        for id in ids {
            if let Some(account) = accounts.iter().find(|r| r.id == id) {
                let lock_status = if funs.cache().exists(&format!("{}{}", funs.rbum_conf_cache_key_cert_locked_(), &id)).await? {
                    IamAccountLockStateKind::PasswordLocked
                } else {
                    account.lock_status.clone()
                };
                lock_accounts.push(format!("{},{}", id, lock_status));
            } else {
                continue;
            }
        }
        Ok(lock_accounts)
    }

    pub async fn find_simple_rel_roles(
        account_id: &str,
        with_sub: bool,
        desc_by_create: Option<bool>,
        desc_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<Vec<RbumRelBoneResp>> {
        IamRelServ::find_from_simple_rels(&IamRelKind::IamAccountRole, with_sub, account_id, desc_by_create, desc_by_update, funs, ctx).await
    }

    pub async fn delete_tokens(id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        RbumItemServ::check_ownership(id, funs, ctx).await?;
        IamIdentCacheServ::delete_tokens_and_contexts_by_account_id(id, get_remote_ip(ctx).await?, funs).await
    }

    pub async fn unlock_account(id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Void> {
        RbumItemServ::check_ownership(id, funs, ctx).await?;
        IamIdentCacheServ::delete_lock_by_account_id(id, funs).await?;
        Self::modify_item(
            id,
            &mut IamAccountModifyReq {
                name: None,
                scope_level: None,
                disabled: None,
                icon: None,
                status: None,
                is_auto: Some(false),
                lock_status: Some(IamAccountLockStateKind::Unlocked),
                temporary: None,
            },
            funs,
            ctx,
        )
        .await?;
        Ok(Void {})
    }

    /// return true means account is global account
    pub async fn is_global_account(account_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<bool> {
        if let Some(bool_) = funs.cache().hget(format!("{}{}", funs.conf::<IamConfig>().cache_key_account_info_, account_id).as_str(), "is_global").await? {
            let bool_ = bool_.parse::<bool>();
            Ok(bool_.unwrap_or(false))
        } else {
            let account = IamAccountServ::get_item(
                account_id,
                &IamAccountFilterReq {
                    basic: RbumBasicFilterReq {
                        ignore_scope: true,
                        with_sub_own_paths: true,
                        own_paths: Some("".to_string()),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                funs,
                ctx,
            )
            .await?;
            let is_global = account.own_paths.is_empty();
            funs.cache()
                .hset(
                    format!("{}{}", funs.conf::<IamConfig>().cache_key_account_info_, account_id).as_str(),
                    "is_global",
                    &is_global.to_string(),
                )
                .await?;
            Ok(is_global)
        }
    }

    pub async fn is_global_account_context(account_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<TardisContext> {
        if Self::is_global_account(account_id, funs, ctx).await? {
            let mut result = ctx.clone();
            result.own_paths = "".to_string();
            Ok(result)
        } else {
            Ok(ctx.clone())
        }
    }

    pub async fn new_context_if_account_is_global(ctx: &TardisContext, funs: &TardisFunsInst) -> TardisResult<TardisContext> {
        if Self::is_global_account(&ctx.owner, funs, ctx).await? {
            let mut result = ctx.clone();
            result.own_paths = "".to_string();
            Ok(result)
        } else {
            Ok(ctx.clone())
        }
    }

    async fn add_or_modify_account_kv(account_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let account = Self::get_item(
            account_id,
            &IamAccountFilterReq {
                basic: RbumBasicFilterReq {
                    ignore_scope: true,
                    own_paths: Some("".to_owned()),
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?;
        SpiKvClient::add_or_modify_key_name(
            &format!("{}:{account_id}", funs.conf::<IamConfig>().spi.kv_account_prefix.clone()),
            &account.name,
            funs,
            ctx,
        )
        .await?;

        Ok(())
    }
}
