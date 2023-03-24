use crate::iam_config::IamConfig;
use serde::Serialize;
use std::collections::HashMap;
use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::{TardisFuns, TardisFunsInst};

pub struct SpiKvClient;

impl SpiKvClient {
    pub async fn add_or_modify_item<T: ?Sized + Serialize>(key: &str, value: &T, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let kv_url = funs.conf::<IamConfig>().spi.kv_url.clone();
        if kv_url.is_empty() {
            return Ok(());
        }
        let spi_ctx = TardisContext {
            owner: funs.conf::<IamConfig>().spi.owner.clone(),
            ..ctx.clone()
        };
        let headers = Some(vec![(
            "Tardis-Context".to_string(),
            TardisFuns::crypto.base64.encode(&TardisFuns::json.obj_to_string(&spi_ctx).unwrap()),
        )]);

        //add kv
        funs.web_client()
            .put_obj_to_str(
                &format!("{kv_url}/ci/item"),
                &HashMap::from([("key", key.to_string()), ("value", TardisFuns::json.obj_to_string(value)?)]),
                headers.clone(),
            )
            .await
            .unwrap();
        Ok(())
    }
}
