use async_trait::async_trait;
use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::db::reldb_client::TardisRelDBlConnection;
use tardis::db::sea_orm::*;
use tardis::db::sea_query::SelectStatement;

use bios_basic::rbum::dto::filer_dto::RbumItemFilterReq;
use bios_basic::rbum::dto::rbum_item_dto::{RbumItemAddReq, RbumItemModifyReq};
use bios_basic::rbum::enumeration::RbumScopeKind;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;

use crate::console_system::dto::iam_cs_app_dto::{IamCsAppAddReq, IamCsAppDetailResp, IamCsAppModifyReq, IamCsAppSummaryResp};
use crate::constants;
use crate::domain::iam_app;

pub struct IamCsAppServ;

#[async_trait]
impl<'a> RbumItemCrudOperation<'a, iam_app::ActiveModel, IamCsAppAddReq, IamCsAppModifyReq, IamCsAppSummaryResp, IamCsAppDetailResp> for IamCsAppServ {
    fn get_ext_table_name() -> &'static str {
        iam_app::Entity.table_name()
    }

    fn get_rbum_kind_id() -> String {
        constants::get_rbum_basic_info().kind_app_id.clone()
    }

    fn get_rbum_domain_id() -> String {
        constants::get_rbum_basic_info().domain_iam_id.clone()
    }

    async fn package_item_add(add_req: &IamCsAppAddReq, _: &TardisRelDBlConnection<'a>, _: &TardisContext) -> TardisResult<RbumItemAddReq> {
        Ok(RbumItemAddReq {
            code: add_req.code.clone(),
            uri_path: None,
            name: add_req.name.clone(),
            icon: add_req.icon.clone(),
            sort: None,
            scope_kind: Some(RbumScopeKind::Global),
            disabled: add_req.disabled,
            rel_rbum_kind_id: "".to_string(),
            rel_rbum_domain_id: "".to_string(),
        })
    }

    async fn package_ext_add(id: &str, add_req: &IamCsAppAddReq, _: &TardisRelDBlConnection<'a>, _: &TardisContext) -> TardisResult<iam_app::ActiveModel> {
        Ok(iam_app::ActiveModel {
            id: Set(id.to_string()),
            contact_phone: Set(add_req.contact_phone.as_ref().unwrap_or(&"".to_string()).to_string()),
        })
    }

    async fn package_item_modify(_: &str, modify_req: &IamCsAppModifyReq, _: &TardisRelDBlConnection<'a>, _: &TardisContext) -> TardisResult<Option<RbumItemModifyReq>> {
        if modify_req.name.is_none() && modify_req.icon.is_none() && modify_req.sort.is_none() && modify_req.disabled.is_none() {
            return Ok(None);
        }
        Ok(Some(RbumItemModifyReq {
            code: None,
            uri_path: None,
            name: modify_req.name.clone(),
            icon: modify_req.icon.clone(),
            sort: modify_req.sort,
            scope_kind: None,
            disabled: modify_req.disabled,
        }))
    }

    async fn package_ext_modify(id: &str, modify_req: &IamCsAppModifyReq, _: &TardisRelDBlConnection<'a>, _: &TardisContext) -> TardisResult<Option<iam_app::ActiveModel>> {
        if modify_req.contact_phone.is_none() {
            return Ok(None);
        }
        let mut iam_app = iam_app::ActiveModel {
            id: Set(id.to_string()),
            ..Default::default()
        };
        if let Some(contact_phone) = &modify_req.contact_phone {
            iam_app.contact_phone = Set(contact_phone.to_string());
        }
        Ok(Some(iam_app))
    }

    async fn package_item_query(query: &mut SelectStatement, _: bool, _: &RbumItemFilterReq, _: &TardisRelDBlConnection<'a>, _: &TardisContext) -> TardisResult<()> {
        query.column((iam_app::Entity, iam_app::Column::ContactPhone));
        Ok(())
    }
}
