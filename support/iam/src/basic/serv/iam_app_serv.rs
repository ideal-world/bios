use std::collections::HashSet;

use async_trait::async_trait;
use bios_basic::rbum::dto::rbum_rel_dto::RbumRelBoneResp;
use bios_basic::rbum::serv::rbum_rel_serv::RbumRelServ;
use bios_sdk_invoke::clients::spi_kv_client::SpiKvClient;
use tardis::basic::dto::TardisContext;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::db::sea_orm::sea_query::{Expr, SelectStatement};
use tardis::db::sea_orm::*;
use tardis::{TardisFuns, TardisFunsInst};

use bios_basic::rbum::dto::rbum_filer_dto::{RbumBasicFilterReq, RbumItemRelFilterReq};
use bios_basic::rbum::dto::rbum_item_dto::{RbumItemKernelAddReq, RbumItemKernelModifyReq};
use bios_basic::rbum::helper::rbum_scope_helper;
use bios_basic::rbum::rbum_enumeration::RbumRelFromKind;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;

use crate::basic::domain::iam_app;
use crate::basic::dto::iam_app_dto::{IamAppAddReq, IamAppAggAddReq, IamAppAggModifyReq, IamAppDetailResp, IamAppModifyReq, IamAppSummaryResp};
use crate::basic::dto::iam_filer_dto::IamAppFilterReq;
#[cfg(feature = "spi_kv")]
use crate::basic::serv::iam_cert_serv::IamCertServ;
use crate::basic::serv::iam_key_cache_serv::IamIdentCacheServ;
use crate::basic::serv::iam_rel_serv::IamRelServ;
use crate::basic::serv::iam_role_serv::IamRoleServ;
use crate::basic::serv::iam_set_serv::IamSetServ;
use crate::iam_config::{IamBasicConfigApi, IamBasicInfoManager, IamConfig};
use crate::iam_constants::{self, RBUM_SCOPE_LEVEL_PRIVATE};
use crate::iam_constants::{RBUM_ITEM_ID_APP_LEN, RBUM_SCOPE_LEVEL_APP};
use crate::iam_enumeration::{IamRelKind, IamSetKind};
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
            sort: Set(add_req.sort.unwrap_or(0)),
            contact_phone: Set(add_req.contact_phone.as_ref().unwrap_or(&"".to_string()).to_string()),
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
        if modify_req.icon.is_none() && modify_req.sort.is_none() && modify_req.contact_phone.is_none() {
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
        Ok(Some(iam_app))
    }

    async fn after_add_item(id: &str, _: &mut IamAppAddReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        #[cfg(feature = "spi_kv")]
        Self::add_or_modify_app_kv(id, funs, ctx).await?;
        Ok(())
    }
    async fn after_modify_item(id: &str, modify_req: &mut IamAppModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        if modify_req.disabled.unwrap_or(false) {
            IamIdentCacheServ::delete_tokens_and_contexts_by_tenant_or_app(id, true, funs, ctx).await?;
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
        query.column((iam_app::Entity, iam_app::Column::Sort));
        if let Some(contact_phone) = &filter.contact_phone {
            query.and_where(Expr::col(iam_app::Column::ContactPhone).eq(contact_phone.as_str()));
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
                icon: add_req.app_icon.clone(),
                sort: add_req.app_sort,
                contact_phone: add_req.app_contact_phone.clone(),
                disabled: add_req.disabled,
                scope_level: Some(iam_constants::RBUM_SCOPE_LEVEL_TENANT),
            },
            funs,
            &app_ctx,
        )
        .await?;
        // todo 是否需要在这里初始化应用级别的set？
        IamSetServ::init_set(IamSetKind::Org, RBUM_SCOPE_LEVEL_APP, funs, &app_ctx).await?;
        IamSetServ::init_set(IamSetKind::Apps, RBUM_SCOPE_LEVEL_APP, funs, &app_ctx).await?;
        if let Some(admin_ids) = &add_req.admin_ids {
            for admin_id in admin_ids {
                IamAppServ::add_rel_account(&app_id, admin_id, false, funs, &app_ctx).await?;
                IamRoleServ::add_rel_account(&funs.iam_basic_role_app_admin_id(), admin_id, None, funs, &app_ctx).await?;
            }
        }
        #[cfg(feature = "spi_kv")]
        {
            //refresh ctx
            let ctx = IamCertServ::use_sys_or_tenant_ctx_unsafe(tenant_ctx.clone())?;
            IamCertServ::package_tardis_account_context_and_resp(&tenant_ctx.owner, &ctx.own_paths, "".to_string(), None, funs, &ctx).await?;
        }
        
        Ok(app_id)
    }

    pub async fn modify_app_agg(id: &str, modify_req: &IamAppAggModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let original_app_admin_account_ids = IamRoleServ::find_id_rel_accounts(&funs.iam_basic_role_app_admin_id(), None, None, funs, ctx).await?;
        let original_app_admin_account_ids = HashSet::from_iter(original_app_admin_account_ids.iter().cloned());
        Self::modify_item(
            id,
            &mut IamAppModifyReq {
                name: modify_req.name.clone(),
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
        if let Some(admin_ids) = &modify_req.admin_ids {
            if !original_app_admin_account_ids.is_empty() {
                // add new admins
                for admin_id in admin_ids {
                    if !original_app_admin_account_ids.contains(admin_id) {
                        IamAppServ::add_rel_account(id, admin_id, true, funs, ctx).await?;
                        IamRoleServ::add_rel_account(&funs.iam_basic_role_app_admin_id(), admin_id, None, funs, ctx).await?;
                    }
                }
                // delete old admins
                for account_id in original_app_admin_account_ids.difference(&admin_ids.iter().cloned().collect::<HashSet<String>>()) {
                    IamRoleServ::delete_rel_account(&funs.iam_basic_role_app_admin_id(), account_id, None, funs, ctx).await?;
                    // IamAppServ::delete_rel_account(id, account_id, funs, ctx).await?;
                }
            }
        }
        Ok(())
    }

    pub async fn find_rel_account(app_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Vec<RbumRelBoneResp>> {
        IamRelServ::find_to_simple_rels(&IamRelKind::IamAccountApp, app_id, None, None, funs, ctx).await
    }

    pub async fn add_rel_account(app_id: &str, account_id: &str, ignore_exist_error: bool, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        IamRelServ::add_simple_rel(&IamRelKind::IamAccountApp, account_id, app_id, None, None, ignore_exist_error, false, funs, ctx).await
    }

    pub async fn delete_rel_account(app_id: &str, account_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        IamRelServ::delete_simple_rel(&IamRelKind::IamAccountApp, account_id, app_id, funs, ctx).await?;
        // todo delete app rel account and role
        let rel_account_roles =
            RbumRelServ::find_from_simple_rels(&IamRelKind::IamAccountRole.to_string(), &RbumRelFromKind::Item, true, account_id, None, None, funs, ctx).await?;
        if rel_account_roles.is_empty() {
            return Ok(());
        }
        for rel in rel_account_roles {
            IamRoleServ::delete_rel_account(&rel.rel_id, account_id, Some(RBUM_SCOPE_LEVEL_APP), funs, ctx).await?;
        }
        Ok(())
    }

    pub async fn count_rel_accounts(app_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<u64> {
        IamRelServ::count_to_rels(&IamRelKind::IamAccountApp, app_id, funs, ctx).await
    }

    pub fn with_app_rel_filter(ctx: &TardisContext, funs: &TardisFunsInst) -> TardisResult<Option<RbumItemRelFilterReq>> {
        Ok(Some(RbumItemRelFilterReq {
            rel_by_from: true,
            tag: Some(IamRelKind::IamAccountApp.to_string()),
            from_rbum_kind: Some(RbumRelFromKind::Item),
            rel_item_id: Some(Self::get_id_by_ctx(ctx, funs)?),
            own_paths: Some(ctx.own_paths.clone()),
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
        SpiKvClient::add_or_modify_key_name(&format!("{}:{app_id}", funs.conf::<IamConfig>().spi.kv_app_prefix.clone()), &app.name, funs, ctx).await?;

        Ok(())
    }
}
