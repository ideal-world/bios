use async_trait::async_trait;
use tardis::basic::dto::TardisContext;
use tardis::basic::error::TardisError;
use tardis::basic::result::TardisResult;
use tardis::db::reldb_client::TardisRelDBlConnection;
use tardis::db::sea_orm::*;
use tardis::db::sea_query::SelectStatement;

use bios_basic::rbum::dto::filer_dto::RbumItemFilterReq;
use bios_basic::rbum::dto::rbum_item_dto::{RbumItemAddReq, RbumItemModifyReq};
use bios_basic::rbum::dto::rbum_rel_dto::RbumRelCheckReq;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;
use bios_basic::rbum::serv::rbum_rel_serv::RbumRelServ;

use crate::basic::constants;
use crate::basic::domain::iam_role;
use crate::basic::dto::iam_role_dto::{IamRoleAddReq, IamRoleDetailResp, IamRoleModifyReq, IamRoleSummaryResp};

pub struct IamRoleServ;

#[async_trait]
impl<'a> RbumItemCrudOperation<'a, iam_role::ActiveModel, IamRoleAddReq, IamRoleModifyReq, IamRoleSummaryResp, IamRoleDetailResp> for IamRoleServ {
    fn get_ext_table_name() -> &'static str {
        iam_role::Entity.table_name()
    }

    fn get_rbum_kind_id() -> String {
        constants::get_rbum_basic_info().kind_role_id.clone()
    }

    fn get_rbum_domain_id() -> String {
        constants::get_rbum_basic_info().domain_iam_id.clone()
    }

    async fn package_item_add(add_req: &IamRoleAddReq, _: &TardisRelDBlConnection<'a>, _: &TardisContext) -> TardisResult<RbumItemAddReq> {
        Ok(RbumItemAddReq {
            id: None,
            uri_path: None,
            name: add_req.name.clone(),
            icon: None,
            sort: None,
            disabled: None,
            rel_rbum_kind_id: "".to_string(),
            rel_rbum_domain_id: "".to_string(),
            scope_level: add_req.scope_level,
        })
    }

    async fn package_ext_add(id: &str, _: &IamRoleAddReq, _: &TardisRelDBlConnection<'a>, _: &TardisContext) -> TardisResult<iam_role::ActiveModel> {
        Ok(iam_role::ActiveModel { id: Set(id.to_string()) })
    }

    async fn package_item_modify(_: &str, modify_req: &IamRoleModifyReq, _: &TardisRelDBlConnection<'a>, _: &TardisContext) -> TardisResult<Option<RbumItemModifyReq>> {
        if modify_req.name.is_none() && modify_req.icon.is_none() && modify_req.sort.is_none() && modify_req.scope_level.is_none() && modify_req.disabled.is_none() {
            return Ok(None);
        }
        Ok(Some(RbumItemModifyReq {
            uri_path: None,
            name: modify_req.name.clone(),
            icon: modify_req.icon.clone(),
            sort: modify_req.sort,
            scope_level: modify_req.scope_level,
            disabled: modify_req.disabled,
        }))
    }

    async fn package_ext_modify(_: &str, _: &IamRoleModifyReq, _: &TardisRelDBlConnection<'a>, _: &TardisContext) -> TardisResult<Option<iam_role::ActiveModel>> {
        return Ok(None);
    }

    async fn package_item_query(_: &mut SelectStatement, _: bool, _: &RbumItemFilterReq, _: &TardisRelDBlConnection<'a>, _: &TardisContext) -> TardisResult<()> {
        Ok(())
    }
}

impl IamRoleServ {
    pub async fn need_sys_admin<'a>(db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<()> {
        Self::need_role(&constants::get_rbum_basic_info().role_sys_admin_id, db, cxt).await
    }

    pub async fn need_tenant_admin<'a>(db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<()> {
        Self::need_role(&constants::get_rbum_basic_info().role_tenant_admin_id, db, cxt).await
    }

    pub async fn need_app_admin<'a>(db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<()> {
        Self::need_role(&constants::get_rbum_basic_info().role_app_admin_id, db, cxt).await
    }

    pub async fn need_role<'a>(iam_role_id: &str, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<()> {
        // TODO cache
        let exist = RbumRelServ::check_rel(
            &mut RbumRelCheckReq {
                tag: constants::RBUM_REL_BIND.to_string(),
                from_rbum_item_id: cxt.account_id.clone(),
                to_rbum_item_id: iam_role_id.to_string(),
                from_attrs: Default::default(),
                to_attrs: Default::default(),
            },
            db,
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
