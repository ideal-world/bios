use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi::{param::Path, payload::Json, OpenApi};
use tardis::web::web_resp::{TardisApiResult, TardisResp, Void};

use bios_basic::rbum::dto::rbum_item_attr_dto::{RbumItemAttrDetailResp, RbumItemAttrModifyReq};
use bios_basic::rbum::dto::rbum_kind_attr_dto::{RbumKindAttrDetailResp, RbumKindAttrModifyReq, RbumKindAttrSummaryResp};

use crate::basic::dto::iam_attr_dto::{IamItemAttrAddReq, IamKindAttrAddReq};
use crate::basic::serv::iam_attr_serv::IamAttrServ;
use crate::iam_constants;
use crate::iam_constants::RBUM_SCOPE_LEVEL_APP;

pub struct IamCaAccountAttrApi;

/// App Console Account Attr API
#[OpenApi(prefix_path = "/ca/account", tag = "crate::iam_enumeration::Tag::App")]
impl IamCaAccountAttrApi {
    /// Add Account Attr
    #[oai(path = "/attr", method = "post")]
    async fn add_attr(&self, mut add_req: Json<IamKindAttrAddReq>, cxt: TardisContextExtractor) -> TardisApiResult<String> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let result = IamAttrServ::add_account_attr(&mut add_req.0, RBUM_SCOPE_LEVEL_APP, &funs, &cxt.0).await?;
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
        let result = IamAttrServ::get_account_attr(&id.0, false, &iam_constants::get_tardis_inst(), &cxt.0).await?;
        TardisResp::ok(result)
    }

    /// Find Account Attrs
    #[oai(path = "/attr", method = "get")]
    async fn find_attrs(&self, cxt: TardisContextExtractor) -> TardisApiResult<Vec<RbumKindAttrSummaryResp>> {
        let result = IamAttrServ::find_account_attrs(false, &iam_constants::get_tardis_inst(), &cxt.0).await?;
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
    #[oai(path = "/:account_id/attr/:attr_id", method = "put")]
    async fn add_attr_value(&self, account_id: Path<String>, attr_id: Path<String>, add_req: Json<IamItemAttrAddReq>, cxt: TardisContextExtractor) -> TardisApiResult<String> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let result = IamAttrServ::add_account_attr_value(add_req.0.value, &attr_id.0, &account_id.0, &funs, &cxt.0).await?;
        funs.commit().await?;
        TardisResp::ok(result)
    }

    /// Modify Account Attr Value By Id
    #[oai(path = "/attr/value/:id", method = "put")]
    async fn modify_attr_value(&self, id: Path<String>, modify_req: Json<RbumItemAttrModifyReq>, cxt: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamAttrServ::modify_account_attr_value(&id.0, modify_req.0.value, &funs, &cxt.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Get Account Attr Value By Id
    #[oai(path = "/attr/value/:id", method = "get")]
    async fn get_attr_value(&self, id: Path<String>, cxt: TardisContextExtractor) -> TardisApiResult<RbumItemAttrDetailResp> {
        let result = IamAttrServ::get_account_attr_value(&id.0, false, &iam_constants::get_tardis_inst(), &cxt.0).await?;
        TardisResp::ok(result)
    }

    /// Delete Account Attr Value By Id
    #[oai(path = "/attr/value/:id", method = "delete")]
    async fn delete_attr_value(&self, id: Path<String>, cxt: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamAttrServ::delete_account_attr_value(&id.0, &funs, &cxt.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }
}
