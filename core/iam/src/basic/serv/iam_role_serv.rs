use async_trait::async_trait;
use bios_basic::process::task_processor::TaskProcessor;
use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::db::sea_orm::sea_query::{Expr, SelectStatement};
use tardis::db::sea_orm::EntityName;
use tardis::db::sea_orm::*;
use tardis::web::web_resp::TardisPage;
use tardis::{TardisFuns, TardisFunsInst};

use bios_basic::rbum::dto::rbum_filer_dto::{RbumBasicFilterReq, RbumRelFilterReq};
use bios_basic::rbum::dto::rbum_item_dto::{RbumItemKernelAddReq, RbumItemModifyReq};
use bios_basic::rbum::dto::rbum_rel_dto::{RbumRelBoneResp, RbumRelCheckReq};
use bios_basic::rbum::helper::rbum_scope_helper;
use bios_basic::rbum::helper::rbum_scope_helper::get_scope_level_by_context;
use bios_basic::rbum::rbum_enumeration::{RbumRelFromKind, RbumScopeLevelKind};
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;
use bios_basic::rbum::serv::rbum_rel_serv::RbumRelServ;

use crate::basic::domain::iam_role;
use crate::basic::dto::iam_account_dto::IamAccountInfoResp;
use crate::basic::dto::iam_filer_dto::{IamAccountFilterReq, IamRoleFilterReq};
use crate::basic::dto::iam_role_dto::{IamRoleAddReq, IamRoleAggAddReq, IamRoleAggModifyReq, IamRoleDetailResp, IamRoleModifyReq, IamRoleSummaryResp};
use crate::basic::serv::iam_key_cache_serv::IamIdentCacheServ;
use crate::basic::serv::iam_rel_serv::IamRelServ;
use crate::iam_config::{IamBasicConfigApi, IamBasicInfoManager, IamConfig};
use crate::iam_constants;
use crate::iam_constants::{RBUM_SCOPE_LEVEL_APP, RBUM_SCOPE_LEVEL_TENANT};
use crate::iam_enumeration::{IamRelKind, IamRoleKind};

use super::iam_account_serv::IamAccountServ;
use super::iam_cert_serv::IamCertServ;

pub struct IamRoleServ;

#[async_trait]
impl RbumItemCrudOperation<iam_role::ActiveModel, IamRoleAddReq, IamRoleModifyReq, IamRoleSummaryResp, IamRoleDetailResp, IamRoleFilterReq> for IamRoleServ {
    fn get_ext_table_name() -> &'static str {
        iam_role::Entity.table_name()
    }

    fn get_rbum_kind_id() -> String {
        IamBasicInfoManager::get_config(|conf| conf.kind_role_id.clone())
    }

    fn get_rbum_domain_id() -> String {
        IamBasicInfoManager::get_config(|conf| conf.domain_iam_id.clone())
    }

    async fn package_item_add(add_req: &IamRoleAddReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<RbumItemKernelAddReq> {
        Ok(RbumItemKernelAddReq {
            id: None,
            code: Some(add_req.code.clone()),
            name: add_req.name.clone(),
            disabled: None,
            scope_level: add_req.scope_level.clone(),
        })
    }

    async fn package_ext_add(id: &str, add_req: &IamRoleAddReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<iam_role::ActiveModel> {
        Ok(iam_role::ActiveModel {
            id: Set(id.to_string()),
            icon: Set(add_req.icon.as_ref().unwrap_or(&"".to_string()).to_string()),
            sort: Set(add_req.sort.unwrap_or(0)),
            kind: Set(add_req.kind.as_ref().unwrap_or(&IamRoleKind::Tenant).to_int()),
            ..Default::default()
        })
    }

    async fn after_add_item(id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
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
        Ok(())
    }

    async fn package_item_modify(_: &str, modify_req: &IamRoleModifyReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<Option<RbumItemModifyReq>> {
        if modify_req.name.is_none() && modify_req.scope_level.is_none() && modify_req.disabled.is_none() {
            return Ok(None);
        }
        Ok(Some(RbumItemModifyReq {
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
                || async move {
                    let funs = iam_constants::get_tardis_inst();
                    let mut count = IamRoleServ::count_rel_accounts(&role_id, &funs, &ctx_clone).await.unwrap() as isize;
                    let mut page_number = 1;
                    while count > 0 {
                        let ids = IamRoleServ::paginate_id_rel_accounts(&role_id, page_number, 100, None, None, &funs, &ctx_clone).await.unwrap().records;
                        for id in ids {
                            IamIdentCacheServ::delete_tokens_and_contexts_by_account_id(&id, &funs).await.unwrap();
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
        Ok(())
    }

    async fn after_delete_item(id: &str, _: &Option<IamRoleDetailResp>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        funs.cache().del(&format!("{}{}", funs.conf::<IamConfig>().cache_key_role_info_, id)).await?;
        let role_id = id.to_string();
        let ctx_clone = ctx.clone();
        TaskProcessor::execute_task_with_ctx(
            &funs.conf::<IamConfig>().cache_key_async_task_status,
            || async move {
                let funs = iam_constants::get_tardis_inst();
                let mut count = IamRoleServ::count_rel_accounts(&role_id, &funs, &ctx_clone).await.unwrap() as isize;
                let mut page_number = 1;
                while count > 0 {
                    let ids = IamRoleServ::paginate_id_rel_accounts(&role_id, page_number, 100, None, None, &funs, &ctx_clone).await.unwrap().records;
                    for id in ids {
                        IamIdentCacheServ::delete_tokens_and_contexts_by_account_id(&id, &funs).await.unwrap();
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
        Ok(())
    }

    async fn package_ext_query(query: &mut SelectStatement, _: bool, filter: &IamRoleFilterReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<()> {
        query.column((iam_role::Entity, iam_role::Column::Icon));
        query.column((iam_role::Entity, iam_role::Column::Sort));
        query.column((iam_role::Entity, iam_role::Column::Kind));
        if let Some(kind) = &filter.kind {
            query.and_where(Expr::col(iam_role::Column::Kind).eq(kind.to_int()));
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
        }
        Ok(())
    }

    pub async fn add_rel_account(role_id: &str, account_id: &str, spec_scope_level: Option<RbumScopeLevelKind>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let scope_level = get_scope_level_by_context(ctx)?;
        if scope_level == RBUM_SCOPE_LEVEL_APP && (role_id == funs.iam_basic_role_sys_admin_id() || role_id == funs.iam_basic_role_tenant_admin_id())
            || scope_level == RBUM_SCOPE_LEVEL_TENANT && role_id == funs.iam_basic_role_sys_admin_id()
        {
            return Err(funs.err().conflict(&Self::get_obj_name(), "add_rel_account", "associated role is invalid", "409-iam-role-rel-conflict"));
        }
        if let Some(spec_scope_level) = spec_scope_level {
            let role = Self::peek_item(role_id, &IamRoleFilterReq::default(), funs, ctx).await?;
            if role.scope_level.to_int() < spec_scope_level.to_int() {
                return Err(funs.err().conflict(&Self::get_obj_name(), "add_rel_account", "associated role is invalid", "409-iam-role-rel-conflict"));
            }
        }
        // TODO only bind the same own_paths roles
        // E.g. sys admin can't bind tenant admin
        IamRelServ::add_simple_rel(&IamRelKind::IamAccountRole, account_id, role_id, None, None, false, false, funs, ctx).await?;

        // TODO reset account cache
        let tenant_ctx = IamCertServ::use_sys_or_tenant_ctx_unsafe(ctx.clone())?;
        IamCertServ::package_tardis_account_context_and_resp(account_id, &tenant_ctx.own_paths, "".to_string(), None, funs, &tenant_ctx).await?;
        Ok(())
    }

    pub async fn delete_rel_account(role_id: &str, account_id: &str, spec_scope_level: Option<RbumScopeLevelKind>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        if let Some(spec_scope_level) = spec_scope_level {
            let role = Self::peek_item(role_id, &IamRoleFilterReq::default(), funs, ctx).await?;
            if role.scope_level != spec_scope_level {
                return Err(funs.err().conflict(&Self::get_obj_name(), "delete_rel_account", "associated role is invalid", "409-iam-role-rel-conflict"));
            }
        }
        if funs.iam_basic_role_sys_admin_id() == role_id || funs.iam_basic_role_tenant_admin_id() == role_id || funs.iam_basic_role_app_admin_id() == role_id {
            let count = IamRelServ::count_to_rels(&IamRelKind::IamAccountRole, role_id, funs, ctx).await?;
            if count == 1 {
                return Err(funs.err().conflict(&Self::get_obj_name(), "delete_rel_account", "associated role is invalid", "409-iam-role-rel-conflict"));
            }
        }
        IamRelServ::delete_simple_rel(&IamRelKind::IamAccountRole, account_id, role_id, funs, ctx).await?;

        // TODO reset account cache
        let tenant_ctx = IamCertServ::use_sys_or_tenant_ctx_unsafe(ctx.clone())?;
        IamCertServ::package_tardis_account_context_and_resp(account_id, &tenant_ctx.own_paths, "".to_string(), None, funs, &tenant_ctx).await?;
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
        page_number: u64,
        page_size: u64,
        desc_by_create: Option<bool>,
        desc_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<TardisPage<String>> {
        IamRelServ::paginate_to_id_rels(&IamRelKind::IamAccountRole, role_id, page_number, page_size, desc_by_create, desc_by_update, funs, ctx).await
    }

    pub async fn paginate_simple_rel_accounts(
        role_id: &str,
        page_number: u64,
        page_size: u64,
        desc_by_create: Option<bool>,
        desc_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<TardisPage<RbumRelBoneResp>> {
        IamRelServ::paginate_to_simple_rels(&IamRelKind::IamAccountRole, role_id, page_number, page_size, desc_by_create, desc_by_update, funs, ctx).await
    }

    pub async fn add_rel_res(role_id: &str, res_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        IamRelServ::add_simple_rel(&IamRelKind::IamResRole, res_id, role_id, None, None, false, false, funs, ctx).await
    }

    pub async fn delete_rel_res(role_id: &str, res_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        IamRelServ::delete_simple_rel(&IamRelKind::IamResRole, res_id, role_id, funs, ctx).await
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
        IamRelServ::find_to_id_rels(&IamRelKind::IamResRole, role_id, desc_by_create, desc_by_update, funs, ctx).await
    }

    pub async fn find_simple_rel_res(
        role_id: &str,
        desc_by_create: Option<bool>,
        desc_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<Vec<RbumRelBoneResp>> {
        IamRelServ::find_to_simple_rels(&IamRelKind::IamResRole, role_id, desc_by_create, desc_by_update, funs, ctx).await
    }

    pub async fn find_simple_rels(
        role_id: &str,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        scope_levels: Option<Vec<u8>>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<Vec<RbumRelBoneResp>> {
        IamRelServ::find_simple_rels(
            &RbumRelFilterReq {
                basic: RbumBasicFilterReq {
                    own_paths: Some(ctx.own_paths.to_string()),
                    with_sub_own_paths: true,
                    ignore_scope: true,
                    ..Default::default()
                },
                tag: Some(IamRelKind::IamResRole.to_string()),
                to_rbum_item_id: Some(role_id.to_string()),
                from_rbum_scope_levels: scope_levels.clone(),
                to_rbum_item_scope_levels: scope_levels.clone(),
                ..Default::default()
            },
            desc_sort_by_create,
            desc_sort_by_update,
            false,
            funs,
            ctx,
        )
        .await
    }

    pub async fn paginate_id_rel_res(
        role_id: &str,
        page_number: u64,
        page_size: u64,
        desc_by_create: Option<bool>,
        desc_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<TardisPage<String>> {
        IamRelServ::paginate_to_id_rels(&IamRelKind::IamResRole, role_id, page_number, page_size, desc_by_create, desc_by_update, funs, ctx).await
    }

    pub async fn paginate_simple_rel_res(
        role_id: &str,
        page_number: u64,
        page_size: u64,
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
}
