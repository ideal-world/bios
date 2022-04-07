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
use crate::basic::domain::iam_http_res;
use crate::basic::dto::iam_http_res_dto::{IamHttpResAddReq, IamHttpResDetailResp, IamHttpResModifyReq, IamHttpResSummaryResp};

pub struct IamHttpResServ;

#[async_trait]
impl<'a> RbumItemCrudOperation<'a, iam_http_res::ActiveModel, IamHttpResAddReq, IamHttpResModifyReq, IamHttpResSummaryResp, IamHttpResDetailResp> for IamHttpResServ {
    fn get_ext_table_name() -> &'static str {
        iam_http_res::Entity.table_name()
    }

    fn get_rbum_kind_id() -> String {
        constants::get_rbum_basic_info().kind_http_res_id.clone()
    }

    fn get_rbum_domain_id() -> String {
        constants::get_rbum_basic_info().domain_iam_id.clone()
    }

    async fn package_item_add(add_req: &IamHttpResAddReq, _: &TardisFunsInst<'a>, _: &TardisContext) -> TardisResult<RbumItemAddReq> {
        Ok(RbumItemAddReq {
            id: None,
            code: Some(add_req.code.clone()),
            name: add_req.name.clone(),
            icon: add_req.icon.clone(),
            sort: add_req.sort,
            disabled: add_req.disabled,
            rel_rbum_kind_id: "".to_string(),
            rel_rbum_domain_id: "".to_string(),
            scope_level: add_req.scope_level,
        })
    }

    async fn package_ext_add(id: &str, add_req: &IamHttpResAddReq, _: &TardisFunsInst<'a>, _: &TardisContext) -> TardisResult<iam_http_res::ActiveModel> {
        Ok(iam_http_res::ActiveModel {
            id: Set(id.to_string()),
            method: Set(add_req.code.0.to_string()),
        })
    }

    async fn package_item_modify(_: &str, modify_req: &IamHttpResModifyReq, _: &TardisFunsInst<'a>, _: &TardisContext) -> TardisResult<Option<RbumItemModifyReq>> {
        if modify_req.code.is_none()
            && modify_req.name.is_none()
            && modify_req.icon.is_none()
            && modify_req.sort.is_none()
            && modify_req.scope_level.is_none()
            && modify_req.disabled.is_none()
        {
            return Ok(None);
        }
        Ok(Some(RbumItemModifyReq {
            code: modify_req.code.clone(),
            name: modify_req.name.clone(),
            icon: modify_req.icon.clone(),
            sort: modify_req.sort,
            scope_level: modify_req.scope_level,
            disabled: modify_req.disabled,
        }))
    }

    async fn package_ext_modify(id: &str, modify_req: &IamHttpResModifyReq, _: &TardisFunsInst<'a>, _: &TardisContext) -> TardisResult<Option<iam_http_res::ActiveModel>> {
        if modify_req.method.is_none() {
            return Ok(None);
        }
        let mut iam_http_res = iam_http_res::ActiveModel {
            id: Set(id.to_string()),
            ..Default::default()
        };
        if let Some(method) = &modify_req.method {
            iam_http_res.method = Set(method.0.to_string());
        }
        Ok(Some(iam_http_res))
    }

    async fn package_item_query(query: &mut SelectStatement, _: bool, _: &RbumItemFilterReq, _: &TardisFunsInst<'a>, _: &TardisContext) -> TardisResult<()> {
        query.column((iam_http_res::Entity, iam_http_res::Column::Method));
        Ok(())
    }
}
