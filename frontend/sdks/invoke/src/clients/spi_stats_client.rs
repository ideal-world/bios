use serde::{Deserialize, Serialize};
use tardis::basic::dto::TardisContext;
use tardis::basic::error::TardisError;
use tardis::basic::result::TardisResult;
use tardis::TardisFunsInst;

use crate::dto::stats_record_dto::{StatsFactRecordLoadReq, StatsFactRecordsLoadReq};
use crate::invoke_config::InvokeConfigApi;
use crate::invoke_enumeration::InvokeModuleKind;

use super::base_spi_client::{BaseSpiClient, SpiBsAddReq};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StatsItemAddReq {
    pub fact_key: String,
    pub record_key: String,
    pub req: StatsFactRecordLoadReq,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StatsItemAddsReq {
    pub fact_key: String,
    pub reqs: Vec<StatsFactRecordsLoadReq>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StatsItemDeleteReq {
    pub fact_key: String,
    pub record_key: String,
}

pub struct SpiStatsClient;

impl SpiStatsClient {
    /// Initialize the Stats backend service: create if not exists and bind to app/tenant.
    /// Reads all configuration from `InvokeModuleConfig.bs_init`.
    ///
    /// 初始化 Stats 后端服务：不存在则创建，并绑定到应用/租户。配置均来自 invoke 配置，返回后端服务id。
    pub async fn init(funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<String> {
        let init_cfg = funs
            .invoke_conf_module_bs_init(InvokeModuleKind::Stats)
            .ok_or_else(|| TardisError::bad_request("stats module bs_init config not set", ""))?;
        let module_url = BaseSpiClient::module_url(InvokeModuleKind::Stats, funs).await?;
        let bs_add_req = SpiBsAddReq {
            name: init_cfg.bs_name.clone(),
            kind_id: init_cfg.bs_kind_id.clone(),
            conn_uri: init_cfg.bs_conn_uri.clone(),
            ak: init_cfg.bs_ak.clone(),
            sk: init_cfg.bs_sk.clone(),
            ext: init_cfg.bs_ext.clone(),
            private: init_cfg.bs_private,
            disabled: init_cfg.bs_disabled,
        };
        let bs_id = BaseSpiClient::add_bs_if_not_exist(&module_url, &bs_add_req, funs, ctx).await?;
        BaseSpiClient::add_bs_rel_if_not_exist(&module_url, &bs_id, &init_cfg.app_tenant_id, funs, ctx).await?;
        Ok(bs_id)
    }

    pub async fn fact_record_load(fact_key: &str, record_key: &str, add_req: StatsFactRecordLoadReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let stats_url: String = BaseSpiClient::module_url(InvokeModuleKind::Stats, funs).await?;
        let headers = BaseSpiClient::headers(None, funs, ctx).await?;
        funs.web_client().put_obj_to_str(&format!("{stats_url}/ci/record/fact/{fact_key}/{record_key}"), &add_req, headers.clone()).await?;
        Ok(())
    }

    pub async fn fact_records_load(fact_key: &str, add_req: Vec<StatsFactRecordsLoadReq>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let stats_url: String = BaseSpiClient::module_url(InvokeModuleKind::Stats, funs).await?;
        let headers = BaseSpiClient::headers(None, funs, ctx).await?;
        funs.web_client().put_obj_to_str(&format!("{stats_url}/ci/record/fact/{fact_key}/batch/load"), &add_req, headers.clone()).await?;
        Ok(())
    }

    pub async fn fact_record_delete(fact_key: &str, record_key: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let stats_url = BaseSpiClient::module_url(InvokeModuleKind::Stats, funs).await?;
        let headers = BaseSpiClient::headers(None, funs, ctx).await?;
        funs.web_client().delete_to_void(&format!("{stats_url}/ci/record/fact/{fact_key}/{record_key}"), headers.clone()).await?;
        Ok(())
    }
}
