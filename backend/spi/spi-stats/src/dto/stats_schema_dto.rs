use serde::{Deserialize, Serialize};
use tardis::web::poem_openapi;

/// MDL Export Response Object（全量/增量导出契约）
///
/// 语义层 MDL 导出响应对象（设计文档 §4.3）
///
/// ```json
/// { "models": [ { "fact_key": "req", "content": "<model YAML>" }, ... ] }
/// ```
#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct StatsMdlExportResp {
    /// Exported model list，按 fact_key 命名，content 为单个 model 的 YAML 文本
    ///
    /// 导出的模型列表，按 fact_key 命名，content 为单个 model 的 YAML 文本
    pub models: Vec<StatsMdlExportModel>,
}

/// MDL Export Model Item
///
/// MDL 导出模型条目
#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct StatsMdlExportModel {
    /// Physical fact table name（= MDL model name，同时用作 sync-mdl.sh 落盘的文件名，如 `starsys_stats_inst_fact_req`）
    ///
    /// 物理事实表名（= MDL model name，同时用作 sync-mdl.sh 落盘的文件名，如 `starsys_stats_inst_fact_req`）
    pub fact_key: String,
    /// Single model YAML content（WrenAI 逐模型格式：name/description/keywords/table_reference/primary_key/columns）
    ///
    /// 单个 model 的 YAML 内容（WrenAI 逐模型格式：name/description/keywords/table_reference/primary_key/columns）
    pub content: String,
}
