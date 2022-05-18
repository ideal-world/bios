use std::collections::HashMap;

use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi::{param::Query, OpenApi};
use tardis::web::web_resp::{TardisApiResult, TardisResp};

use bios_basic::rbum::dto::rbum_kind_attr_dto::RbumKindAttrSummaryResp;

use crate::basic::serv::iam_attr_serv::IamAttrServ;
use crate::iam_constants;

pub struct IamCtAccountAttrApi;

/// Tenant Console Account Attr API
#[OpenApi(prefix_path = "/ct/account/attr", tag = "crate::iam_enumeration::Tag::Tenant")]
impl IamCtAccountAttrApi {
    /// Find Account Attrs By Current Tenant
    #[oai(path = "/", method = "get")]
    async fn find_attrs(&self, cxt: TardisContextExtractor) -> TardisApiResult<Vec<RbumKindAttrSummaryResp>> {
        let funs = iam_constants::get_tardis_inst();
        let result = IamAttrServ::find_account_attrs(&funs, &cxt.0).await?;
        TardisResp::ok(result)
    }

    /// Find Account Ext Attr Values
    #[oai(path = "/values", method = "get")]
    async fn find_account_attr_values(&self, account_id: Query<String>, cxt: TardisContextExtractor) -> TardisApiResult<HashMap<String, String>> {
        let funs = iam_constants::get_tardis_inst();
        let result = IamAttrServ::find_account_attr_values(&account_id.0, &funs, &cxt.0).await?;
        TardisResp::ok(result)
    }
}
