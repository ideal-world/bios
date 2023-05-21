use std::collections::HashMap;

use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    serde_json::Value,
    tokio, TardisFuns, TardisFunsInst,
};

use crate::iam_config::IamConfig;

pub struct SpiLogClient;

impl SpiLogClient {
    pub async fn add_item(
        tag: String,
        content: String,
        kind: Option<String>,
        search_ext: Option<Value>,
        key: Option<String>,
        op: Option<String>,
        rel_key: Option<String>,
        ts: Option<String>,
        is_async: bool,
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
        if is_async {
            let web_client = funs.web_client();
            tokio::spawn(async move {
                web_client.post_obj_to_str(&format!("{log_url}/ci/item"), &body, headers.clone()).await.unwrap();
            });
        } else {
            funs.web_client().post_obj_to_str(&format!("{log_url}/ci/item"), &body, headers.clone()).await.unwrap();
        }
        Ok(())
    }
}
