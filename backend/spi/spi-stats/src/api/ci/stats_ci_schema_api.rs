use std::str::FromStr;

use tardis::{
    basic::error::TardisError,
    web::{
        context_extractor::TardisContextExtractor,
        poem::{self},
        poem_openapi::{
            self,
            param::Query,
            payload::{Attachment, Json, PlainText},
        },
    },
    TardisFuns,
};

use crate::serv::stats_schema_serv;

#[derive(Clone)]
pub struct StatsCiSchemaApi;

/// Interface Console Statistics Schema API
///
/// 统计语义层 Schema（MDL）导出 API（设计文档 §4.3）
#[poem_openapi::OpenApi(prefix_path = "/ci/schema", tag = "bios_basic::ApiTag::Interface")]
impl StatsCiSchemaApi {
    /// Export MDL（Metric Definition Language）Semantic Layer Model
    ///
    /// 导出 MDL（指标定义语言）语义层模型
    ///
    /// - 不带 `fact_key`：导出全部 online 事实，响应为 `{ "models": [ { "fact_key", "content" } ] }`（content 为单个 model 的 YAML 文本）
    /// - 带 `fact_key`：导出单个事实，响应为单个 model 的 YAML 文本
    /// - 带 `since`：增量导出（ISO8601 时间戳），仅返回该时间之后有变更/新增的事实
    #[oai(path = "/mdl-export", method = "get")]
    async fn mdl_export(&self, fact_key: Query<Option<String>>, since: Query<Option<String>>, ctx: TardisContextExtractor) -> poem::Result<PlainText<String>> {
        let funs = crate::get_tardis_inst();
        let fact_key = fact_key.0;
        let resp = stats_schema_serv::mdl_export(fact_key.clone(), since.0, &funs, &ctx.0).await.map_err(tardis_err_to_poem_err)?;
        if fact_key.is_some() {
            // 单事实导出：返回单个 model 的 YAML 文本（fact_key 兼容短 key 或完整物理表名）
            let content = resp.models.into_iter().next().map(|model| model.content).unwrap_or_default();
            return Ok(PlainText(content));
        }
        // 全量/增量导出：返回 { "models": [...] } JSON
        let body = TardisFuns::json.obj_to_string(&resp).map_err(tardis_err_to_poem_err)?;
        Ok(PlainText(body))
    }

    /// Download MDL Semantic Layer Model Files
    ///
    /// 下载 MDL 语义层模型文件：
    /// - 带 `fact_key`：下载单个事实的 model YAML 文件（`{物理表名}.yml`）
    /// - 不带 `fact_key`：下载全部 online 事实模型打包的 zip（`hai-wren-models.zip`）
    #[oai(path = "/mdl-file", method = "get")]
    async fn mdl_file(&self, fact_key: Query<Option<String>>, since: Query<Option<String>>, ctx: TardisContextExtractor) -> poem::Result<Attachment<Vec<u8>>> {
        let funs = crate::get_tardis_inst();
        let fact_key = fact_key.0;
        let resp = stats_schema_serv::mdl_export(fact_key.clone(), since.0, &funs, &ctx.0).await.map_err(tardis_err_to_poem_err)?;
        if fact_key.is_some() {
            // 单事实：下载单个 .yml 文件（fact_key 兼容短 key 或完整物理表名；找不到返回 404）
            let model = resp
                .models
                .into_iter()
                .next()
                .ok_or_else(|| poem::Error::from_string("fact not found", poem::http::StatusCode::NOT_FOUND))?;
            return Ok(Attachment::new(model.content.into_bytes()).filename(format!("{}.yml", model.fact_key)));
        }
        // 全量：打包 zip 下载
        let zip_bytes = stats_schema_serv::mdl_models_zip(&resp).map_err(tardis_err_to_poem_err)?;
        Ok(Attachment::new(zip_bytes).filename("hai-wren-models.zip"))
    }

    /// Describe Statistics Semantic Layer as AI Readable JSON（设计文档 §4.3 P1）
    ///
    /// 以 AI 可读的 JSON 描述统计语义层
    #[oai(path = "/describe", method = "get")]
    async fn describe(&self, ctx: TardisContextExtractor) -> poem::Result<Json<serde_json::Value>> {
        let funs = crate::get_tardis_inst();
        let resp = stats_schema_serv::describe(&funs, &ctx.0).await.map_err(tardis_err_to_poem_err)?;
        Ok(Json(resp))
    }
}

fn tardis_err_to_poem_err(err: TardisError) -> poem::Error {
    let status = poem::http::StatusCode::from_str(&err.code).unwrap_or(poem::http::StatusCode::INTERNAL_SERVER_ERROR);
    poem::Error::from_string(err.message, status)
}
