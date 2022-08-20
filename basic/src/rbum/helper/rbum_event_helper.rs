use std::collections::HashMap;
use std::future::Future;

use serde::{Deserialize, Serialize};
use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::chrono::Utc;
use tardis::TardisFunsInst;

use crate::rbum::rbum_config::RbumConfigApi;

pub async fn try_notify<'a>(table_name: &str, operate: &str, record_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<bool> {
    #[cfg(feature = "with-mq")]
    {
        if funs.rbum_conf_match_event(table_name, operate) {
            funs.mq()
                .request(
                    &funs.rbum_conf_mq_topic_event(),
                    tardis::TardisFuns::json.obj_to_string(&RbumEventMessage {
                        table_name: table_name.to_string(),
                        operate: operate.to_string(),
                        operator: ctx.owner.clone(),
                        record_id: record_id.to_string(),
                        ts: Utc::now().timestamp_millis(),
                    })?,
                    &HashMap::new(),
                )
                .await?;
        }
        Ok(true)
    }
    #[cfg(not(feature = "with-mq"))]
    {
        Ok(false)
    }
}

pub async fn receive<F, T>(fun: F, funs: &TardisFunsInst) -> TardisResult<bool>
where
    F: Fn((HashMap<String, String>, String)) -> T + Send + Sync + 'static,
    T: Future<Output = TardisResult<()>> + Send + 'static,
{
    #[cfg(feature = "with-mq")]
    {
        funs.mq().response(&funs.rbum_conf_mq_topic_event(), fun).await?;
        Ok(true)
    }
    #[cfg(not(feature = "with-mq"))]
    {
        Ok(false)
    }
}

pub fn parse_message(message: String) -> TardisResult<RbumEventMessage> {
    tardis::TardisFuns::json.str_to_obj::<RbumEventMessage>(&message)
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RbumEventMessage {
    pub table_name: String,
    pub operate: String,
    pub operator: String,
    pub record_id: String,
    pub ts: i64,
}
