use async_trait::async_trait;
use tardis::basic::dto::{TardisContext, TardisFunsInst};
use tardis::basic::result::TardisResult;
use tardis::db::sea_orm::*;
use tardis::db::sea_query::SelectStatement;

use bios_basic::rbum::dto::rbum_item_dto::{RbumItemAddReq, RbumItemModifyReq};
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;

use crate::basic::domain::iam_account;
use crate::basic::dto::iam_account_dto::{IamAccountAddReq, IamAccountDetailResp, IamAccountModifyReq, IamAccountSummaryResp};
use crate::basic::dto::iam_filer_dto::IamAccountFilterReq;
use crate::iam_config::IamBasicInfoManager;

pub struct IamAccountServ;

#[async_trait]
impl<'a> RbumItemCrudOperation<'a, iam_account::ActiveModel, IamAccountAddReq, IamAccountModifyReq, IamAccountSummaryResp, IamAccountDetailResp, IamAccountFilterReq>
    for IamAccountServ
{
    fn get_ext_table_name() -> &'static str {
        iam_account::Entity.table_name()
    }

    fn get_rbum_kind_id() -> String {
        IamBasicInfoManager::get().kind_account_id.clone()
    }

    fn get_rbum_domain_id() -> String {
        IamBasicInfoManager::get().domain_iam_id.clone()
    }

    async fn package_item_add(add_req: &IamAccountAddReq, _: &TardisFunsInst<'a>, _: &TardisContext) -> TardisResult<RbumItemAddReq> {
        Ok(RbumItemAddReq {
            id: add_req.id.clone(),
            code: None,
            name: add_req.name.clone(),
            disabled: add_req.disabled,
            rel_rbum_kind_id: "".to_string(),
            rel_rbum_domain_id: "".to_string(),
            scope_level: add_req.scope_level.clone(),
        })
    }

    async fn package_ext_add(id: &str, add_req: &IamAccountAddReq, _: &TardisFunsInst<'a>, _: &TardisContext) -> TardisResult<iam_account::ActiveModel> {
        Ok(iam_account::ActiveModel {
            id: Set(id.to_string()),
            icon: Set(add_req.icon.as_ref().unwrap_or(&"".to_string()).to_string()),
        })
    }

    async fn package_item_modify(_: &str, modify_req: &IamAccountModifyReq, _: &TardisFunsInst<'a>, _: &TardisContext) -> TardisResult<Option<RbumItemModifyReq>> {
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

    async fn package_ext_modify(id: &str, modify_req: &IamAccountModifyReq, _: &TardisFunsInst<'a>, _: &TardisContext) -> TardisResult<Option<iam_account::ActiveModel>> {
        if modify_req.icon.is_none() {
            return Ok(None);
        }
        let mut iam_account = iam_account::ActiveModel {
            id: Set(id.to_string()),
            ..Default::default()
        };
        if let Some(icon) = &modify_req.icon {
            iam_account.icon = Set(icon.to_string());
        }
        Ok(Some(iam_account))
    }

    async fn package_item_query(query: &mut SelectStatement, _: bool, _: &IamAccountFilterReq, _: &TardisFunsInst<'a>, _: &TardisContext) -> TardisResult<()> {
        query.column((iam_account::Entity, iam_account::Column::Icon));
        Ok(())
    }
}
