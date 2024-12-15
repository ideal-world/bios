use std::collections::HashMap;

use bios_basic::rbum::dto::rbum_filer_dto::RbumBasicFilterReq;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;
use bios_sdk_invoke::clients::spi_kv_client::KvItemSummaryResp;
use bios_sdk_invoke::clients::spi_search_client::SpiSearchClient;
use bios_sdk_invoke::dto::search_item_dto::SearchItemModifyReq;
use serde_json::json;
use tardis::basic::dto::TardisContext;
use tardis::tokio;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::payload::Json;
use tardis::web::web_resp::{TardisApiResult, TardisPage, TardisResp, Void};

use crate::dto::flow_config_dto::FlowConfigModifyReq;

use crate::dto::flow_state_dto::FlowStateFilterReq;
use crate::flow_constants;
use crate::serv::flow_config_serv::FlowConfigServ;
use crate::serv::flow_inst_serv::FlowInstServ;
use crate::serv::flow_state_serv::FlowStateServ;
#[derive(Clone)]
pub struct FlowCsConfigApi;

/// Flow Config process API
#[poem_openapi::OpenApi(prefix_path = "/cs/config")]
impl FlowCsConfigApi {
    /// Modify Config / 编辑配置
    #[oai(path = "/", method = "post")]
    async fn modify_config(&self, req: Json<Vec<FlowConfigModifyReq>>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = flow_constants::get_tardis_inst();
        funs.begin().await?;
        FlowConfigServ::modify_config(&req.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Get Config / 获取配置
    #[oai(path = "/", method = "get")]
    async fn get(&self, ctx: TardisContextExtractor) -> TardisApiResult<Option<TardisPage<KvItemSummaryResp>>> {
        let mut funs = flow_constants::get_tardis_inst();
        funs.begin().await?;
        let result = FlowConfigServ::get_config(&funs, &ctx.0).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }
}
