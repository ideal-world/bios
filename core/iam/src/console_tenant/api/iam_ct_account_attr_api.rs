use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem::web::Query;
use tardis::web::poem_openapi::{OpenApi, param::Path, payload::Json};
use tardis::web::web_resp::{TardisApiResult, TardisResp, Void};

use bios_basic::rbum::dto::rbum_kind_attr_dto::{RbumKindAttrDetailResp, RbumKindAttrModifyReq, RbumKindAttrSummaryResp};

use crate::basic::dto::iam_attr_dto::IamKindAttrAddReq;
use crate::basic::serv::iam_attr_serv::IamAttrServ;
use crate::iam_constants;
use crate::iam_constants::RBUM_SCOPE_LEVEL_TENANT;

pub struct IamCtAccountAttrApi;

/// Tenant Console Account Attr API
#[OpenApi(prefix_path = "/ct/account", tag = "crate::iam_enumeration::Tag::Tenant")]
impl IamCtAccountAttrApi {
    /// Add Account Attr
    #[oai(path = "/attr", method = "post")]
    async fn add_attr(&self, mut add_req: Json<IamKindAttrAddReq>, cxt: TardisContextExtractor) -> TardisApiResult<String> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let result = IamAttrServ::add_account_attr(&mut add_req.0, RBUM_SCOPE_LEVEL_TENANT, &funs, &cxt.0).await?;
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
    async fn get_attr(&self, id: Path<String>, include_apps: Query<bool>, cxt: TardisContextExtractor) -> TardisApiResult<RbumKindAttrDetailResp> {
        let result = IamAttrServ::get_account_attr(&id.0, include_apps.0, &iam_constants::get_tardis_inst(), &cxt.0).await?;
        TardisResp::ok(result)
    }

    /// Find Account Attrs
    #[oai(path = "/attr", method = "get")]
    async fn find_attrs(&self, include_apps: Query<bool>, cxt: TardisContextExtractor) -> TardisApiResult<Vec<RbumKindAttrSummaryResp>> {
        let result = IamAttrServ::find_account_attrs(include_apps.0, &iam_constants::get_tardis_inst(), &cxt.0).await?;
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
}
