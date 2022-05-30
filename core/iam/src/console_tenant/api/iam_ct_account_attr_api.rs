use std::collections::HashMap;

use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi::{param::Path, param::Query, payload::Json, OpenApi};
use tardis::web::web_resp::{TardisApiResult, TardisResp, Void};

use bios_basic::rbum::dto::rbum_kind_attr_dto::{RbumKindAttrDetailResp, RbumKindAttrModifyReq, RbumKindAttrSummaryResp};

use crate::basic::dto::iam_attr_dto::IamKindAttrAddReq;
use crate::basic::serv::iam_attr_serv::IamAttrServ;
use crate::iam_constants;

pub struct IamCtAccountAttrApi;

/// Tenant Console Account Attr API
///
/// Note: the current account attr only supports tenant level.
#[OpenApi(prefix_path = "/ct/account/attr", tag = "crate::iam_enumeration::Tag::Tenant")]
impl IamCtAccountAttrApi {
    /// Add Account Attr By Current Tenant
    #[oai(path = "/", method = "post")]
    async fn add_attr(&self, add_req: Json<IamKindAttrAddReq>, cxt: TardisContextExtractor) -> TardisApiResult<String> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let result = IamAttrServ::add_account_attr(&add_req.0, &funs, &cxt.0).await?;
        funs.commit().await?;
        TardisResp::ok(result)
    }

    /// Modify Account Attr By Account Attr Id
    #[oai(path = "/:id", method = "put")]
    async fn modify_attr(&self, id: Path<String>, mut modify_req: Json<RbumKindAttrModifyReq>, cxt: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamAttrServ::modify_account_attr(&id.0, &mut modify_req.0, &funs, &cxt.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Get Account Attr By Account Attr Id
    #[oai(path = "/:id", method = "get")]
    async fn get_attr(&self, id: Path<String>, cxt: TardisContextExtractor) -> TardisApiResult<RbumKindAttrDetailResp> {
        let funs = iam_constants::get_tardis_inst();
        let result = IamAttrServ::get_account_attr(&id.0, true, &funs, &cxt.0).await?;
        TardisResp::ok(result)
    }

    /// Find Account Attrs By Current Tenant
    #[oai(path = "/", method = "get")]
    async fn find_attrs(&self, cxt: TardisContextExtractor) -> TardisApiResult<Vec<RbumKindAttrSummaryResp>> {
        let funs = iam_constants::get_tardis_inst();
        let result = IamAttrServ::find_account_attrs(&funs, &cxt.0).await?;
        TardisResp::ok(result)
    }

    /// Delete Account Attr By Account Attr Id
    #[oai(path = "/:id", method = "delete")]
    async fn delete_attr(&self, id: Path<String>, cxt: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamAttrServ::delete_account_attr(&id.0, &funs, &cxt.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Find Account Ext Attr Values By Account Id
    #[oai(path = "/value", method = "get")]
    async fn find_account_attr_values(&self, account_id: Query<String>, cxt: TardisContextExtractor) -> TardisApiResult<HashMap<String, String>> {
        let funs = iam_constants::get_tardis_inst();
        let result = IamAttrServ::find_account_attr_values(&account_id.0, &funs, &cxt.0).await?;
        TardisResp::ok(result)
    }
}
