use async_trait::async_trait;
use sea_orm::EntityName;
use tardis::basic::dto::{TardisContext, TardisFunsInst};
use tardis::basic::error::TardisError;
use tardis::basic::result::TardisResult;
use tardis::db::sea_orm::*;
use tardis::db::sea_query::SelectStatement;

use bios_basic::rbum::dto::rbum_item_dto::{RbumItemKernelAddReq, RbumItemModifyReq};
use bios_basic::rbum::dto::rbum_rel_dto::RbumRelCheckReq;
use bios_basic::rbum::rbum_enumeration::RbumRelFromKind;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;
use bios_basic::rbum::serv::rbum_rel_serv::RbumRelServ;

use crate::basic::domain::iam_role;
use crate::basic::dto::iam_filer_dto::IamRoleFilterReq;
use crate::basic::dto::iam_role_dto::{IamRoleAddReq, IamRoleDetailResp, IamRoleModifyReq, IamRoleSummaryResp};
use crate::iam_config::IamBasicInfoManager;
use crate::iam_enumeration::IAMRelKind;

pub struct IamRoleServ;

#[async_trait]
impl<'a> RbumItemCrudOperation<'a, iam_role::ActiveModel, IamRoleAddReq, IamRoleModifyReq, IamRoleSummaryResp, IamRoleDetailResp, IamRoleFilterReq> for IamRoleServ {
    fn get_ext_table_name() -> &'static str {
        iam_role::Entity.table_name()
    }

    fn get_rbum_kind_id() -> String {
        IamBasicInfoManager::get().kind_role_id
    }

    fn get_rbum_domain_id() -> String {
        IamBasicInfoManager::get().domain_iam_id
    }

    async fn package_item_add(add_req: &IamRoleAddReq, _: &TardisFunsInst<'a>, _: &TardisContext) -> TardisResult<RbumItemKernelAddReq> {
        Ok(RbumItemKernelAddReq {
            id: None,
            code: None,
            name: add_req.name.clone(),
            disabled: None,
            scope_level: add_req.scope_level.clone(),
        })
    }

    async fn package_ext_add(id: &str, add_req: &IamRoleAddReq, _: &TardisFunsInst<'a>, _: &TardisContext) -> TardisResult<iam_role::ActiveModel> {
        Ok(iam_role::ActiveModel {
            id: Set(id.to_string()),
            icon: Set(add_req.icon.as_ref().unwrap_or(&"".to_string()).to_string()),
            sort: Set(add_req.sort.unwrap_or(0)),
        })
    }

    async fn package_item_modify(_: &str, modify_req: &IamRoleModifyReq, _: &TardisFunsInst<'a>, _: &TardisContext) -> TardisResult<Option<RbumItemModifyReq>> {
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

    async fn package_ext_modify(id: &str, modify_req: &IamRoleModifyReq, _: &TardisFunsInst<'a>, _: &TardisContext) -> TardisResult<Option<iam_role::ActiveModel>> {
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
        Ok(Some(iam_role))
    }

    async fn package_ext_query(query: &mut SelectStatement, _: bool, _: &IamRoleFilterReq, _: &TardisFunsInst<'a>, _: &TardisContext) -> TardisResult<()> {
        query.column((iam_role::Entity, iam_role::Column::Icon));
        query.column((iam_role::Entity, iam_role::Column::Sort));
        Ok(())
    }
}

impl IamRoleServ {
    pub async fn need_sys_admin<'a>(funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<()> {
        Self::need_role(&IamBasicInfoManager::get().role_sys_admin_id, funs, cxt).await
    }

    pub async fn need_tenant_admin<'a>(funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<()> {
        Self::need_role(&IamBasicInfoManager::get().role_tenant_admin_id, funs, cxt).await
    }

    pub async fn need_app_admin<'a>(funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<()> {
        Self::need_role(&IamBasicInfoManager::get().role_app_admin_id, funs, cxt).await
    }

    pub async fn need_role<'a>(iam_role_id: &str, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<()> {
        // TODO cache
        let exist = RbumRelServ::check_rel(
            &RbumRelCheckReq {
                tag: IAMRelKind::IamAccountRole.to_string(),
                from_rbum_kind: RbumRelFromKind::Item,
                from_rbum_id: cxt.owner.clone(),
                to_rbum_item_id: iam_role_id.to_string(),
                from_attrs: Default::default(),
                to_attrs: Default::default(),
            },
            funs,
            cxt,
        )
        .await?;
        if !exist {
            Err(TardisError::Unauthorized("illegal operation".to_string()))
        } else {
            Ok(())
        }
    }
}
