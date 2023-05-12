use std::collections::HashMap;

use serde::Serialize;
use tardis::{TardisFunsInst, basic::{dto::TardisContext, result::TardisResult}, TardisFuns};

use crate::iam_config::IamConfig;

pub struct SpiLogClient;

impl SpiLogClient {
    pub async fn add_item<T: ?Sized + Serialize>(tag: &str, content: &str, key: Option<&str>, op: Option<&str>, rel_key: Option<&str>, ts:Option<&str>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let log_url = funs.conf::<IamConfig>().spi.log_url.clone();
        if log_url.is_empty() {
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

        //add log item
        let mut body = HashMap::from([
            ("tag", tag.to_string()),
            ("content", content.to_string()),
        ]);
        if let Some(key) = key {
            body.insert("key", key.to_string());
        }
        if let Some(op) = op {
            body.insert("op", op.to_string());
        }
        if let Some(rel_key) = rel_key {
            body.insert("rel_key", rel_key.to_string());
        }
        if let Some(ts) = ts {
            body.insert("ts", ts.to_string());
        }
        funs.web_client()
            .post_obj_to_str(
                &format!("{log_url}/ci/item"),
                &body,
                headers.clone(),
            )
            .await
            .unwrap();
        Ok(())
    }
}