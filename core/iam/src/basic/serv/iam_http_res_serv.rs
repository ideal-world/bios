use async_trait::async_trait;
use tardis::basic::dto::{TardisContext, TardisFunsInst};
use tardis::basic::result::TardisResult;
use tardis::db::sea_orm::*;
use tardis::db::sea_query::{Expr, SelectStatement};

use bios_basic::rbum::dto::rbum_item_dto::{RbumItemAddReq, RbumItemModifyReq};
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;

use crate::basic::constants;
use crate::basic::domain::iam_http_res;
use crate::basic::dto::iam_filer_dto::IamHttpResFilterReq;
use crate::basic::dto::iam_http_res_dto::{IamHttpResAddReq, IamHttpResDetailResp, IamHttpResModifyReq, IamHttpResSummaryResp};

pub struct IamHttpResServ;

#[async_trait]
impl<'a> RbumItemCrudOperation<'a, iam_http_res::ActiveModel, IamHttpResAddReq, IamHttpResModifyReq, IamHttpResSummaryResp, IamHttpResDetailResp, IamHttpResFilterReq>
    for IamHttpResServ
{
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
            disabled: add_req.disabled,
            rel_rbum_kind_id: "".to_string(),
            rel_rbum_domain_id: "".to_string(),
            scope_level: add_req.scope_level.clone(),
        })
    }

    async fn package_ext_add(id: &str, add_req: &IamHttpResAddReq, _: &TardisFunsInst<'a>, _: &TardisContext) -> TardisResult<iam_http_res::ActiveModel> {
        Ok(iam_http_res::ActiveModel {
            id: Set(id.to_string()),
            icon: Set(add_req.icon.as_ref().unwrap_or(&"".to_string()).to_string()),
            sort: Set(add_req.sort.unwrap_or(0)),
            method: Set(add_req.code.0.to_string()),
        })
    }

    async fn package_item_modify(_: &str, modify_req: &IamHttpResModifyReq, _: &TardisFunsInst<'a>, _: &TardisContext) -> TardisResult<Option<RbumItemModifyReq>> {
        if modify_req.code.is_none() && modify_req.name.is_none() && modify_req.scope_level.is_none() && modify_req.disabled.is_none() {
            return Ok(None);
        }
        Ok(Some(RbumItemModifyReq {
            code: modify_req.code.clone(),
            name: modify_req.name.clone(),
            scope_level: modify_req.scope_level.clone(),
            disabled: modify_req.disabled,
        }))
    }

    async fn package_ext_modify(id: &str, modify_req: &IamHttpResModifyReq, _: &TardisFunsInst<'a>, _: &TardisContext) -> TardisResult<Option<iam_http_res::ActiveModel>> {
        if modify_req.icon.is_none() && modify_req.sort.is_none() && modify_req.method.is_none() {
            return Ok(None);
        }
        let mut iam_http_res = iam_http_res::ActiveModel {
            id: Set(id.to_string()),
            ..Default::default()
        };
        if let Some(icon) = &modify_req.icon {
            iam_http_res.icon = Set(icon.to_string());
        }
        if let Some(sort) = modify_req.sort {
            iam_http_res.sort = Set(sort);
        }
        if let Some(method) = &modify_req.method {
            iam_http_res.method = Set(method.0.to_string());
        }
        Ok(Some(iam_http_res))
    }

    async fn package_item_query(query: &mut SelectStatement, _: bool, filter: &IamHttpResFilterReq, _: &TardisFunsInst<'a>, _: &TardisContext) -> TardisResult<()> {
        query.column((iam_http_res::Entity, iam_http_res::Column::Icon));
        query.column((iam_http_res::Entity, iam_http_res::Column::Sort));
        query.column((iam_http_res::Entity, iam_http_res::Column::Method));
        if let Some(method) = &filter.method {
            query.and_where(Expr::col(iam_http_res::Column::Method).eq(method.as_str()));
        }
        Ok(())
    }
}
