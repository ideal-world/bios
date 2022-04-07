use async_trait::async_trait;
use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::db::reldb_client::TardisRelDBlConnection;
use tardis::db::sea_orm::*;
use tardis::db::sea_query::SelectStatement;

use bios_basic::rbum::dto::filer_dto::RbumItemFilterReq;
use bios_basic::rbum::dto::rbum_item_dto::{RbumItemAddReq, RbumItemModifyReq};
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;

use crate::basic::constants;
use crate::basic::domain::iam_account;
use crate::basic::dto::iam_account_dto::{IamAccountAddReq, IamAccountDetailResp, IamAccountModifyReq, IamAccountSummaryResp};

pub struct IamAccountServ;

#[async_trait]
impl<'a> RbumItemCrudOperation<'a, iam_account::ActiveModel, IamAccountAddReq, IamAccountModifyReq, IamAccountSummaryResp, IamAccountDetailResp> for IamAccountServ {
    fn get_ext_table_name() -> &'static str {
        iam_account::Entity.table_name()
    }

    fn get_rbum_kind_id() -> String {
        constants::get_rbum_basic_info().kind_account_id.clone()
    }

    fn get_rbum_domain_id() -> String {
        constants::get_rbum_basic_info().domain_iam_id.clone()
    }

    async fn package_item_add(add_req: &IamAccountAddReq, _: &TardisFunsInst<'a>, _: &TardisContext) -> TardisResult<RbumItemAddReq> {
        Ok(RbumItemAddReq {
            id: add_req.id.clone(),
            code: None,
            name: add_req.name.clone(),
            icon: add_req.icon.clone(),
            sort: None,
            disabled: add_req.disabled,
            rel_rbum_kind_id: "".to_string(),
            rel_rbum_domain_id: "".to_string(),
            scope_level: add_req.scope_level,
        })
    }

    async fn package_ext_add(id: &str, _: &IamAccountAddReq, _: &TardisFunsInst<'a>, _: &TardisContext) -> TardisResult<iam_account::ActiveModel> {
        Ok(iam_account::ActiveModel { id: Set(id.to_string()) })
    }

    async fn package_item_modify(_: &str, modify_req: &IamAccountModifyReq, _: &TardisFunsInst<'a>, _: &TardisContext) -> TardisResult<Option<RbumItemModifyReq>> {
        if modify_req.name.is_none() && modify_req.icon.is_none() && modify_req.scope_level.is_none() && modify_req.disabled.is_none() {
            return Ok(None);
        }
        Ok(Some(RbumItemModifyReq {
            code: None,
            name: modify_req.name.clone(),
            icon: modify_req.icon.clone(),
            sort: None,
            scope_level: modify_req.scope_level.clone(),
            disabled: modify_req.disabled,
        }))
    }

    async fn package_ext_modify(_: &str, _: &IamAccountModifyReq, _: &TardisFunsInst<'a>, _: &TardisContext) -> TardisResult<Option<iam_account::ActiveModel>> {
        return Ok(None);
    }

    async fn package_item_query(_: &mut SelectStatement, _: bool, _: &RbumItemFilterReq, _: &TardisFunsInst<'a>, _: &TardisContext) -> TardisResult<()> {
        Ok(())
    }
}
