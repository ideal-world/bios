use tardis::web::{
    context_extractor::TardisContextExtractor,
    poem_openapi::{self, param::Path},
    web_resp::{TardisApiResult, TardisResp, Void},
};

use crate::serv::stats_sync_serv;

#[derive(Clone)]
pub struct StatsCiSyncApi;

/// Interface Console Statistics Sync API
///
/// 统计同步接口
#[poem_openapi::OpenApi(prefix_path = "/ci", tag = "bios_basic::ApiTag::Interface")]
impl StatsCiSyncApi {
    /// Sync Fact Record
    ///
    /// 同步事实记录
    #[oai(path = "/fact/:fact_key/sync", method = "put")]
    async fn fact_record_sync(&self, fact_key: Path<String>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let funs = crate::get_tardis_inst();
        stats_sync_serv::fact_record_sync(&fact_key.0, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    /// Sync Fact Column Record
    ///
    /// 同步事实列记录
    #[oai(path = "/fact/:fact_key/col/:col_key/sync", method = "put")]
    async fn fact_col_record_sync(&self, fact_key: Path<String>, col_key: Path<String>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let funs = crate::get_tardis_inst();
        stats_sync_serv::fact_col_record_sync(&fact_key.0, &col_key.0, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }
}
