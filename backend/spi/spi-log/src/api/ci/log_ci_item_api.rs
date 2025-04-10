use bios_basic::process::task_processor::TaskProcessor;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::param::Path;
use tardis::web::poem_openapi::payload::Json;
use tardis::web::web_resp::{TardisApiResult, TardisPage, TardisResp, Void};

use crate::dto::log_item_dto::{LogConfigReq, LogExportDataReq, LogExportDataResp, LogImportDataReq, LogItemAddReq, LogItemAddV2Req, LogItemFindReq, LogItemFindResp};
use crate::serv::{log_item_serv, log_transfer_serv};
use tardis::serde_json::Value;

#[derive(Clone)]
pub struct LogCiItemApi;

#[derive(Clone)]
pub struct LogCiItemApiV2;

/// Interface Console Log API
#[poem_openapi::OpenApi(prefix_path = "/ci/item", tag = "bios_basic::ApiTag::Interface")]
impl LogCiItemApi {
    /// Add Item
    #[oai(path = "/", method = "post")]
    async fn add(&self, mut add_req: Json<LogItemAddReq>, ctx: TardisContextExtractor) -> TardisApiResult<String> {
        let funs = crate::get_tardis_inst();
        let id = log_item_serv::add(&mut add_req.0, &funs, &ctx.0).await?;
        TardisResp::ok(id)
    }

    /// Find Items
    #[oai(path = "/find", method = "put")]
    async fn find(&self, mut find_req: Json<LogItemFindReq>, ctx: TardisContextExtractor) -> TardisApiResult<TardisPage<LogItemFindResp>> {
        let funs = crate::get_tardis_inst();
        let resp = log_item_serv::find(&mut find_req.0, &funs, &ctx.0).await?;
        TardisResp::ok(resp)
    }

    /// Modify Item ext by key
    #[oai(path = "/modify/:tag/:key/ext", method = "post")]
    async fn modify_ext(&self, tag: Path<String>, key: Path<String>, mut ext: Json<Value>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let funs = crate::get_tardis_inst();
        log_item_serv::modify_ext(&tag.0, &key.0, &mut ext.0, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }
}

/// Interface Console Log API V2
#[poem_openapi::OpenApi(prefix_path = "/ci/v2/item", tag = "bios_basic::ApiTag::Interface")]
impl LogCiItemApiV2 {
    /// Add Item
    #[oai(path = "/", method = "post")]
    async fn add(&self, mut add_req: Json<LogItemAddV2Req>, ctx: TardisContextExtractor) -> TardisApiResult<String> {
        let funs = crate::get_tardis_inst();
        let id = log_item_serv::addv2(&mut add_req.0, &funs, &ctx.0).await?;
        TardisResp::ok(id)
    }

    /// Find Items
    #[oai(path = "/find", method = "put")]
    async fn find(&self, mut find_req: Json<LogItemFindReq>, ctx: TardisContextExtractor) -> TardisApiResult<TardisPage<LogItemFindResp>> {
        let funs = crate::get_tardis_inst();
        let resp = log_item_serv::findv2(&mut find_req.0, &funs, &ctx.0).await?;
        TardisResp::ok(resp)
    }

    /// Modify Item ext by key
    #[oai(path = "/modify/:tag/:key/ext", method = "post")]
    async fn modify_ext(&self, tag: Path<String>, key: Path<String>, mut ext: Json<Value>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let funs = crate::get_tardis_inst();
        log_item_serv::modify_ext(&tag.0, &key.0, &mut ext.0, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    /// Add config
    #[oai(path = "/config", method = "post")]
    async fn add_config(&self, mut find_req: Json<LogConfigReq>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let funs = crate::get_tardis_inst();
        log_item_serv::add_config(&mut find_req.0, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    /// Delete config
    #[oai(path = "/config", method = "delete")]
    async fn delete_config(&self, mut find_req: Json<LogConfigReq>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let funs = crate::get_tardis_inst();
        log_item_serv::delete_config(&mut find_req.0, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    #[oai(path = "/export", method = "put")]
    async fn export_data(&self, export_req: Json<LogExportDataReq>, ctx: TardisContextExtractor) -> TardisApiResult<LogExportDataResp> {
        let funs = crate::get_tardis_inst();
        let result = log_transfer_serv::export_data(&export_req.0, &funs, &ctx.0).await?;
        TardisResp::ok(result)
    }

    #[oai(path = "/import", method = "put")]
    async fn import_data(&self, import_req: Json<LogImportDataReq>, ctx: TardisContextExtractor) -> TardisApiResult<Option<String>> {
        let funs = crate::get_tardis_inst();
        let _ = log_transfer_serv::import_data(&import_req.0, &funs, &ctx.0).await?;
        if let Some(task_id) = TaskProcessor::get_task_id_with_ctx(&ctx.0).await? {
            TardisResp::accepted(Some(task_id))
        } else {
            TardisResp::ok(None)
        }
    }
}
