use std::collections::HashSet;

use async_trait::async_trait;
use bios_basic::rbum::dto::rbum_rel_dto::RbumRelBoneResp;
use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;
use bios_basic::rbum::serv::rbum_rel_serv::RbumRelServ;
use bios_basic::rbum::serv::rbum_set_serv::RbumSetItemServ;
use tardis::basic::dto::TardisContext;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::db::sea_orm::sea_query::{Alias, Expr, SelectStatement};
use tardis::db::sea_orm::*;
use tardis::futures_util::future::join_all;
use tardis::{TardisFuns, TardisFunsInst};

use bios_basic::rbum::dto::rbum_filer_dto::{RbumBasicFilterReq, RbumItemRelFilterReq, RbumSetItemFilterReq};
use bios_basic::rbum::dto::rbum_item_dto::{RbumItemKernelAddReq, RbumItemKernelModifyReq, RbumItemTransferOwnershipReq};
use bios_basic::rbum::helper::rbum_scope_helper;
use bios_basic::rbum::rbum_enumeration::RbumRelFromKind;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;

use crate::basic::domain::iam_app;
use crate::basic::dto::iam_app_dto::{
    IamAppAddReq, IamAppAggAddReq, IamAppAggModifyReq, IamAppDetailResp, IamAppKind, IamAppModifyReq, IamAppSummaryResp, IamAppTransferOwnershipReq,
};
use crate::basic::dto::iam_filer_dto::IamAppFilterReq;
use crate::basic::dto::iam_set_dto::IamSetItemAddReq;
use crate::basic::serv::iam_cert_serv::IamCertServ;
use crate::basic::serv::iam_key_cache_serv::IamIdentCacheServ;
use crate::basic::serv::iam_rel_serv::IamRelServ;
use crate::basic::serv::iam_role_serv::IamRoleServ;
use crate::basic::serv::iam_set_serv::IamSetServ;
use crate::iam_config::{IamBasicConfigApi, IamBasicInfoManager, IamConfig};
use crate::iam_constants::{self, RBUM_SCOPE_LEVEL_PRIVATE};
use crate::iam_constants::{RBUM_ITEM_ID_APP_LEN, RBUM_SCOPE_LEVEL_APP};
use crate::iam_enumeration::{IamRelKind, IamSetKind};

use super::clients::iam_kv_client::IamKvClient;
use super::clients::iam_search_client::IamSearchClient;
pub struct IamAppServ;

#[async_trait]
impl RbumItemCrudOperation<iam_app::ActiveModel, IamAppAddReq, IamAppModifyReq, IamAppSummaryResp, IamAppDetailResp, IamAppFilterReq> for IamAppServ {
    fn get_ext_table_name() -> &'static str {
        iam_app::Entity.table_name()
    }

    fn get_rbum_kind_id() -> Option<String> {
        Some(IamBasicInfoManager::get_config(|conf| conf.kind_app_id.clone()))
    }

    fn get_rbum_domain_id() -> Option<String> {
        Some(IamBasicInfoManager::get_config(|conf| conf.domain_iam_id.clone()))
    }

    async fn package_item_add(add_req: &IamAppAddReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<RbumItemKernelAddReq> {
        Ok(RbumItemKernelAddReq {
            id: add_req.id.clone(),
            name: add_req.name.clone(),
            disabled: add_req.disabled,
            scope_level: add_req.scope_level.clone(),
            ..Default::default()
        })
    }

    async fn package_ext_add(id: &str, add_req: &IamAppAddReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<iam_app::ActiveModel> {
        Ok(iam_app::ActiveModel {
            id: Set(id.to_string()),
            icon: Set(add_req.icon.as_ref().unwrap_or(&"".to_string()).to_string()),
            description: Set(add_req.description.clone()),
            sort: Set(add_req.sort.unwrap_or(0)),
            contact_phone: Set(add_req.contact_phone.as_ref().unwrap_or(&"".to_string()).to_string()),
            kind: Set(add_req.kind.clone().unwrap_or(IamAppKind::Product)),
            ..Default::default()
        })
    }

    async fn package_item_modify(_: &str, modify_req: &IamAppModifyReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<Option<RbumItemKernelModifyReq>> {
        if modify_req.name.is_none() && modify_req.scope_level.is_none() && modify_req.disabled.is_none() {
            return Ok(None);
        }
        Ok(Some(RbumItemKernelModifyReq {
            code: None,
            name: modify_req.name.clone(),
            scope_level: modify_req.scope_level.clone(),
            disabled: modify_req.disabled,
        }))
    }

    async fn package_ext_modify(id: &str, modify_req: &IamAppModifyReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<Option<iam_app::ActiveModel>> {
        if modify_req.icon.is_none() && modify_req.sort.is_none() && modify_req.contact_phone.is_none() && modify_req.description.is_none() {
            return Ok(None);
        }
        let mut iam_app = iam_app::ActiveModel {
            id: Set(id.to_string()),
            ..Default::default()
        };
        if let Some(icon) = &modify_req.icon {
            iam_app.icon = Set(icon.to_string());
        }
        if let Some(sort) = modify_req.sort {
            iam_app.sort = Set(sort);
        }
        if let Some(contact_phone) = &modify_req.contact_phone {
            iam_app.contact_phone = Set(contact_phone.to_string());
        }
        if let Some(description) = &modify_req.description {
            iam_app.description = Set(Some(description.to_string()));
        }
        Ok(Some(iam_app))
    }

    async fn after_add_item(id: &str, _: &mut IamAppAddReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        #[cfg(feature = "spi_kv")]
        Self::add_or_modify_app_kv(id, funs, ctx).await?;
        Ok(())
    }
    async fn after_modify_item(id: &str, modify_req: &mut IamAppModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        if modify_req.disabled.unwrap_or(false) {
            // IamIdentCacheServ::delete_tokens_and_contexts_by_tenant_or_app(id, true, funs, ctx).await?;
            IamIdentCacheServ::refresh_tokens_and_contexts_by_tenant_or_app(id, true, funs, ctx).await?;
        }
        #[cfg(feature = "spi_kv")]
        Self::add_or_modify_app_kv(id, funs, ctx).await?;
        Ok(())
    }

    async fn before_delete_item(_: &str, funs: &TardisFunsInst, _: &TardisContext) -> TardisResult<Option<IamAppDetailResp>> {
        Err(funs.err().conflict(&Self::get_obj_name(), "delete", "app can only be disabled but not deleted", "409-iam-app-can-not-delete"))
    }

    async fn package_ext_query(query: &mut SelectStatement, _: bool, filter: &IamAppFilterReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<()> {
        query.column((iam_app::Entity, iam_app::Column::ContactPhone));
        query.column((iam_app::Entity, iam_app::Column::Icon));
        query.column((iam_app::Entity, iam_app::Column::Description));
        query.column((iam_app::Entity, iam_app::Column::Sort));
        query.column((iam_app::Entity, iam_app::Column::Kind));
        if let Some(contact_phone) = &filter.contact_phone {
            query.and_where(Expr::col(iam_app::Column::ContactPhone).eq(contact_phone.as_str()));
        }
        if let Some(kind) = &filter.kind {
            query.and_where(Expr::col(iam_app::Column::Kind).eq(kind.to_string()));
        }
        if let Some(set_rel) = &filter.set_rel {
            Self::package_set_rel(query, Alias::new("rbum_set_rel"), set_rel);
        }
        Ok(())
    }
}

impl IamAppServ {
    pub fn get_new_id() -> String {
        TardisFuns::field.nanoid_len(RBUM_ITEM_ID_APP_LEN as usize)
    }

    pub fn is_app_level_by_ctx(ctx: &TardisContext) -> bool {
        rbum_scope_helper::get_scope_level_by_context(ctx).unwrap_or(RBUM_SCOPE_LEVEL_PRIVATE) == RBUM_SCOPE_LEVEL_APP
    }

    pub fn get_id_by_ctx(ctx: &TardisContext, funs: &TardisFunsInst) -> TardisResult<String> {
        if let Some(id) = rbum_scope_helper::get_path_item(RBUM_SCOPE_LEVEL_APP.to_int(), &ctx.own_paths) {
            Ok(id)
        } else {
            Err(funs.err().unauthorized(
                &Self::get_obj_name(),
                "get_id",
                &format!("app id not found in tardis content {}", ctx.own_paths),
                "401-iam-app-context-not-exist",
            ))
        }
    }

    pub async fn add_app_agg(add_req: &IamAppAggAddReq, funs: &TardisFunsInst, tenant_ctx: &TardisContext) -> TardisResult<String> {
        let app_id = Self::get_new_id();
        let app_ctx = TardisContext {
            own_paths: format!("{}/{}", tenant_ctx.own_paths, app_id),
            ak: "".to_string(),
            roles: vec![],
            groups: vec![],
            owner: tenant_ctx.owner.to_string(),
            ..Default::default()
        };
        Self::add_item(
            &mut IamAppAddReq {
                id: Some(TrimString(app_id.clone())),
                name: add_req.app_name.clone(),
                description: add_req.app_description.clone(),
                icon: add_req.app_icon.clone(),
                sort: add_req.app_sort,
                contact_phone: add_req.app_contact_phone.clone(),
                disabled: add_req.disabled,
                scope_level: Some(iam_constants::RBUM_SCOPE_LEVEL_TENANT),
                kind: add_req.kind.clone(),
                sync_apps_group: add_req.sync_apps_group,
            },
            funs,
            &app_ctx,
        )
        .await?;
        IamRoleServ::add_app_copy_role_agg(&app_id, funs, &app_ctx).await?;
        let app_admin_role_id = IamRoleServ::get_embed_sub_role_id(&funs.iam_basic_role_app_admin_id(), funs, &app_ctx).await?;
        // TODO 是否需要在这里初始化应用级别的set？
        IamSetServ::init_set(IamSetKind::Org, RBUM_SCOPE_LEVEL_APP, funs, &app_ctx).await?;
        IamSetServ::init_set(IamSetKind::Apps, RBUM_SCOPE_LEVEL_APP, funs, &app_ctx).await?;
        if let Some(admin_ids) = &add_req.admin_ids {
            for admin_id in admin_ids {
                IamAppServ::add_rel_account(&app_id, admin_id, false, funs, &app_ctx).await?;
                IamRoleServ::add_rel_account(&app_admin_role_id, admin_id, None, funs, &app_ctx).await?;
            }
        }
        //refresh ctx
        let ctx = IamCertServ::use_sys_or_tenant_ctx_unsafe(tenant_ctx.clone())?;
        let _ = IamCertServ::package_tardis_account_context_and_resp(&tenant_ctx.owner, &ctx.own_paths, "".to_string(), None, funs, &ctx).await;

        if add_req.sync_apps_group.unwrap_or(true) {
            let apps_set_id = IamSetServ::get_default_set_id_by_ctx(&IamSetKind::Apps, funs, &ctx).await?;
            IamSetServ::add_set_item(
                &IamSetItemAddReq {
                    set_id: apps_set_id.clone(),
                    set_cate_id: add_req.set_cate_id.clone().unwrap_or_default(),
                    sort: add_req.app_sort.unwrap_or(0),
                    rel_rbum_item_id: app_id.to_string(),
                },
                funs,
                &ctx,
            )
            .await?;
        }
        app_ctx.execute_task().await?;

        Ok(app_id)
    }

    pub async fn modify_app_agg(id: &str, modify_req: &IamAppAggModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        Self::modify_item(
            id,
            &mut IamAppModifyReq {
                name: modify_req.name.clone(),
                description: modify_req.description.clone(),
                scope_level: modify_req.scope_level.clone(),
                disabled: modify_req.disabled,
                icon: modify_req.icon.clone(),
                sort: modify_req.sort,
                contact_phone: modify_req.contact_phone.clone(),
            },
            funs,
            ctx,
        )
        .await?;
        let app = Self::get_item(
            id,
            &IamAppFilterReq {
                basic: RbumBasicFilterReq {
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
        let app_admin_role_id = IamRoleServ::get_embed_sub_role_id(&funs.iam_basic_role_app_admin_id(), funs, ctx).await?;
        let original_app_admin_account_ids = IamRoleServ::find_id_rel_accounts(&app_admin_role_id, None, None, funs, ctx).await?;
        let original_app_admin_account_ids = HashSet::from_iter(original_app_admin_account_ids.iter().cloned());
        if let Some(admin_ids) = &modify_req.admin_ids {
            if !original_app_admin_account_ids.is_empty() {
                // add new admins
                for admin_id in admin_ids {
                    if !original_app_admin_account_ids.contains(admin_id) {
                        IamAppServ::add_rel_account(id, admin_id, true, funs, ctx).await?;
                        IamRoleServ::add_rel_account(&app_admin_role_id, admin_id, None, funs, ctx).await?;
                    }
                }
                // delete old admins
                for account_id in original_app_admin_account_ids.difference(&admin_ids.iter().cloned().collect::<HashSet<String>>()) {
                    IamRoleServ::delete_rel_account(&app_admin_role_id, account_id, None, funs, ctx).await?;
                }
            }
        }
        if app.kind == IamAppKind::Project {
            let tenant_app_manager_role_id = IamRoleServ::get_embed_sub_role_id(&funs.iam_basic_role_tenant_app_manager_id(), funs, ctx).await?;
            if let Some(admin_ids) = &modify_req.admin_ids {
                if !original_app_admin_account_ids.is_empty() {
                    // add new admins
                    for admin_id in admin_ids {
                        if !original_app_admin_account_ids.contains(admin_id) {
                            IamAppServ::add_rel_account(id, admin_id, true, funs, ctx).await?;
                            IamRoleServ::add_rel_account(&tenant_app_manager_role_id, admin_id, None, funs, ctx).await?;
                        }
                    }
                }
            }
        }
        if modify_req.sync_apps_group.unwrap_or(true) {
            if let Some(set_cate_id) = &modify_req.set_cate_id {
                let tenant_ctx = IamCertServ::use_sys_or_tenant_ctx_unsafe(ctx.clone())?;
                let apps_set_id = IamSetServ::get_default_set_id_by_ctx(&IamSetKind::Apps, funs, &tenant_ctx).await?;
                let set_items = IamSetServ::find_set_items(Some(apps_set_id.clone()), None, Some(id.to_owned()), None, true, Some(true), funs, &tenant_ctx).await?;
                for set_item in set_items {
                    IamSetServ::delete_set_item(&set_item.id, funs, &tenant_ctx).await?;
                }
                IamSetServ::add_set_item(
                    &IamSetItemAddReq {
                        set_id: apps_set_id.clone(),
                        set_cate_id: set_cate_id.to_string(),
                        sort: modify_req.sort.unwrap_or(0),
                        rel_rbum_item_id: id.to_string(),
                    },
                    funs,
                    &tenant_ctx,
                )
                .await?;
            }
        }
        if let Some(disabled) = &modify_req.disabled {
            if *disabled {
                join_all(
                    RbumSetItemServ::find_id_rbums(
                        &RbumSetItemFilterReq {
                            rel_rbum_item_ids: Some(vec![id.to_string()]),
                            ..Default::default()
                        },
                        None,
                        None,
                        funs,
                        ctx,
                    )
                    .await?
                    .into_iter()
                    .map(|set_item_id| async move { RbumSetItemServ::delete_rbum(&set_item_id, funs, ctx).await }),
                )
                .await;
            }
        }
        Ok(())
    }

    pub async fn find_rel_account(app_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Vec<RbumRelBoneResp>> {
        IamRelServ::find_to_simple_rels(&IamRelKind::IamAccountApp, app_id, None, None, funs, ctx).await
    }

    pub async fn add_rel_account(app_id: &str, account_id: &str, ignore_exist_error: bool, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        IamRelServ::add_simple_rel(&IamRelKind::IamAccountApp, account_id, app_id, None, None, ignore_exist_error, false, funs, ctx).await?;
        IamSearchClient::sync_add_or_modify_account_search(account_id, Box::new(true), "", funs, ctx).await?;
        Ok(())
    }

    pub async fn delete_rel_account(app_id: &str, account_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        IamRelServ::delete_simple_rel(&IamRelKind::IamAccountApp, account_id, app_id, funs, ctx).await?;
        let rel_account_roles =
            RbumRelServ::find_from_simple_rels(&IamRelKind::IamAccountRole.to_string(), &RbumRelFromKind::Item, true, account_id, None, None, funs, ctx).await?;
        if rel_account_roles.is_empty() {
            return Ok(());
        }
        for rel in rel_account_roles {
            IamRoleServ::delete_rel_account(&rel.rel_id, account_id, Some(RBUM_SCOPE_LEVEL_APP), funs, ctx).await?;
        }
        IamSearchClient::sync_add_or_modify_account_search(account_id, Box::new(true), "", funs, ctx).await?;
        Ok(())
    }

    pub async fn count_rel_accounts(app_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<u64> {
        IamRelServ::count_to_rels(&IamRelKind::IamAccountApp, app_id, funs, ctx).await
    }

    pub async fn add_rel_tenant_all(app_id: &str, tenant_ids: Vec<String>, ignore_exist_error: bool, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let original_app_tenant_ids = Self::find_id_rel_tenant(&app_id, None, None, funs, ctx).await?;
        let original_app_tenant_ids = HashSet::from_iter(original_app_tenant_ids.iter().cloned());
        for tenant_id in tenant_ids.clone() {
            if original_app_tenant_ids.contains(&tenant_id) {
                continue;
            }
            Self::add_rel_tenant(app_id, &tenant_id, ignore_exist_error, funs, ctx).await?;
        }
        // delete old tenants
        for tenant_id in original_app_tenant_ids.difference(&tenant_ids.iter().cloned().collect::<HashSet<String>>()) {
            Self::delete_rel_tenant(&app_id, tenant_id, funs, ctx).await?;
        }
        Ok(())
    }

    pub async fn add_rel_tenant(app_id: &str, tenant_id: &str, ignore_exist_error: bool, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        IamRelServ::add_simple_rel(&IamRelKind::IamAppTenant, app_id, tenant_id, None, None, ignore_exist_error, false, funs, ctx).await
    }

    pub async fn delete_rel_tenant(app_id: &str, tenant_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        IamRelServ::delete_simple_rel(&IamRelKind::IamAppTenant, app_id, tenant_id, funs, ctx).await
    }

    pub async fn find_id_rel_tenant(
        app_id: &str,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<Vec<String>> {
        IamRelServ::find_from_id_rels(&IamRelKind::IamAppTenant, true, app_id, desc_sort_by_create, desc_sort_by_update, funs, ctx).await
    }

    pub async fn count_rel_tenant(app_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<u64> {
        let mock_ctx = TardisContext {
            own_paths: "".to_string(),
            ..ctx.clone()
        };
        IamRelServ::count_to_rels(&IamRelKind::IamAppTenant, app_id, funs, &mock_ctx).await
    }

    pub fn with_app_rel_filter(ctx: &TardisContext, funs: &TardisFunsInst) -> TardisResult<Option<RbumItemRelFilterReq>> {
        Ok(Some(RbumItemRelFilterReq {
            rel_by_from: true,
            tag: Some(IamRelKind::IamAccountApp.to_string()),
            from_rbum_kind: Some(RbumRelFromKind::Item),
            rel_item_id: Some(Self::get_id_by_ctx(ctx, funs)?),
            // todo 开放人员的 own_paths 限制
            // own_paths: Some(ctx.own_paths.clone()),
            ..Default::default()
        }))
    }

    pub async fn find_name_by_ids(filter: IamAppFilterReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Vec<String>> {
        IamAppServ::find_items(&filter, None, None, funs, ctx).await.map(|r| r.into_iter().map(|r| format!("{},{},{}", r.id, r.name, r.icon)).collect())
    }

    async fn add_or_modify_app_kv(app_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let app = Self::get_item(
            app_id,
            &IamAppFilterReq {
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
        IamKvClient::add_or_modify_key_name(&funs.conf::<IamConfig>().spi.kv_app_prefix.clone(), app_id, &app.name, None, funs, ctx).await?;
        Ok(())
    }

    pub async fn transfer_app_ownership(app_id: &str, transfer_req: &IamAppTransferOwnershipReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        if transfer_req.retain_in_app.unwrap_or(true) == false && transfer_req.retain_admin.unwrap_or(true) != false {
            return Err(funs.err().conflict(
                &Self::get_obj_name(),
                "transfer_app_ownership",
                "retain_in_app is false, but retain_admin is true",
                "409-iam-app-retain-in-app-is-false-but-retain-admin-is-true",
            ));
        }
        let current_owner = &ctx.owner;
        let new_owner = &transfer_req.new_owner;

        // Transfer the app item ownership first
        Self::transfer_item_ownership(
            app_id,
            &RbumItemTransferOwnershipReq {
                new_owner: transfer_req.new_owner.clone(),
                new_own_paths: None,
            },
            funs,
            ctx,
        )
        .await?;

        // Create app context for operations within the app scope
        let new_app_ctx = TardisContext {
            owner: new_owner.to_string(),
            ..ctx.clone()
        };

        // 如果目标所有者不保留其他角色，那么先删除目标所有者的其他的app角色
        if !transfer_req.new_owner_retain_other_roles.unwrap_or(true) {
            let rel_account_roles = RbumRelServ::find_from_simple_rels(
                &IamRelKind::IamAccountRole.to_string(),
                &RbumRelFromKind::Item,
                true,
                new_owner,
                None,
                None,
                funs,
                &new_app_ctx,
            )
            .await?;
            for rel in rel_account_roles {
                IamRoleServ::delete_rel_account(&rel.rel_id, new_owner, Some(RBUM_SCOPE_LEVEL_APP), funs, &new_app_ctx).await?;
            }
        }

        //给新的owner添加app管理员角色
        let app_admin_role_id = IamRoleServ::get_embed_sub_role_id(&funs.iam_basic_role_app_admin_id(), funs, &new_app_ctx).await?;
        IamRoleServ::add_rel_account(&app_admin_role_id, new_owner, None, funs, &new_app_ctx).await?;

        // If the administrator is not retained, delete the administrator role of the old owner
        if !transfer_req.retain_admin.unwrap_or(true) {
            let app_admin_role_id = IamRoleServ::get_embed_sub_role_id(&funs.iam_basic_role_app_admin_id(), funs, ctx).await?;
            IamRoleServ::delete_rel_account(&app_admin_role_id, current_owner, None, funs, ctx).await?;
        }

        // 检查新的owner是否在app中,如果不在就添加
        if !IamRelServ::exist_rels(&IamRelKind::IamAccountApp, new_owner, app_id, funs, &new_app_ctx).await? {
            Self::add_rel_account(app_id, new_owner, true, funs, &new_app_ctx).await?;
        }

        // 如果不要求留在app中,那么删除关联关系
        if !transfer_req.retain_in_app.unwrap_or(true) {
            Self::delete_rel_account(app_id, current_owner, funs, &ctx).await?;
            // 从Apps Set集合中删除原所有者对app的访问权限
            let tenant_ctx = IamCertServ::use_sys_or_tenant_ctx_unsafe(ctx.clone())?;
            let apps_set_id = IamSetServ::get_default_set_id_by_ctx(&IamSetKind::Apps, funs, &tenant_ctx).await?;
            let set_items = IamSetServ::find_set_items(Some(apps_set_id), None, Some(app_id.to_owned()), None, true, Some(true), funs, &tenant_ctx).await?;
            for set_item in set_items {
                // 只删除当前用户相关的set item，通过检查rel_rbum_item_id是否匹配app_id
                if set_item.rel_rbum_item_id == app_id {
                    IamSetServ::delete_set_item(&set_item.id, funs, &tenant_ctx).await?;
                }
            }
        }

        Ok(())
    }
}
