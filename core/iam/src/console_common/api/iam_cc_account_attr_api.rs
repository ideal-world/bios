use std::collections::HashMap;

use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi::{param::Path, payload::Json, OpenApi};
use tardis::web::web_resp::{TardisApiResult, TardisResp, Void};

use bios_basic::rbum::dto::rbum_item_attr_dto::RbumItemAttrSummaryResp;
use bios_basic::rbum::dto::rbum_kind_attr_dto::{RbumKindAttrDetailResp, RbumKindAttrModifyReq, RbumKindAttrSummaryResp};

use crate::basic::dto::iam_attr_dto::IamKindAttrAddReq;
use crate::basic::serv::iam_attr_serv::IamAttrServ;
use crate::iam_constants;

pub struct IamCcAccountAttrApi;

/// Common Console Account Attr API
#[OpenApi(prefix_path = "/cc/account", tag = "crate::iam_enumeration::Tag::Common")]
impl IamCcAccountAttrApi {
    /// Add Account Attr
    #[oai(path = "/attr", method = "post")]
    async fn add_attr(&self, mut add_req: Json<IamKindAttrAddReq>, cxt: TardisContextExtractor) -> TardisApiResult<String> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let result = IamAttrServ::add_account_attr(&mut add_req.0, &funs, &cxt.0).await?;
        funs.commit().await?;
        TardisResp::ok(result)
    }

    /// Modify Account Attr By Id
    #[oai(path = "/attr/:id", method = "put")]
    async fn modify_attr(&self, id: Path<String>, mut modify_req: Json<RbumKindAttrModifyReq>, cxt: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamAttrServ::modify_account_attr(&id.0, &mut modify_req.0, &funs, &cxt.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Get Account Attr By Id
    #[oai(path = "/attr/:id", method = "get")]
    async fn get_attr(&self, id: Path<String>, cxt: TardisContextExtractor) -> TardisApiResult<RbumKindAttrDetailResp> {
        let funs = iam_constants::get_tardis_inst();
        let result = IamAttrServ::get_account_attr(&id.0, &funs, &cxt.0).await?;
        TardisResp::ok(result)
    }

    /// Find Account Attrs
    #[oai(path = "/attr", method = "get")]
    async fn find_attrs(&self, cxt: TardisContextExtractor) -> TardisApiResult<Vec<RbumKindAttrSummaryResp>> {
        let funs = iam_constants::get_tardis_inst();
        let result = IamAttrServ::find_account_attrs(&funs, &cxt.0).await?;
        TardisResp::ok(result)
    }

    /// Delete Account Attr By Id
    #[oai(path = "/attr/:id", method = "delete")]
    async fn delete_attr(&self, id: Path<String>, cxt: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamAttrServ::delete_account_attr(&id.0, &funs, &cxt.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Add Account Attr Value
    #[oai(path = "/:account_id/attr/values", method = "put")]
    async fn add_or_modify_account_attr_values(&self, account_id: Path<String>, add_req: Json<HashMap<String, String>>, cxt: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamAttrServ::add_or_modify_account_attr_values(&account_id.0, add_req.0, &funs, &cxt.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Find Account Ext Attr Values
    #[oai(path = "/:account_id/attr/values", method = "get")]
    async fn find_account_ext_attr_values(&self, account_id: Path<String>, cxt: TardisContextExtractor) -> TardisApiResult<Vec<RbumItemAttrSummaryResp>> {
        let funs = iam_constants::get_tardis_inst();
        let result = IamAttrServ::find_account_ext_attr_values(&account_id.0, false, &funs, &cxt.0).await?;
        TardisResp::ok(result)
    }
}
