use bios_sdk_invoke::clients::spi_kv_client::KvItemSummaryResp;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::payload::Json;
use tardis::web::web_resp::{TardisApiResult, TardisResp, Void, TardisPage};

use crate::dto::flow_config_dto::FlowConfigEditReq;

use crate::flow_constants;
use crate::serv::flow_config_serv::FlowConfigServ;
#[derive(Clone)]
pub struct FlowCsConfigApi;

/// Flow Config process API
#[poem_openapi::OpenApi(prefix_path = "/cs/config")]
impl FlowCsConfigApi {
    /// Edit Config / 编辑配置
    #[oai(path = "/", method = "post")]
    async fn edit_config(&self, req: Json<Vec<FlowConfigEditReq>>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = flow_constants::get_tardis_inst();
        funs.begin().await?;
        FlowConfigServ::edit_config(&req.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Get Config / 获取配置
    #[oai(path = "/", method = "get")]
    async fn get(&self, ctx: TardisContextExtractor) -> TardisApiResult<Option<TardisPage<KvItemSummaryResp>>> {
        let mut funs = flow_constants::get_tardis_inst();
        funs.begin().await?;
        let result = FlowConfigServ::get_config(&funs, &ctx.0).await?;
        funs.commit().await?;
        TardisResp::ok(result)
    }
}
