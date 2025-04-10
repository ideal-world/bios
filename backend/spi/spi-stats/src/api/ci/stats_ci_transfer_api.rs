use bios_basic::process::task_processor::TaskProcessor;
use tardis::web::{
    context_extractor::TardisContextExtractor,
    poem::web::Json,
    poem_openapi::{self, param::Path},
    web_resp::{TardisApiResult, TardisResp, Void},
};

use crate::{
    dto::stats_transfer_dto::{StatsExportDataReq, StatsExportDataResp, StatsImportDataReq},
    serv::stats_transfer_serv,
};

#[derive(Clone)]
pub struct StatsCiTransferApi;

/// Interface Console Statistics Transfer API
///
/// stats 数据传输接口
#[poem_openapi::OpenApi(prefix_path = "/ci", tag = "bios_basic::ApiTag::Interface")]
impl StatsCiTransferApi {
    #[oai(path = "/export", method = "put")]
    async fn export_data(&self, export_req: Json<StatsExportDataReq>, ctx: TardisContextExtractor) -> TardisApiResult<StatsExportDataResp> {
        let funs = crate::get_tardis_inst();
        let result = stats_transfer_serv::export_data(&export_req.0, &funs, &ctx.0).await?;
        TardisResp::ok(result)
    }

    #[oai(path = "/import", method = "put")]
    async fn import_data(&self, import_req: Json<StatsImportDataReq>, ctx: TardisContextExtractor) -> TardisApiResult<Option<String>> {
        let funs = crate::get_tardis_inst();
        let _ = stats_transfer_serv::import_data(&import_req.0, &funs, &ctx.0).await?;
        if let Some(task_id) = TaskProcessor::get_task_id_with_ctx(&ctx.0).await? {
            TardisResp::accepted(Some(task_id))
        } else {
            TardisResp::ok(None)
        }
    }
}
