use serde::{Deserialize, Serialize};
use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::TardisFunsInst;

use crate::dto::stats_record_dto::{StatsFactRecordLoadReq, StatsFactRecordsLoadReq};
use crate::invoke_enumeration::InvokeModuleKind;

use super::base_spi_client::BaseSpiClient;

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
