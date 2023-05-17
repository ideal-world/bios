use std::collections::HashMap;

use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    serde_json::Value,
    tokio, TardisFuns, TardisFunsInst,
};

use crate::iam_config::IamConfig;

pub struct SpiLogClient;

pub struct LogContent {
    op: String,
    ext: Option<String>,
    user_name: String,
    name: String,
    ip: String,
}

impl SpiLogClient {
    pub async fn add_item(
        tag: String,
        content: LogContent,
        kind: Option<String>,
        key: Option<String>,
        op: Option<String>,
        rel_key: Option<String>,
        ts: Option<String>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<()> {
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
        let mut body = HashMap::from([("tag", tag), ("content", content), ("owner", ctx.owner.clone()), ("owner_paths", ctx.own_paths.clone())]);
        if let Some(kind) = kind {
            body.insert("kind", kind);
        }
        if let Some(search_ext) = search_ext {
            body.insert("search_ext", search_ext.to_string());
        }
        if let Some(key) = key {
            body.insert("key", key);
        }
        if let Some(op) = op {
            body.insert("op", op);
        }
        if let Some(rel_key) = rel_key {
            body.insert("rel_key", rel_key);
        }
        if let Some(ts) = ts {
            body.insert("ts", ts);
        }
        funs.web_client().post_obj_to_str(&format!("{log_url}/ci/item"), &body, headers.clone()).await.unwrap();
        Ok(())
    }
}
