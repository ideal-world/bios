use async_trait::async_trait;
use bios_basic::helper::request_helper::get_remote_ip;
use bios_basic::process::task_processor::TaskProcessor;
use tardis::basic::dto::TardisContext;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::db::sea_orm::sea_query::{Expr, SelectStatement};
use tardis::db::sea_orm::EntityName;
use tardis::db::sea_orm::*;
use tardis::log::info;
use tardis::web::web_resp::TardisPage;
use tardis::{TardisFuns, TardisFunsInst};

use bios_basic::rbum::dto::rbum_filer_dto::{RbumBasicFilterReq, RbumRelFilterReq};
use bios_basic::rbum::dto::rbum_item_dto::{RbumItemKernelAddReq, RbumItemKernelModifyReq};
use bios_basic::rbum::dto::rbum_rel_dto::{RbumRelBoneResp, RbumRelCheckReq};
use bios_basic::rbum::helper::rbum_scope_helper;
use bios_basic::rbum::helper::rbum_scope_helper::get_scope_level_by_context;
use bios_basic::rbum::rbum_enumeration::{RbumRelFromKind, RbumScopeLevelKind};
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;
use bios_basic::rbum::serv::rbum_rel_serv::RbumRelServ;

use crate::basic::domain::iam_role;
use crate::basic::dto::iam_filer_dto::IamRoleFilterReq;
use crate::basic::dto::iam_role_dto::{IamRoleAddReq, IamRoleAggAddReq, IamRoleAggModifyReq, IamRoleDetailResp, IamRoleModifyReq, IamRoleSummaryResp};
use crate::basic::serv::iam_account_serv::IamAccountServ;
use crate::basic::serv::iam_key_cache_serv::IamIdentCacheServ;
use crate::basic::serv::iam_rel_serv::IamRelServ;
use crate::iam_config::{IamBasicConfigApi, IamBasicInfoManager, IamConfig};
use crate::iam_constants::{self, RBUM_ITEM_ID_SUB_ROLE_LEN};
use crate::iam_constants::{RBUM_SCOPE_LEVEL_APP, RBUM_SCOPE_LEVEL_TENANT};
use crate::iam_enumeration::{IamRelKind, IamRoleKind};

use super::clients::iam_log_client::{IamLogClient, LogParamTag};
use super::iam_cert_serv::IamCertServ;

pub struct IamRoleServ;

#[async_trait]
impl RbumItemCrudOperation<iam_role::ActiveModel, IamRoleAddReq, IamRoleModifyReq, IamRoleSummaryResp, IamRoleDetailResp, IamRoleFilterReq> for IamRoleServ {
    fn get_ext_table_name() -> &'static str {
        iam_role::Entity.table_name()
    }

    fn get_rbum_kind_id() -> Option<String> {
        Some(IamBasicInfoManager::get_config(|conf| conf.kind_role_id.clone()))
    }

    fn get_rbum_domain_id() -> Option<String> {
        Some(IamBasicInfoManager::get_config(|conf| conf.domain_iam_id.clone()))
    }

    async fn package_item_add(add_req: &IamRoleAddReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<RbumItemKernelAddReq> {
        Ok(RbumItemKernelAddReq {
            id: if add_req.extend_role_id.is_some() {
                Some(TrimString::from(format!(
                    "{}:{}",
                    add_req.extend_role_id.clone().unwrap_or_default(),
                    Self::get_sub_new_id()
                )))
            } else {
                None
            },
            code: add_req.code.clone(),
            name: add_req.name.clone(),
            scope_level: add_req.scope_level.clone(),
            ..Default::default()
        })
    }

    async fn package_ext_add(id: &str, add_req: &IamRoleAddReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<iam_role::ActiveModel> {
        Ok(iam_role::ActiveModel {
            id: Set(id.to_string()),
            icon: Set(add_req.icon.as_ref().unwrap_or(&"".to_string()).to_string()),
            sort: Set(add_req.sort.unwrap_or(0)),
            kind: Set(add_req.kind.as_ref().unwrap_or(&IamRoleKind::Tenant).to_int()),
            in_embed: Set(add_req.in_embed.unwrap_or(false)),
            in_base: Set(add_req.in_base.unwrap_or(false)),
            extend_role_id: Set(add_req.extend_role_id.as_ref().unwrap_or(&"".to_string()).to_string()),
            ..Default::default()
        })
    }

    async fn after_add_item(id: &str, _: &mut IamRoleAddReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let role = Self::do_get_item(
            id,
            &IamRoleFilterReq {
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
        funs.cache()
            .set(
                &format!("{}{}", funs.conf::<IamConfig>().cache_key_role_info_, id),
                TardisFuns::json.obj_to_string(&role)?.as_str(),
            )
            .await?;

        let _ = IamLogClient::add_ctx_task(
            LogParamTag::IamRole,
            Some(id.to_string()),
            "添加自定义角色".to_string(),
            Some("AddCustomizeRole".to_string()),
            ctx,
        )
        .await;

        Ok(())
    }

    async fn package_item_modify(_: &str, modify_req: &IamRoleModifyReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<Option<RbumItemKernelModifyReq>> {
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

    async fn package_ext_modify(id: &str, modify_req: &IamRoleModifyReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<Option<iam_role::ActiveModel>> {
        if modify_req.icon.is_none() && modify_req.sort.is_none() {
            return Ok(None);
        }
        let mut iam_role = iam_role::ActiveModel {
            id: Set(id.to_string()),
            ..Default::default()
        };
        if let Some(icon) = &modify_req.icon {
            iam_role.icon = Set(icon.to_string());
        }
        if let Some(sort) = modify_req.sort {
            iam_role.sort = Set(sort);
        }
        if let Some(kind) = &modify_req.kind {
            iam_role.kind = Set(kind.to_int());
        }
        Ok(Some(iam_role))
    }

    async fn after_modify_item(id: &str, modify_req: &mut IamRoleModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let role = Self::do_get_item(
            id,
            &IamRoleFilterReq {
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
        funs.cache()
            .set(
                &format!("{}{}", funs.conf::<IamConfig>().cache_key_role_info_, id),
                TardisFuns::json.obj_to_string(&role)?.as_str(),
            )
            .await?;
        let role_id = id.to_string();
        let ctx_clone = ctx.clone();
        if modify_req.disabled.unwrap_or(false) {
            TaskProcessor::execute_task_with_ctx(
                &funs.conf::<IamConfig>().cache_key_async_task_status,
                |_task_id| async move {
                    let funs = iam_constants::get_tardis_inst();
                    let mut count = IamRoleServ::count_rel_accounts(&role_id, &funs, &ctx_clone).await.unwrap_or_default() as isize;
                    let mut page_number = 1;
                    while count > 0 {
                        let mut ids = Vec::new();
                        if let Ok(page) = IamRoleServ::paginate_id_rel_accounts(&role_id, page_number, 100, None, None, &funs, &ctx_clone).await {
                            ids = page.records;
                        }
                        for id in ids {
                            IamIdentCacheServ::delete_tokens_and_contexts_by_account_id(&id, get_remote_ip(&ctx_clone).await?, &funs).await?;
                        }
                        page_number += 1;
                        count -= 100;
                    }
                    Ok(())
                },
                funs,
                ctx,
            )
            .await?;
        }

        let mut op_describe = String::new();
        let mut op_kind = String::new();
        if modify_req.name.is_some() {
            if Self::is_custom_role(role.kind, role.scope_level) {
                op_describe = format!("编辑自定义角色名称为{}", modify_req.name.as_ref().unwrap_or(&TrimString::from("")));
                op_kind = "ModifyCustomizeRoleName".to_string();
            } else {
                op_describe = format!("编辑内置角色名称为{}", modify_req.name.as_ref().unwrap_or(&TrimString::from("")));
                op_kind = "ModifyBuiltRoleName".to_string();
            }
        }

        if !op_describe.is_empty() {
            let _ = IamLogClient::add_ctx_task(LogParamTag::IamRole, Some(id.to_string()), op_describe, Some(op_kind), ctx).await;
        }

        Ok(())
    }

    async fn before_delete_item(id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Option<IamRoleDetailResp>> {
        let item = IamRoleServ::get_item(
            id,
            &IamRoleFilterReq {
                basic: RbumBasicFilterReq {
                    ignore_scope: true,
                    rel_ctx_owner: true,
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?;
        if item.scope_level != RbumScopeLevelKind::Private
            || item.in_embed
            || item.in_base
            || id == funs.iam_basic_role_app_admin_id()
            || id == funs.iam_basic_role_sys_admin_id()
            || id == funs.iam_basic_role_tenant_admin_id()
        {
            return Err(funs.err().conflict(&Self::get_obj_name(), "delete", "role is not private", "409-iam-delete-role-conflict"));
        }
        Ok(None)
    }

    async fn after_delete_item(id: &str, _: &Option<IamRoleDetailResp>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        funs.cache().del(&format!("{}{}", funs.conf::<IamConfig>().cache_key_role_info_, id)).await?;
        let role_id = id.to_string();
        let ctx_clone = ctx.clone();
        // todo 待优化 增加-缓存-角色与用户的关联关系 | 以便解决删除角色后，用户的token和context不会被删除的问题
        // 现代码问题: 删除角色后，角色与用户的关联关系逻辑有冲突，导致用户的token和context不会被删除
        TaskProcessor::execute_task_with_ctx(
            &funs.conf::<IamConfig>().cache_key_async_task_status,
            |_task_id| async move {
                let funs = iam_constants::get_tardis_inst();
                let mut count = IamRoleServ::count_rel_accounts(&role_id, &funs, &ctx_clone).await.unwrap_or_default() as isize;
                let mut page_number = 1;
                while count > 0 {
                    let mut ids = Vec::new();
                    if let Ok(page) = IamRoleServ::paginate_id_rel_accounts(&role_id, page_number, 100, None, None, &funs, &ctx_clone).await {
                        ids = page.records;
                    }
                    for id in ids {
                        IamIdentCacheServ::delete_tokens_and_contexts_by_account_id(&id, get_remote_ip(&ctx_clone).await?, &funs).await?;
                    }
                    page_number += 1;
                    count -= 100;
                }
                Ok(())
            },
            funs,
            ctx,
        )
        .await?;

        let _ = IamLogClient::add_ctx_task(
            LogParamTag::IamRole,
            Some(id.to_string()),
            "删除自定义角色".to_string(),
            Some("DeleteCustomizeRole".to_string()),
            ctx,
        )
        .await;

        Ok(())
    }

    async fn package_ext_query(query: &mut SelectStatement, _: bool, filter: &IamRoleFilterReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<()> {
        query.column((iam_role::Entity, iam_role::Column::Icon));
        query.column((iam_role::Entity, iam_role::Column::Sort));
        query.column((iam_role::Entity, iam_role::Column::Kind));
        query.column((iam_role::Entity, iam_role::Column::InBase));
        query.column((iam_role::Entity, iam_role::Column::InEmbed));
        query.column((iam_role::Entity, iam_role::Column::ExtendRoleId));
        if let Some(kind) = &filter.kind {
            query.and_where(Expr::col(iam_role::Column::Kind).eq(kind.to_int()));
        }
        if let Some(in_embed) = &filter.in_embed {
            query.and_where(Expr::col(iam_role::Column::InEmbed).eq(*in_embed));
        }
        if let Some(in_base) = &filter.in_base {
            query.and_where(Expr::col(iam_role::Column::InBase).eq(*in_base));
        }
        if let Some(extend_role_id) = &filter.extend_role_id {
            query.and_where(Expr::col(iam_role::Column::ExtendRoleId).eq(extend_role_id));
        }
        Ok(())
    }

    async fn get_item(id: &str, filter: &IamRoleFilterReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<IamRoleDetailResp> {
        if let Some(role) = funs.cache().get(&format!("{}{}", funs.conf::<IamConfig>().cache_key_role_info_, id)).await? {
            let role = TardisFuns::json.str_to_obj::<IamRoleDetailResp>(&role)?;
            if rbum_scope_helper::check_scope(&role.own_paths, Some(role.scope_level.to_int()), &filter.basic, ctx) {
                return Ok(role);
            }
        }
        let role = Self::do_get_item(id, filter, funs, ctx).await?;
        funs.cache()
            .set(
                &format!("{}{}", funs.conf::<IamConfig>().cache_key_role_info_, id),
                TardisFuns::json.obj_to_string(&role)?.as_str(),
            )
            .await?;
        Ok(role)
    }
}

impl IamRoleServ {
    pub fn get_sub_new_id() -> String {
        TardisFuns::field.nanoid_len(RBUM_ITEM_ID_SUB_ROLE_LEN as usize)
    }

    pub async fn copy_role_agg(tenant_or_app_id: &str, kind: &IamRoleKind, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let base_roles = Self::find_detail_items(
            &IamRoleFilterReq {
                basic: RbumBasicFilterReq {
                    ignore_scope: true,
                    with_sub_own_paths: false,
                    own_paths: Some("".to_string()),
                    ..Default::default()
                },
                kind: Some(kind.clone()),
                in_embed: Some(true),
                ..Default::default()
            },
            None,
            None,
            funs,
            ctx,
        )
        .await?;
        for base_role in base_roles {
            Self::add_role_agg(
                &mut IamRoleAggAddReq {
                    role: IamRoleAddReq {
                        code: Some(TrimString::from(format!("{}:{}", tenant_or_app_id, base_role.code))),
                        name: TrimString::from(base_role.name),
                        icon: Some(base_role.icon),
                        sort: Some(base_role.sort),
                        kind: Some(base_role.kind),
                        scope_level: Some(RbumScopeLevelKind::Private),
                        in_embed: Some(base_role.in_embed),
                        extend_role_id: Some(base_role.id),
                        disabled: Some(base_role.disabled),
                        in_base: Some(false),
                    },
                    res_ids: None,
                },
                funs,
                ctx,
            )
            .await?;
        }
        Ok(())
    }

    pub async fn get_embed_subrole_id(extend_role_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<String> {
        let scope_level = get_scope_level_by_context(ctx)?;
        info!(
            "【get_embed_subrole_id】 : extend_role_id = {}, scope_level = {}, own_paths = {}",
            extend_role_id, scope_level, ctx.own_paths
        );
        let kind = if scope_level == RBUM_SCOPE_LEVEL_APP { IamRoleKind::App } else { IamRoleKind::Tenant };
        if let Some(base_role) = Self::find_one_item(
            &IamRoleFilterReq {
                kind: Some(kind),
                in_embed: Some(true),
                extend_role_id: Some(extend_role_id.to_string()),
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?
        {
            return Ok(base_role.id);
        }
        Err(funs.err().not_found(&Self::get_obj_name(), "get_embed_subrole_id", "role not found", "404-iam-role-not-found"))
    }

    pub async fn add_role_agg(add_req: &mut IamRoleAggAddReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<String> {
        let role_id = Self::add_item(&mut add_req.role, funs, ctx).await?;
        if let Some(res_ids) = &add_req.res_ids {
            for res_id in res_ids {
                Self::add_rel_res(&role_id, res_id, funs, ctx).await?;
            }
        }
        Ok(role_id)
    }

    pub async fn modify_role_agg(id: &str, modify_req: &mut IamRoleAggModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        Self::modify_item(id, &mut modify_req.role, funs, ctx).await?;
        if let Some(input_res_ids) = &modify_req.res_ids {
            let stored_res = Self::find_simple_rel_res(id, None, None, funs, ctx).await?;
            let stored_res_ids: Vec<String> = stored_res.into_iter().map(|x| x.rel_id).collect();
            for input_res_id in input_res_ids {
                if !stored_res_ids.contains(input_res_id) {
                    Self::add_rel_res(id, input_res_id, funs, ctx).await?;
                }
            }
            for stored_res_id in stored_res_ids {
                if !input_res_ids.contains(&stored_res_id) {
                    Self::delete_rel_res(id, &stored_res_id, funs, ctx).await?;
                }
            }

            let role = Self::do_get_item(
                id,
                &IamRoleFilterReq {
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

            let (op_describe, op_kind) = if Self::is_custom_role(role.kind, role.scope_level) {
                ("编辑自定义角色权限".to_string(), "ModifyCustomizeRolePermissions".to_string())
            } else {
                ("编辑内置角色权限".to_string(), "ModifyBuiltRolePermissions".to_string())
            };
            let _ = IamLogClient::add_ctx_task(LogParamTag::IamRole, Some(id.to_string()), op_describe, Some(op_kind), ctx).await;
        }
        Ok(())
    }

    pub async fn add_rel_account(role_id: &str, account_id: &str, spec_scope_level: Option<RbumScopeLevelKind>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let scope_level = get_scope_level_by_context(ctx)?;
        let sub_tenant_admin_role_id = match scope_level {
            RBUM_SCOPE_LEVEL_APP => Self::get_embed_subrole_id(&funs.iam_basic_role_tenant_admin_id(), funs, &IamCertServ::use_sys_or_tenant_ctx_unsafe(ctx.clone())?).await?,
            RBUM_SCOPE_LEVEL_TENANT => Self::get_embed_subrole_id(&funs.iam_basic_role_tenant_admin_id(), funs, ctx).await?,
            _ => "".to_string(),
        };
        if scope_level == RBUM_SCOPE_LEVEL_APP
            && (role_id == funs.iam_basic_role_sys_admin_id() || role_id == funs.iam_basic_role_tenant_admin_id() || sub_tenant_admin_role_id == role_id)
            || scope_level == RBUM_SCOPE_LEVEL_TENANT && role_id == funs.iam_basic_role_sys_admin_id()
        {
            return Err(funs.err().conflict(&Self::get_obj_name(), "add_rel_account", "associated role is invalid", "409-iam-role-rel-conflict"));
        }
        if let Some(spec_scope_level) = spec_scope_level {
            let role = Self::peek_item(role_id, &IamRoleFilterReq::default(), funs, ctx).await?;
            // The role is not private and current scope
            if role.scope_level != RbumScopeLevelKind::Private && role.scope_level.to_int() < spec_scope_level.to_int() {
                return Err(funs.err().conflict(&Self::get_obj_name(), "add_rel_account", "associated role is invalid", "409-iam-role-rel-conflict"));
            }
        }
        match Self::get_embed_subrole_id(role_id, funs, ctx).await {
            Ok(sub_role_id) => {
                IamRelServ::add_simple_rel(&IamRelKind::IamAccountRole, account_id, &sub_role_id, None, None, false, false, funs, ctx).await?;
            }
            Err(_) => {
                // TODO only bind the same own_paths roles
                // E.g. sys admin can't bind tenant admin
                IamRelServ::add_simple_rel(&IamRelKind::IamAccountRole, account_id, role_id, None, None, false, false, funs, ctx).await?;
            }
        }
        IamAccountServ::async_add_or_modify_account_search(account_id.to_string(), Box::new(true), "".to_string(), funs, ctx).await?;
        Ok(())
    }

    pub async fn delete_rel_account(role_id: &str, account_id: &str, spec_scope_level: Option<RbumScopeLevelKind>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        if let Some(spec_scope_level) = spec_scope_level {
            let role = Self::peek_item(role_id, &IamRoleFilterReq::default(), funs, ctx).await?;
            // The role is not private and current scope
            if role.scope_level != RbumScopeLevelKind::Private && role.scope_level != spec_scope_level {
                return Err(funs.err().conflict(&Self::get_obj_name(), "delete_rel_account", "associated role is invalid", "409-iam-role-rel-conflict"));
            }
        }
        let scope_level = get_scope_level_by_context(ctx)?;
        let sub_role_id = match scope_level {
            RBUM_SCOPE_LEVEL_APP => Self::get_embed_subrole_id(&funs.iam_basic_role_app_admin_id(), funs, ctx).await?,
            RBUM_SCOPE_LEVEL_TENANT => Self::get_embed_subrole_id(&funs.iam_basic_role_tenant_admin_id(), funs, ctx).await?,
            _ => "".to_string(),
        };
        if funs.iam_basic_role_sys_admin_id() == role_id
            || funs.iam_basic_role_tenant_admin_id() == role_id
            || funs.iam_basic_role_app_admin_id() == role_id
            || sub_role_id == role_id
        {
            let count = IamRelServ::count_to_rels(&IamRelKind::IamAccountRole, role_id, funs, ctx).await?;
            if count == 1 {
                return Err(funs.err().conflict(
                    &Self::get_obj_name(),
                    "delete_rel_account",
                    "the current role has only one user and cannot be deleted",
                    "409-iam-role-del-only-one-user",
                ));
            }
        }
        match Self::get_embed_subrole_id(role_id, funs, ctx).await {
            Ok(sub_role_id) => {
                IamRelServ::delete_simple_rel(&IamRelKind::IamAccountRole, account_id, &sub_role_id, funs, ctx).await?;
            }
            Err(_) => {
                IamRelServ::delete_simple_rel(&IamRelKind::IamAccountRole, account_id, role_id, funs, ctx).await?;
            }
        }
        IamAccountServ::async_add_or_modify_account_search(account_id.to_string(), Box::new(true), "".to_string(), funs, ctx).await?;
        Ok(())
    }

    pub async fn count_rel_accounts(role_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<u64> {
        IamRelServ::count_to_rels(&IamRelKind::IamAccountRole, role_id, funs, ctx).await
    }

    pub async fn find_id_rel_accounts(
        role_id: &str,
        desc_by_create: Option<bool>,
        desc_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<Vec<String>> {
        IamRelServ::find_to_id_rels(&IamRelKind::IamAccountRole, role_id, desc_by_create, desc_by_update, funs, ctx).await
    }

    pub async fn find_simple_rel_accounts(
        role_id: &str,
        desc_by_create: Option<bool>,
        desc_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<Vec<RbumRelBoneResp>> {
        IamRelServ::find_to_simple_rels(&IamRelKind::IamAccountRole, role_id, desc_by_create, desc_by_update, funs, ctx).await
    }

    pub async fn paginate_id_rel_accounts(
        role_id: &str,
        page_number: u32,
        page_size: u32,
        desc_by_create: Option<bool>,
        desc_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<TardisPage<String>> {
        IamRelServ::paginate_to_id_rels(&IamRelKind::IamAccountRole, role_id, page_number, page_size, desc_by_create, desc_by_update, funs, ctx).await
    }

    pub async fn paginate_simple_rel_accounts(
        role_id: &str,
        page_number: u32,
        page_size: u32,
        desc_by_create: Option<bool>,
        desc_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<TardisPage<RbumRelBoneResp>> {
        IamRelServ::paginate_to_simple_rels(&IamRelKind::IamAccountRole, role_id, page_number, page_size, desc_by_create, desc_by_update, funs, ctx).await
    }

    pub async fn add_rel_res(role_id: &str, res_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        IamRelServ::add_simple_rel(&IamRelKind::IamResRole, res_id, role_id, None, None, false, false, funs, ctx).await?;
        Ok(())
    }

    pub async fn delete_rel_res(role_id: &str, res_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        IamRelServ::delete_simple_rel(&IamRelKind::IamResRole, res_id, role_id, funs, ctx).await?;
        Ok(())
    }

    pub async fn count_rel_res(role_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<u64> {
        IamRelServ::count_to_rels(&IamRelKind::IamResRole, role_id, funs, ctx).await
    }

    pub async fn find_id_rel_res(
        role_id: &str,
        desc_by_create: Option<bool>,
        desc_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<Vec<String>> {
        let res_ids = IamRelServ::find_to_id_rels(&IamRelKind::IamResRole, role_id, desc_by_create, desc_by_update, funs, ctx).await?;
        let role = Self::get_item(
            role_id,
            &IamRoleFilterReq {
                basic: RbumBasicFilterReq {
                    own_paths: Some("".to_string()),
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                ..Default::default()
            },
            &funs,
            &ctx,
        )
        .await?;
        if role.extend_role_id != "" {
            let extend_res_ids = IamRelServ::find_to_id_rels(&IamRelKind::IamResRole, &role.extend_role_id, desc_by_create, desc_by_update, funs, ctx).await?;
            Ok(vec![res_ids, extend_res_ids].concat())
        } else {
            Ok(res_ids)
        }
    }

    pub async fn find_simple_rel_res(
        role_id: &str,
        desc_by_create: Option<bool>,
        desc_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<Vec<RbumRelBoneResp>> {
        let res = IamRelServ::find_to_simple_rels(&IamRelKind::IamResRole, role_id, desc_by_create, desc_by_update, funs, ctx).await?;
        let role = Self::get_item(
            role_id,
            &IamRoleFilterReq {
                basic: RbumBasicFilterReq {
                    own_paths: Some("".to_string()),
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                ..Default::default()
            },
            &funs,
            &ctx,
        )
        .await?;
        if role.extend_role_id != "" {
            let extend_res = IamRelServ::find_to_simple_rels(&IamRelKind::IamResRole, &role.extend_role_id, desc_by_create, desc_by_update, funs, ctx).await?;
            Ok(vec![res, extend_res].concat())
        } else {
            Ok(res)
        }
    }

    pub async fn find_simple_rels(
        role_id: &str,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        from_scope_levels: Option<Vec<i16>>,
        to_scope_levels: Option<Vec<i16>>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<Vec<RbumRelBoneResp>> {
        let res = IamRelServ::find_simple_rels(
            &RbumRelFilterReq {
                basic: RbumBasicFilterReq {
                    own_paths: Some(ctx.own_paths.to_string()),
                    with_sub_own_paths: true,
                    ignore_scope: true,
                    ..Default::default()
                },
                tag: Some(IamRelKind::IamResRole.to_string()),
                to_rbum_item_id: Some(role_id.to_string()),
                from_rbum_scope_levels: from_scope_levels.clone(),
                to_rbum_item_scope_levels: to_scope_levels.clone(),
                ..Default::default()
            },
            desc_sort_by_create,
            desc_sort_by_update,
            false,
            funs,
            ctx,
        )
        .await?;
        let role = Self::get_item(
            role_id,
            &IamRoleFilterReq {
                basic: RbumBasicFilterReq {
                    own_paths: Some("".to_string()),
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                ..Default::default()
            },
            &funs,
            &ctx,
        )
        .await?;
        if role.extend_role_id != "" {
            let extend_res = IamRelServ::find_simple_rels(
                &RbumRelFilterReq {
                    basic: RbumBasicFilterReq {
                        own_paths: Some(ctx.own_paths.to_string()),
                        with_sub_own_paths: true,
                        ignore_scope: true,
                        ..Default::default()
                    },
                    tag: Some(IamRelKind::IamResRole.to_string()),
                    to_rbum_item_id: Some(role.extend_role_id.to_string()),
                    from_rbum_scope_levels: from_scope_levels.clone(),
                    to_rbum_item_scope_levels: to_scope_levels.clone(),
                    ..Default::default()
                },
                desc_sort_by_create,
                desc_sort_by_update,
                false,
                funs,
                ctx,
            )
            .await?;
            Ok(vec![res, extend_res].concat())
        } else {
            Ok(res)
        }
    }

    pub async fn paginate_id_rel_res(
        role_id: &str,
        page_number: u32,
        page_size: u32,
        desc_by_create: Option<bool>,
        desc_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<TardisPage<String>> {
        IamRelServ::paginate_to_id_rels(&IamRelKind::IamResRole, role_id, page_number, page_size, desc_by_create, desc_by_update, funs, ctx).await
    }

    pub async fn paginate_simple_rel_res(
        role_id: &str,
        page_number: u32,
        page_size: u32,
        desc_by_create: Option<bool>,
        desc_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<TardisPage<RbumRelBoneResp>> {
        IamRelServ::paginate_to_simple_rels(&IamRelKind::IamResRole, role_id, page_number, page_size, desc_by_create, desc_by_update, funs, ctx).await
    }

    pub async fn need_sys_admin(funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        Self::need_role(&funs.iam_basic_role_sys_admin_id(), funs, ctx).await
    }

    pub async fn need_tenant_admin(funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        Self::need_role(&funs.iam_basic_role_tenant_admin_id(), funs, ctx).await
    }

    pub async fn need_app_admin(funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        Self::need_role(&funs.iam_basic_role_app_admin_id(), funs, ctx).await
    }

    pub async fn need_role(role_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let exist = RbumRelServ::check_rel(
            &mut RbumRelCheckReq {
                tag: IamRelKind::IamAccountRole.to_string(),
                from_rbum_kind: RbumRelFromKind::Item,
                from_rbum_id: ctx.owner.clone(),
                to_rbum_item_id: role_id.to_string(),
                from_attrs: Default::default(),
                to_attrs: Default::default(),
            },
            funs,
            ctx,
        )
        .await?;
        if !exist {
            Err(funs.err().unauthorized(&Self::get_obj_name(), "need_role", "illegal operation", "401-iam-role-illegal"))
        } else {
            Ok(())
        }
    }

    pub fn is_custom_role(kind: IamRoleKind, scope_level: RbumScopeLevelKind) -> bool {
        kind != IamRoleKind::System && scope_level == RbumScopeLevelKind::Private
    }

    pub async fn find_name_by_ids(ids: Vec<String>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Vec<String>> {
        Self::find_items(
            &IamRoleFilterReq {
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
        .map(|r| r.into_iter().map(|r| format!("{},{}", r.id, r.name)).collect())
    }
}
