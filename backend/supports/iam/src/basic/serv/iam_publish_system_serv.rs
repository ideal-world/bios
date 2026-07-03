use std::collections::{HashMap, HashSet};

use async_trait::async_trait;
use bios_basic::rbum::dto::rbum_filer_dto::{RbumBasicFilterReq, RbumItemRelFilterReq};
use bios_basic::rbum::dto::rbum_item_dto::{RbumItemKernelAddReq, RbumItemKernelModifyReq};
use bios_basic::rbum::helper::rbum_scope_helper;
use bios_basic::rbum::rbum_enumeration::{RbumRelFromKind, RbumScopeLevelKind};
use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;
use bios_basic::rbum::serv::rbum_item_serv::{RbumItemCrudOperation, RbumItemServ};
use tardis::basic::{dto::TardisContext, field::TrimString, result::TardisResult};
use tardis::db::sea_orm::sea_query::{Expr, SelectStatement};
use tardis::db::sea_orm::*;
use tardis::web::web_resp::TardisPage;
use tardis::TardisFunsInst;

use crate::basic::domain::iam_publish_system;
use crate::basic::dto::iam_filer_dto::IamPublishSystemFilterReq;
use crate::basic::dto::iam_publish_system_dto::{
    IamPublishSystemAddReq, IamPublishSystemDeleteResp, IamPublishSystemDetailResp, IamPublishSystemModifyReq, IamPublishSystemSummaryResp,
};
use crate::basic::serv::clients::iam_search_client::IamSearchClient;
use crate::basic::serv::iam_rel_serv::IamRelServ;
use crate::basic::serv::iam_tenant_serv::IamTenantServ;
use crate::iam_config::IamBasicInfoManager;
use crate::iam_enumeration::IamRelKind;

pub struct IamPublishSystemServ;

#[async_trait]
impl
    RbumItemCrudOperation<
        iam_publish_system::ActiveModel,
        IamPublishSystemAddReq,
        IamPublishSystemModifyReq,
        IamPublishSystemSummaryResp,
        IamPublishSystemDetailResp,
        IamPublishSystemFilterReq,
    > for IamPublishSystemServ
{
    fn get_ext_table_name() -> &'static str {
        iam_publish_system::Entity.table_name()
    }

    fn get_rbum_kind_id() -> Option<String> {
        Some(IamBasicInfoManager::get_config(|conf| conf.kind_publish_system_id.clone()))
    }

    fn get_rbum_domain_id() -> Option<String> {
        Some(IamBasicInfoManager::get_config(|conf| conf.domain_iam_id.clone()))
    }

    async fn before_add_item(add_req: &mut IamPublishSystemAddReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        if add_req.rel_tenant_ids.is_empty() {
            return Err(funs.err().bad_request(&Self::get_obj_name(), "add", "rel_tenant_ids cannot be empty", "400-iam-publish-system-tenant-empty"));
        }
        Self::check_tenants_exist(&add_req.rel_tenant_ids, funs, ctx).await?;
        Self::check_name_duplicate(&add_req.name.to_string(), None, funs, ctx).await?;
        if let Some(sys_ident) = &add_req.sys_ident {
            if !sys_ident.is_empty() {
                Self::check_sys_ident_duplicate(sys_ident, None, funs, ctx).await?;
            }
        }
        Ok(())
    }

    async fn before_modify_item(id: &str, modify_req: &mut IamPublishSystemModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        if let Some(rel_tenant_ids) = &modify_req.rel_tenant_ids {
            if rel_tenant_ids.is_empty() {
                return Err(funs.err().bad_request(&Self::get_obj_name(), "modify", "rel_tenant_ids cannot be empty", "400-iam-publish-system-tenant-empty"));
            }
            Self::check_tenants_exist(rel_tenant_ids, funs, ctx).await?;
            let current_tenant_ids = Self::find_id_rel_tenant(id, None, None, funs, ctx).await?;
            let new_tenant_ids = Self::normalize_tenant_ids(rel_tenant_ids);
            let removed_tenant_ids = Self::removed_tenant_ids(&current_tenant_ids, &new_tenant_ids);
            if !removed_tenant_ids.is_empty() && Self::has_app_rel_in_tenants(id, &removed_tenant_ids, funs, ctx).await? {
                return Err(funs.err().conflict(
                    &Self::get_obj_name(),
                    "modify",
                    "publish system has apps in removed tenants and cannot remove tenant association",
                    "409-iam-publish-system-tenant-change-conflict",
                ));
            }
        }
        if let Some(name) = &modify_req.name {
            Self::check_name_duplicate(&name.to_string(), Some(id), funs, ctx).await?;
        }
        if let Some(sys_ident) = &modify_req.sys_ident {
            if !sys_ident.is_empty() {
                Self::check_sys_ident_duplicate(sys_ident, Some(id), funs, ctx).await?;
            }
        }
        Ok(())
    }

    async fn before_delete_item(id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Option<IamPublishSystemDetailResp>> {
        if Self::count_app_rel(id, funs, ctx).await? > 0 {
            return Err(funs.err().conflict(
                &Self::get_obj_name(),
                "delete",
                "publish system is associated with apps and cannot be deleted",
                "409-iam-publish-system-delete-conflict",
            ));
        }
        Ok(None)
    }

    async fn package_item_add(add_req: &IamPublishSystemAddReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<RbumItemKernelAddReq> {
        Ok(RbumItemKernelAddReq {
            id: add_req.id.clone(),
            name: add_req.name.clone(),
            scope_level: add_req.scope_level.clone(),
            ..Default::default()
        })
    }

    async fn package_ext_add(id: &str, add_req: &IamPublishSystemAddReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<iam_publish_system::ActiveModel> {
        Ok(iam_publish_system::ActiveModel {
            id: Set(id.to_string()),
            sys_ident: Set(add_req.sys_ident.clone().filter(|s| !s.is_empty())),
            description: Set(add_req.description.clone()),
            ..Default::default()
        })
    }

    async fn package_item_modify(_: &str, modify_req: &IamPublishSystemModifyReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<Option<RbumItemKernelModifyReq>> {
        if modify_req.name.is_none() && modify_req.scope_level.is_none() {
            return Ok(None);
        }
        Ok(Some(RbumItemKernelModifyReq {
            code: None,
            name: modify_req.name.clone(),
            scope_level: modify_req.scope_level.clone(),
            disabled: None,
        }))
    }

    async fn package_ext_modify(id: &str, modify_req: &IamPublishSystemModifyReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<Option<iam_publish_system::ActiveModel>> {
        if modify_req.sys_ident.is_none() && modify_req.description.is_none() {
            return Ok(None);
        }
        let mut model = iam_publish_system::ActiveModel {
            id: Set(id.to_string()),
            ..Default::default()
        };
        if modify_req.sys_ident.is_some() {
            model.sys_ident = Set(modify_req.sys_ident.clone().filter(|s| !s.is_empty()));
        }
        if modify_req.description.is_some() {
            model.description = Set(modify_req.description.clone());
        }
        Ok(Some(model))
    }

    async fn after_add_item(id: &str, add_req: &mut IamPublishSystemAddReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        Self::add_rel_tenant_all(id, Self::normalize_tenant_ids(&add_req.rel_tenant_ids), false, funs, ctx).await?;
        IamSearchClient::async_add_or_modify_publish_system_search(id, funs, ctx).await?;
        Ok(())
    }

    async fn after_modify_item(id: &str, modify_req: &mut IamPublishSystemModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        if let Some(rel_tenant_ids) = &modify_req.rel_tenant_ids {
            Self::add_rel_tenant_all(id, Self::normalize_tenant_ids(rel_tenant_ids), false, funs, ctx).await?;
        }
        IamSearchClient::async_add_or_modify_publish_system_search(id, funs, ctx).await?;
        Ok(())
    }

    async fn after_delete_item(id: &str, _: &Option<IamPublishSystemDetailResp>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        IamSearchClient::async_delete_publish_system_search(id.to_string(), funs, ctx.clone()).await?;
        Ok(())
    }

    async fn package_ext_query(query: &mut SelectStatement, _: bool, filter: &IamPublishSystemFilterReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<()> {
        query.column((iam_publish_system::Entity, iam_publish_system::Column::SysIdent));
        query.column((iam_publish_system::Entity, iam_publish_system::Column::Description));
        if let Some(sys_ident) = &filter.sys_ident {
            query.and_where(Expr::col((iam_publish_system::Entity, iam_publish_system::Column::SysIdent)).eq(sys_ident.as_str()));
        }
        Ok(())
    }

    async fn peek_item(id: &str, filter: &IamPublishSystemFilterReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<IamPublishSystemSummaryResp> {
        let mut resp = Self::do_peek_item(id, filter, funs, ctx).await?;
        Self::fill_summary_rel_tenant_ids(&mut resp, funs, ctx).await?;
        Ok(resp)
    }

    async fn get_item(id: &str, filter: &IamPublishSystemFilterReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<IamPublishSystemDetailResp> {
        let mut resp = Self::do_get_item(id, filter, funs, ctx).await?;
        Self::fill_detail_rel_tenant_ids(&mut resp, funs, ctx).await?;
        Ok(resp)
    }

    async fn paginate_items(
        filter: &IamPublishSystemFilterReq,
        page_number: u32,
        page_size: u32,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<TardisPage<IamPublishSystemSummaryResp>> {
        let mut page = Self::do_paginate_items(filter, page_number, page_size, desc_sort_by_create, desc_sort_by_update, funs, ctx).await?;
        Self::fill_summaries_rel_tenant_ids(&mut page.records, funs, ctx).await?;
        Ok(page)
    }

    async fn paginate_detail_items(
        filter: &IamPublishSystemFilterReq,
        page_number: u32,
        page_size: u32,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<TardisPage<IamPublishSystemDetailResp>> {
        let mut page = Self::do_paginate_detail_items(filter, page_number, page_size, desc_sort_by_create, desc_sort_by_update, funs, ctx).await?;
        Self::fill_details_rel_tenant_ids(&mut page.records, funs, ctx).await?;
        Ok(page)
    }

    async fn find_one_item(filter: &IamPublishSystemFilterReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Option<IamPublishSystemSummaryResp>> {
        let mut resp = Self::do_find_one_item(filter, funs, ctx).await?;
        if let Some(item) = resp.as_mut() {
            Self::fill_summary_rel_tenant_ids(item, funs, ctx).await?;
        }
        Ok(resp)
    }

    async fn find_items(
        filter: &IamPublishSystemFilterReq,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<Vec<IamPublishSystemSummaryResp>> {
        let mut items = Self::do_find_items(filter, desc_sort_by_create, desc_sort_by_update, funs, ctx).await?;
        Self::fill_summaries_rel_tenant_ids(&mut items, funs, ctx).await?;
        Ok(items)
    }

    async fn find_one_detail_item(filter: &IamPublishSystemFilterReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Option<IamPublishSystemDetailResp>> {
        let mut resp = Self::do_find_one_detail_item(filter, funs, ctx).await?;
        if let Some(item) = resp.as_mut() {
            Self::fill_detail_rel_tenant_ids(item, funs, ctx).await?;
        }
        Ok(resp)
    }

    async fn find_detail_items(
        filter: &IamPublishSystemFilterReq,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<Vec<IamPublishSystemDetailResp>> {
        let mut items = Self::do_find_detail_items(filter, desc_sort_by_create, desc_sort_by_update, funs, ctx).await?;
        Self::fill_details_rel_tenant_ids(&mut items, funs, ctx).await?;
        Ok(items)
    }
}

impl IamPublishSystemServ {
    fn normalize_tenant_ids(tenant_ids: &[TrimString]) -> Vec<String> {
        tenant_ids.iter().map(|id| id.to_string()).collect()
    }

    fn removed_tenant_ids(current: &[String], new: &[String]) -> Vec<String> {
        let new_set: HashSet<_> = new.iter().cloned().collect();
        current.iter().filter(|id| !new_set.contains(*id)).cloned().collect()
    }

    async fn has_app_rel_in_tenants(publish_system_id: &str, tenant_ids: &[String], funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<bool> {
        if tenant_ids.is_empty() {
            return Ok(false);
        }
        let tenant_set: HashSet<_> = tenant_ids.iter().cloned().collect();
        let global_ctx = TardisContext {
            own_paths: "".to_string(),
            ..ctx.clone()
        };
        let app_ids = IamRelServ::find_to_id_rels(&IamRelKind::IamAppPublishSystem, publish_system_id, None, None, funs, &global_ctx).await?;
        if app_ids.is_empty() {
            return Ok(false);
        }
        let apps = RbumItemServ::find_rbums(
            &RbumBasicFilterReq {
                ids: Some(app_ids),
                own_paths: Some("".to_string()),
                with_sub_own_paths: true,
                ..Default::default()
            },
            None,
            None,
            funs,
            &global_ctx,
        )
        .await?;
        Ok(apps.iter().any(|app| {
            rbum_scope_helper::get_path_item(RbumScopeLevelKind::L1.to_int(), &app.own_paths)
                .is_some_and(|tenant_id| tenant_set.contains(&tenant_id))
        }))
    }

    async fn fill_summary_rel_tenant_ids(resp: &mut IamPublishSystemSummaryResp, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        resp.rel_tenant_ids = Self::find_id_rel_tenant(&resp.id, None, None, funs, ctx).await?;
        Ok(())
    }

    async fn fill_detail_rel_tenant_ids(resp: &mut IamPublishSystemDetailResp, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        resp.rel_tenant_ids = Self::find_id_rel_tenant(&resp.id, None, None, funs, ctx).await?;
        Ok(())
    }

    async fn fill_summaries_rel_tenant_ids(items: &mut [IamPublishSystemSummaryResp], funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        for item in items.iter_mut() {
            Self::fill_summary_rel_tenant_ids(item, funs, ctx).await?;
        }
        Ok(())
    }

    async fn fill_details_rel_tenant_ids(items: &mut [IamPublishSystemDetailResp], funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        for item in items.iter_mut() {
            Self::fill_detail_rel_tenant_ids(item, funs, ctx).await?;
        }
        Ok(())
    }

    pub fn with_tenant_rel_filter(rel_tenant_id: &str) -> RbumItemRelFilterReq {
        RbumItemRelFilterReq {
            rel_by_from: true,
            tag: Some(IamRelKind::IamPublishSystemTenant.to_string()),
            from_rbum_kind: Some(RbumRelFromKind::Item),
            rel_item_id: Some(rel_tenant_id.to_string()),
            ..Default::default()
        }
    }

    pub async fn add_rel_tenant_all(
        publish_system_id: &str,
        tenant_ids: Vec<String>,
        ignore_exist_error: bool,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<()> {
        let original_tenant_ids = Self::find_id_rel_tenant(publish_system_id, None, None, funs, ctx).await?;
        let original_tenant_ids = HashSet::from_iter(original_tenant_ids.iter().cloned());
        for tenant_id in tenant_ids.clone() {
            if original_tenant_ids.contains(&tenant_id) {
                continue;
            }
            Self::add_rel_tenant(publish_system_id, &tenant_id, ignore_exist_error, funs, ctx).await?;
        }
        for tenant_id in original_tenant_ids.difference(&tenant_ids.iter().cloned().collect::<HashSet<String>>()) {
            Self::delete_rel_tenant(publish_system_id, tenant_id, funs, ctx).await?;
        }
        Ok(())
    }

    pub async fn add_rel_tenant(publish_system_id: &str, tenant_id: &str, ignore_exist_error: bool, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        IamRelServ::add_simple_rel(
            &IamRelKind::IamPublishSystemTenant,
            publish_system_id,
            tenant_id,
            None,
            None,
            ignore_exist_error,
            false,
            funs,
            ctx,
        )
        .await
    }

    pub async fn delete_rel_tenant(publish_system_id: &str, tenant_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        IamRelServ::delete_simple_rel(&IamRelKind::IamPublishSystemTenant, publish_system_id, tenant_id, funs, ctx).await
    }

    pub async fn find_id_rel_tenant(
        publish_system_id: &str,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<Vec<String>> {
        let global_ctx = TardisContext {
            own_paths: "".to_string(),
            ..ctx.clone()
        };
        IamRelServ::find_from_id_rels(
            &IamRelKind::IamPublishSystemTenant,
            true,
            publish_system_id,
            desc_sort_by_create,
            desc_sort_by_update,
            funs,
            &global_ctx,
        )
        .await
    }

    /// 平台级系统名称重名校验
    async fn check_name_duplicate(name: &str, exclude_id: Option<&str>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        if let Some(existing) = Self::find_one_item(
            &IamPublishSystemFilterReq {
                basic: RbumBasicFilterReq {
                    name: Some(name.to_string()),
                    own_paths: Some("".to_string()),
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?
        {
            if exclude_id.map(|id| id != existing.id).unwrap_or(true) {
                return Err(funs.err().conflict(&Self::get_obj_name(), "check_name", "name already exists", "409-iam-publish-system-name-exist"));
            }
        }
        Ok(())
    }

    /// 平台级系统标识重名校验
    async fn check_sys_ident_duplicate(sys_ident: &str, exclude_id: Option<&str>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        if let Some(existing) = Self::find_one_item(
            &IamPublishSystemFilterReq {
                basic: RbumBasicFilterReq {
                    own_paths: Some("".to_string()),
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                sys_ident: Some(sys_ident.to_string()),
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?
        {
            if exclude_id.map(|id| id != existing.id).unwrap_or(true) {
                return Err(funs.err().conflict(
                    &Self::get_obj_name(),
                    "check_sys_ident",
                    "sys_ident already exists",
                    "409-iam-publish-system-sys-ident-exist",
                ));
            }
        }
        Ok(())
    }

    async fn check_tenants_exist(tenant_ids: &[TrimString], funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let ids: Vec<String> = tenant_ids.iter().map(|id| id.to_string()).collect();
        if IamTenantServ::count_items(
            &crate::basic::dto::iam_filer_dto::IamTenantFilterReq {
                basic: RbumBasicFilterReq {
                    ids: Some(ids.clone()),
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?
            != ids.len() as u64
        {
            return Err(funs.err().not_found(&Self::get_obj_name(), "check_tenant", "tenant not found", "404-iam-publish-system-tenant-not-exist"));
        }
        Ok(())
    }

    async fn count_app_rel(id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<u64> {
        let global_ctx = TardisContext {
            own_paths: "".to_string(),
            ..ctx.clone()
        };
        IamRelServ::count_to_rels(&IamRelKind::IamAppPublishSystem, id, funs, &global_ctx).await
    }

    async fn find_rel_app_ids_by_tenant(id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<HashMap<String, Vec<String>>> {
        let global_ctx = TardisContext {
            own_paths: "".to_string(),
            ..ctx.clone()
        };
        let app_ids = IamRelServ::find_to_id_rels(&IamRelKind::IamAppPublishSystem, id, None, None, funs, &global_ctx).await?;
        if app_ids.is_empty() {
            return Ok(HashMap::new());
        }
        let apps = RbumItemServ::find_rbums(
            &RbumBasicFilterReq {
                ids: Some(app_ids),
                own_paths: Some("".to_string()),
                with_sub_own_paths: true,
                ..Default::default()
            },
            None,
            None,
            funs,
            &global_ctx,
        )
        .await?;
        let mut rel_app_ids: HashMap<String, Vec<String>> = HashMap::new();
        for app in apps {
            if let Some(tenant_id) = rbum_scope_helper::get_path_item(RbumScopeLevelKind::L1.to_int(), &app.own_paths) {
                rel_app_ids.entry(tenant_id).or_default().push(app.id);
            }
        }
        Ok(rel_app_ids)
    }

    /// 删除发布系统；若存在关联产品则拒绝删除并返回按租户分组的产品 ID
    pub async fn delete_checked(id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<IamPublishSystemDeleteResp> {
        let rel_app_ids = Self::find_rel_app_ids_by_tenant(id, funs, ctx).await?;
        if !rel_app_ids.is_empty() {
            return Ok(IamPublishSystemDeleteResp {
                deleted: false,
                rel_app_ids,
            });
        }
        Self::delete_item_with_all_rels(id, funs, ctx).await?;
        Ok(IamPublishSystemDeleteResp {
            deleted: true,
            rel_app_ids: HashMap::new(),
        })
    }
}
