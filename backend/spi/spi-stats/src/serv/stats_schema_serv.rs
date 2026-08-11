use bios_basic::spi::{spi_constants, spi_funs::SpiBsInstExtractor};
use tardis::basic::{dto::TardisContext, error::TardisError, result::TardisResult};
use tardis::TardisFunsInst;

use crate::{
    dto::stats_schema_dto::StatsMdlExportResp,
    stats_initializer,
};

use super::pg;

/// Export MDL（Metric Definition Language）model
///
/// 导出 MDL（指标定义语言）语义层模型（设计文档 §4.3）
pub async fn mdl_export(fact_conf_key: Option<String>, since: Option<String>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<StatsMdlExportResp> {
    let inst = funs.init(None, ctx, true, stats_initializer::init_fun).await?;
    match inst.kind_code() {
        #[cfg(feature = "spi-pg")]
        spi_constants::SPI_PG_KIND_CODE => pg::stats_pg_schema_serv::mdl_export(fact_conf_key, since, funs, ctx, &inst).await,
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
}

/// Describe the statistics semantic layer as AI readable JSON（设计文档 §4.3 P1）
///
/// 以 AI 可读的 JSON 描述统计语义层
pub async fn describe(funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<serde_json::Value> {
    let inst = funs.init(None, ctx, true, stats_initializer::init_fun).await?;
    match inst.kind_code() {
        #[cfg(feature = "spi-pg")]
        spi_constants::SPI_PG_KIND_CODE => pg::stats_pg_schema_serv::describe(funs, ctx, &inst).await,
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
}

/// Pack all exported models into an in-memory zip archive（供文件下载接口使用）。
///
/// 将导出的全部模型打包为 zip 字节（每个模型一个 `{物理表名}.yml` 文件）
pub fn mdl_models_zip(resp: &StatsMdlExportResp) -> TardisResult<Vec<u8>> {
    use std::io::Write;

    let mut buf = Vec::new();
    {
        let mut zip_writer = zip::ZipWriter::new(std::io::Cursor::new(&mut buf));
        let options = zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
        for model in &resp.models {
            zip_writer
                .start_file(format!("{}.yml", model.fact_key), options)
                .map_err(|err| TardisError::internal_error(&format!("zip start_file failed: {err}"), "500-spi-stats-mdl-zip"))?;
            zip_writer
                .write_all(model.content.as_bytes())
                .map_err(|err| TardisError::internal_error(&format!("zip write failed: {err}"), "500-spi-stats-mdl-zip"))?;
        }
        zip_writer
            .finish()
            .map_err(|err| TardisError::internal_error(&format!("zip finish failed: {err}"), "500-spi-stats-mdl-zip"))?;
    }
    Ok(buf)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dto::stats_schema_dto::StatsMdlExportModel;

    #[test]
    fn test_mdl_models_zip() {
        let resp = StatsMdlExportResp {
            models: vec![
                StatsMdlExportModel {
                    fact_key: "starsys_stats_inst_fact_demo_a".to_string(),
                    content: "name: starsys_stats_inst_fact_demo_a\n".to_string(),
                },
                StatsMdlExportModel {
                    fact_key: "starsys_stats_inst_fact_demo_b".to_string(),
                    content: "name: starsys_stats_inst_fact_demo_b\n".to_string(),
                },
            ],
        };
        let bytes = mdl_models_zip(&resp).expect("zip bytes");
        let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes)).expect("open zip archive");
        assert_eq!(archive.len(), 2);
        let mut names = vec![];
        for index in 0..archive.len() {
            let file = archive.by_index(index).expect("zip entry");
            names.push(file.name().to_string());
        }
        assert!(names.contains(&"starsys_stats_inst_fact_demo_a.yml".to_string()));
        assert!(names.contains(&"starsys_stats_inst_fact_demo_b.yml".to_string()));
    }
}
