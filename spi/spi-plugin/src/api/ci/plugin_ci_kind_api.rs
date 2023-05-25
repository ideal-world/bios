use bios_basic::rbum::dto::rbum_filer_dto::{RbumBasicFilterReq, RbumKindAttrFilterReq, RbumKindFilterReq};
use bios_basic::rbum::dto::rbum_kind_attr_dto::RbumKindAttrSummaryResp;
use bios_basic::rbum::dto::rbum_kind_dto::RbumKindSummaryResp;
use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;
use bios_basic::rbum::serv::rbum_kind_serv::{RbumKindAttrServ, RbumKindServ};
use bios_basic::TardisFunInstExtractor;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem::Request;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::param::{Path, Query};
use tardis::web::web_resp::{TardisApiResult, TardisPage, TardisResp};

use crate::plugin_constants::KIND_MODULE_CODE;

pub struct PluginKindApi;

/// Plugin kind API
#[poem_openapi::OpenApi(prefix_path = "/ci/kind", tag = "bios_basic::ApiTag::Interface")]
impl PluginKindApi {
    /// find Plugin kind
    #[oai(path = "/", method = "get")]
    async fn find_page(
        &self,
        code: Query<Option<String>>,
        page_number: Query<u32>,
        page_size: Query<u32>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<TardisPage<RbumKindSummaryResp>> {
        let funs = request.tardis_fun_inst();
        let result = RbumKindServ::paginate_rbums(
            &RbumKindFilterReq {
                basic: RbumBasicFilterReq {
                    code: code.0,
                    ..Default::default()
                },
                module: Some(KIND_MODULE_CODE.to_string()),
            },
            page_number.0,
            page_size.0,
            None,
            None,
            &funs,
            &ctx.0,
        )
        .await?;
        TardisResp::ok(result)
    }

    /// find Plugin kind attr
    #[oai(path = "/attr/:kind_id", method = "get")]
    async fn find_kind_attr(&self, kind_id: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Vec<RbumKindAttrSummaryResp>> {
        let funs = request.tardis_fun_inst();
        let result = RbumKindAttrServ::find_rbums(
            &RbumKindAttrFilterReq {
                basic: RbumBasicFilterReq {
                    rbum_kind_id: Some(kind_id.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            None,
            None,
            &funs,
            &ctx.0,
        )
        .await?;
        TardisResp::ok(result)
    }
}
