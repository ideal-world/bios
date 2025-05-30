use tardis::basic::dto::TardisContext;
use tardis::chrono::{DateTime, Utc};
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem::web::Query;

use tardis::log::error;
use tardis::tokio;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::param::Path;
use tardis::web::poem_openapi::payload::Json;
use tardis::web::web_resp::{TardisApiResult, TardisPage, TardisResp, Void};

use crate::dto::search_item_dto::{
    GroupSearchItemSearchReq, GroupSearchItemSearchResp, SearchExportDataReq, SearchExportDataResp, SearchImportDataReq, SearchItemAddReq, SearchItemModifyReq,
    SearchItemSearchReq, SearchItemSearchResp, SearchQueryMetricsReq, SearchQueryMetricsResp,
};
use crate::serv::search_item_serv;

#[derive(Clone)]
pub struct SearchCiItemApi;

/// Interface Console Search API
#[poem_openapi::OpenApi(prefix_path = "/ci/item", tag = "bios_basic::ApiTag::Interface")]
impl SearchCiItemApi {
    /// Add Item
    #[oai(path = "/", method = "put")]
    async fn add(&self, mut add_req: Json<SearchItemAddReq>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let funs = crate::get_tardis_inst();
        search_item_serv::add(&mut add_req.0, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    /// Modify Item
    #[oai(path = "/:tag/:key", method = "put")]
    async fn modify(&self, tag: Path<String>, key: Path<String>, mut modify_req: Json<SearchItemModifyReq>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let funs = crate::get_tardis_inst();
        search_item_serv::modify(&tag.0, &key.0, &mut modify_req.0, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    /// Delete Item
    #[oai(path = "/:tag/:key", method = "delete")]
    async fn delete(&self, tag: Path<String>, key: Path<String>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let funs = crate::get_tardis_inst();
        search_item_serv::delete(&tag.0, &key.0, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    /// 删除单项资源
    /// 注意:此操作将物理删除所有事实记录，且无法恢复，请谨慎使用!
    /// Delete Item
    /// Note:This operation will physically delete all fact records and cannot be recovered, please use caution!
    #[oai(path = "/:tag/ownership", method = "delete")]
    async fn delete_by_ownership(&self, tag: Path<String>, own_paths: Query<String>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let funs = crate::get_tardis_inst();
        search_item_serv::delete_by_ownership(&tag.0, &own_paths.0, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    /// Search Items
    #[oai(path = "/search", method = "put")]
    async fn search(&self, mut search_req: Json<SearchItemSearchReq>, ctx: TardisContextExtractor) -> TardisApiResult<TardisPage<SearchItemSearchResp>> {
        let funs = crate::get_tardis_inst();
        let resp = search_item_serv::search(&mut search_req.0, &funs, &ctx.0).await?;
        TardisResp::ok(resp)
    }

    ///group Search Items
    #[oai(path = "/group/search", method = "put")]
    async fn group_search(&self, mut search_req: Json<GroupSearchItemSearchReq>, ctx: TardisContextExtractor) -> TardisApiResult<Vec<GroupSearchItemSearchResp>> {
        let funs = crate::get_tardis_inst();
        let resp = search_item_serv::group_search(&mut search_req.0, &funs, &ctx.0).await?;
        TardisResp::ok(resp)
    }

    /// Query Metrics
    #[oai(path = "/metrics", method = "put")]
    async fn query_metrics(&self, query_req: Json<SearchQueryMetricsReq>, ctx: TardisContextExtractor) -> TardisApiResult<SearchQueryMetricsResp> {
        let funs = crate::get_tardis_inst();
        let resp = search_item_serv::query_metrics(&query_req.0, &funs, &ctx.0).await?;
        TardisResp::ok(resp)
    }

    /// Refresh TSV Result By Tag
    ///
    /// 通过指定 tag 刷新分词结果
    #[oai(path = "/:tag/refresh", method = "put")]
    async fn refresh_tsv(&self, tag: Path<String>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let global_ctx = TardisContext {
            own_paths: "".to_string(),
            ..ctx.0.clone()
        };
        tokio::spawn(async move {
            let funs = crate::get_tardis_inst();
            let result = search_item_serv::refresh_tsv(&tag.0, &funs, &global_ctx).await;
            if let Err(err) = result {
                error!("[BIOS.Search] failed to refresh: {}", err);
            }
        });

        TardisResp::ok(Void {})
    }

    #[oai(path = "/export", method = "put")]
    async fn export_data(&self, export_req: Json<SearchExportDataReq>, ctx: TardisContextExtractor) -> TardisApiResult<SearchExportDataResp> {
        let funs = crate::get_tardis_inst();
        let result = search_item_serv::export_data(&export_req.0, &funs, &ctx.0).await?;
        TardisResp::ok(result)
    }

    #[oai(path = "/import", method = "put")]
    async fn import_data(&self, import_req: Json<SearchImportDataReq>, ctx: TardisContextExtractor) -> TardisApiResult<bool> {
        let funs = crate::get_tardis_inst();
        let result = search_item_serv::import_data(&import_req.0, &funs, &ctx.0).await?;
        TardisResp::ok(result)
    }
}
