use bios_basic::rbum::dto::rbum_filer_dto::RbumBasicFilterReq;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;

use bios_basic::spi::serv::spi_bs_serv::SpiBsServ;
use tardis::chrono::{self, Utc};
use tardis::web::context_extractor::TardisContextExtractor;

use tardis::web::poem_openapi;
use tardis::web::poem_openapi::param::{Path, Query};
use tardis::web::poem_openapi::payload::Json;
use tardis::web::web_resp::{TardisApiResult, TardisPage, TardisResp, Void};

use crate::dto::plugin_api_dto::{PluginApiAddOrModifyReq, PluginApiDetailResp, PluginApiFilterReq, PluginApiSummaryResp};
use crate::serv::plugin_api_serv::PluginApiServ;
#[derive(Clone)]

pub struct PluginApiApi;

/// Plugin Api API
#[poem_openapi::OpenApi(prefix_path = "/ci/spi/plugin/api", tag = "bios_basic::ApiTag::Interface")]
impl PluginApiApi {
    /// Add or modify Plugin Api
    #[oai(path = "/", method = "put")]
    async fn add(&self, mut add_or_modify_req: Json<PluginApiAddOrModifyReq>, ctx: TardisContextExtractor) -> TardisApiResult<String> {
        let mut funs = crate::get_tardis_inst();
        funs.begin().await?;
        let id = PluginApiServ::add_or_modify_item(&mut add_or_modify_req.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        TardisResp::ok(id)
    }

    /// Delete Plugin Api
    #[oai(path = "/:code", method = "delete")]
    async fn delete(&self, code: Path<String>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let funs = crate::get_tardis_inst();
        PluginApiServ::delete_by_code(&code.0, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    #[oai(path = "/kind/:kind_id", method = "delete")]
    async fn delete_by_kind(&self, kind_id: Path<String>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let funs = crate::get_tardis_inst();
        PluginApiServ::delete_by_kind(&kind_id.0, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    /// Get Plugin Api
    #[oai(path = "/:kind_id/:code", method = "get")]
    async fn get_api_by_code(&self, kind_id: Path<String>, code: Path<String>, ctx: TardisContextExtractor) -> TardisApiResult<PluginApiDetailResp> {
        let funs = crate::get_tardis_inst();
        let result = PluginApiServ::get_by_code(&kind_id.0, &code.0, &funs, &ctx.0)
            .await?
            .ok_or_else(|| funs.err().not_found(&SpiBsServ::get_obj_name(), "get_bs_by_rel", "api is not fond", "404-plugin-api-not-exist"))?;
        TardisResp::ok(result)
    }

    /// find Plugin Api page
    #[oai(path = "/", method = "get")]
    async fn find_page(
        &self,
        code: Query<Option<String>>,
        path_and_query: Query<Option<String>>,
        create_start: Query<Option<chrono::DateTime<Utc>>>,
        create_end: Query<Option<chrono::DateTime<Utc>>>,
        page_number: Query<u32>,
        page_size: Query<u32>,
        ctx: TardisContextExtractor,
    ) -> TardisApiResult<TardisPage<PluginApiSummaryResp>> {
        let funs = crate::get_tardis_inst();
        let result = PluginApiServ::paginate_items(
            &PluginApiFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                code: code.0,
                path_and_query: path_and_query.0,
                create_start: create_start.0,
                create_end: create_end.0,
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
}
