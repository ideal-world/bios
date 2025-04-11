use bios_basic::process::task_processor::TaskProcessor;
use tardis::web::{
    context_extractor::TardisContextExtractor,
    poem::web::Json,
    poem_openapi,
    web_resp::{TardisApiResult, TardisResp},
};

use crate::{
    dto::kv_transfer_dto::{KvExportDataReq, KvExportDataResp, KvImportDataReq},
    serv::kv_transfer_serv,
};

#[derive(Clone)]
pub struct KvTransferApi;

/// Interface Console Kv Transfer API
///
/// Kv 数据传输接口
#[poem_openapi::OpenApi(prefix_path = "/ci", tag = "bios_basic::ApiTag::Interface")]
impl KvTransferApi {
    #[oai(path = "/export", method = "put")]
    async fn export_data(&self, export_req: Json<KvExportDataReq>, ctx: TardisContextExtractor) -> TardisApiResult<KvExportDataResp> {
        let funs = crate::get_tardis_inst();
        let result = kv_transfer_serv::export_data(&export_req.0, &funs, &ctx.0).await?;
        TardisResp::ok(result)
    }

    #[oai(path = "/import", method = "put")]
    async fn import_data(&self, import_req: Json<KvImportDataReq>, ctx: TardisContextExtractor) -> TardisApiResult<Option<String>> {
        let funs = crate::get_tardis_inst();
        let _ = kv_transfer_serv::import_data(&import_req.0, &funs, &ctx.0).await?;
        if let Some(task_id) = TaskProcessor::get_task_id_with_ctx(&ctx.0).await? {
            TardisResp::accepted(Some(task_id))
        } else {
            TardisResp::ok(None)
        }
    }
}
