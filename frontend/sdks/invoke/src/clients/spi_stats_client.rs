use serde::{Deserialize, Serialize};
use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
#[cfg(feature = "event")]
use tardis::futures::TryFutureExt as _;
use tardis::TardisFunsInst;

use crate::dto::stats_record_dto::{StatsFactRecordLoadReq, StatsFactRecordsLoadReq};
#[cfg(feature = "event")]
use crate::invoke_config::InvokeConfigApi as _;
use crate::invoke_enumeration::InvokeModuleKind;

use super::base_spi_client::BaseSpiClient;
#[cfg(feature = "event")]
use super::event_client::{get_topic, mq_error, EventAttributeExt as _, SPI_RPC_TOPIC};

#[cfg(feature = "event")]
pub mod event {
    use asteroid_mq::prelude::*;

    impl EventAttribute for super::StatsItemAddReq {
        const SUBJECT: Subject = Subject::const_new("stats/add");
    }
    impl EventAttribute for super::StatsItemAddsReq {
        const SUBJECT: Subject = Subject::const_new("stats/adds");
    }
    impl EventAttribute for super::StatsItemDeleteReq {
        const SUBJECT: Subject = Subject::const_new("stats/delete");
    }
}

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
        #[cfg(feature = "event")]
        if funs.invoke_conf_in_event(InvokeModuleKind::Stats) {
            if let Some(topic) = get_topic(&SPI_RPC_TOPIC) {
                topic
                    .send_event(
                        StatsItemAddReq {
                            fact_key: fact_key.to_string(),
                            record_key: record_key.to_string(),
                            req: add_req,
                        }
                        .inject_context(funs, ctx)
                        .json(),
                    )
                    .map_err(mq_error)
                    .await?;
                return Ok(());
            }
        }

        let stats_url: String = BaseSpiClient::module_url(InvokeModuleKind::Stats, funs).await?;
        let headers = BaseSpiClient::headers(None, funs, ctx).await?;
        funs.web_client().put_obj_to_str(&format!("{stats_url}/ci/record/fact/{fact_key}/{record_key}"), &add_req, headers.clone()).await?;
        Ok(())
    }

    pub async fn fact_records_load(fact_key: &str, add_req: Vec<StatsFactRecordsLoadReq>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        #[cfg(feature = "event")]
        if funs.invoke_conf_in_event(InvokeModuleKind::Stats) {
            if let Some(topic) = get_topic(&SPI_RPC_TOPIC) {
                topic
                    .send_event(
                        StatsItemAddsReq {
                            fact_key: fact_key.to_string(),
                            reqs: add_req,
                        }
                        .inject_context(funs, ctx)
                        .json(),
                    )
                    .map_err(mq_error)
                    .await?;
                return Ok(());
            }
        }

        let stats_url: String = BaseSpiClient::module_url(InvokeModuleKind::Stats, funs).await?;
        let headers = BaseSpiClient::headers(None, funs, ctx).await?;
        funs.web_client().put_obj_to_str(&format!("{stats_url}/ci/record/fact/{fact_key}/batch/load"), &add_req, headers.clone()).await?;
        Ok(())
    }

    pub async fn fact_record_delete(fact_key: &str, record_key: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        #[cfg(feature = "event")]
        if funs.invoke_conf_in_event(InvokeModuleKind::Stats) {
            if let Some(topic) = get_topic(&SPI_RPC_TOPIC) {
                topic
                    .send_event(
                        StatsItemDeleteReq {
                            fact_key: fact_key.to_string(),
                            record_key: record_key.to_string(),
                        }
                        .inject_context(funs, ctx)
                        .json(),
                    )
                    .map_err(mq_error)
                    .await?;
                return Ok(());
            }
        }

        let stats_url = BaseSpiClient::module_url(InvokeModuleKind::Stats, funs).await?;
        let headers = BaseSpiClient::headers(None, funs, ctx).await?;
        funs.web_client().delete_to_void(&format!("{stats_url}/ci/record/fact/{fact_key}/{record_key}"), headers.clone()).await?;
        Ok(())
    }
}
