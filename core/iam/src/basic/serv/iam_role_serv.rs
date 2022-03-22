use async_trait::async_trait;
use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::db::reldb_client::TardisRelDBlConnection;
use tardis::db::sea_orm::*;
use tardis::db::sea_query::SelectStatement;

use bios_basic::rbum::dto::filer_dto::RbumItemFilterReq;
use bios_basic::rbum::dto::rbum_item_dto::{RbumItemAddReq, RbumItemModifyReq};
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;

use crate::basic::domain::iam_role;
use crate::basic::dto::iam_role_dto::{IamRoleAddReq, IamRoleDetailResp, IamRoleModifyReq, IamRoleSummaryResp};
use crate::constants;

pub struct IamRoleCrudServ;

#[async_trait]
impl<'a> RbumItemCrudOperation<'a, iam_role::ActiveModel, IamRoleAddReq, IamRoleModifyReq, IamRoleSummaryResp, IamRoleDetailResp> for IamRoleCrudServ {
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
            code: None,
            uri_path: None,
            name: add_req.name.clone(),
            icon: None,
            sort: None,
            scope_kind: add_req.scope_kind.clone(),
            disabled: None,
            rel_rbum_kind_id: "".to_string(),
            rel_rbum_domain_id: "".to_string(),
        })
    }

    async fn package_ext_add(id: &str, _: &IamRoleAddReq, _: &TardisRelDBlConnection<'a>, _: &TardisContext) -> TardisResult<iam_role::ActiveModel> {
        Ok(iam_role::ActiveModel { id: Set(id.to_string()) })
    }

    async fn package_item_modify(_: &str, modify_req: &IamRoleModifyReq, _: &TardisRelDBlConnection<'a>, _: &TardisContext) -> TardisResult<Option<RbumItemModifyReq>> {
        if modify_req.name.is_none() && modify_req.icon.is_none() && modify_req.sort.is_none() &&modify_req.scope_kind.is_none() && modify_req.disabled.is_none() {
            return Ok(None);
        }
        Ok(Some(RbumItemModifyReq {
            code: None,
            uri_path: None,
            name: modify_req.name.clone(),
            icon: modify_req.icon.clone(),
            sort: modify_req.sort,
            scope_kind: modify_req.scope_kind.clone(),
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
