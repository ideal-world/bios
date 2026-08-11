use serde::{Deserialize, Serialize};
use tardis::web::poem_openapi;

/// 分批推送业务 key 列表请求
///
/// 业务服务分批推送业务 key，Search 库接收后写入临时对账表 `tmp_sync_keys`。
/// 一次同步覆盖一个 tag+kind 维度在该 schema 下的全部数据，无需 own_paths 维度。
#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct SearchSyncBatchReq {
    /// 本次同步唯一标识
    pub sync_batch_id: String,
    /// Search 表 tag（对应 `search_{tag}` 表）
    #[oai(validator(pattern = r"^[a-z0-9-_]+$"))]
    pub tag: String,
    /// IDP 数据类型（对应 search 行 kind）
    #[oai(validator(min_length = "2"))]
    pub kind: String,
    /// 本批业务 key 列表（对应 search 行 key），单批数量 ≤ 5000
    pub keys: Vec<String>,
}

/// 同步完成请求（批次收尾）
///
/// 业务服务多次调用 sync/batch 推送完全部 key 后，调用 sync/finish 结束推送阶段；
/// 该接口返回已推送 key 数量（落盘确认）与对账 Diff 结果。
#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct SearchSyncFinishReq {
    /// 本次同步唯一标识
    pub sync_batch_id: String,
    /// Search 表 tag（对应 `search_{tag}` 表）
    #[oai(validator(pattern = r"^[a-z0-9-_]+$"))]
    pub tag: String,
    /// 数据类型（对应 search 行 kind）
    #[oai(validator(min_length = "2"))]
    pub kind: String,
}

/// 同步完成响应（落盘确认 + 对账 Diff）
///
/// spi-search 仅做差异比对，不执行删除/写入；具体操作由业务服务调用
/// `batch_delete` / `batch_save` 完成。
#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Default)]
pub struct SearchSyncFinishResp {
    /// 本次同步已写入临时对账表的业务 key 数量
    pub total: i64,
    /// Search 库有、本次同步无的冗余 key 列表（业务服务据此调用 batch_delete）
    pub deleted_keys: Vec<String>,
    /// 本次同步有、Search 库无的缺失 key 列表（业务服务据此调用 batch_save 补推）
    pub missing_keys: Vec<String>,
}
