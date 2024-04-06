use std::collections::HashMap;

use serde::Serialize;
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    serde_json::json,
    TardisFuns,
};

use crate::{auth_config::AuthConfig, auth_constants::DOMAIN_CODE, dto::auth_kernel_dto::AuthReq};

#[derive(Serialize, Default, Debug)]
pub struct LogParamContent {
    pub op: String,
    pub ext: Option<String>,
    pub addr: String,
    pub auth_req: Option<AuthReq>,
}

pub struct SpiLogClient;

impl SpiLogClient {
    pub async fn add_item(content: LogParamContent, key: Option<String>, op: Option<String>, ctx: &TardisContext) -> TardisResult<()> {
        let ts = tardis::chrono::Utc::now().to_rfc3339();

        let log_url = TardisFuns::cs_config::<AuthConfig>(DOMAIN_CODE).spi.log_url.clone();
        let spi_owner = TardisFuns::cs_config::<AuthConfig>(DOMAIN_CODE).spi.owner.clone();
        if log_url.is_empty() || spi_owner.is_empty() {
            return Ok(());
        }
        let spi_ctx = TardisContext { owner: spi_owner, ..ctx.clone() };
        let headers = [("Tardis-Context".to_string(), TardisFuns::crypto.base64.encode(TardisFuns::json.obj_to_string(&spi_ctx)?))];

        //add log item
        let mut body = HashMap::from([
            ("tag", "auth".to_string()),
            ("content", TardisFuns::json.obj_to_string(&content)?),
            ("owner", ctx.owner.clone()),
            ("owner_paths", ctx.own_paths.clone()),
        ]);
        // create search_ext
        let search_ext = json!({
            "ext":content.ext,
            "ts":ts,
            "op":op,
        })
        .to_string();
        body.insert("ext", search_ext);

        if let Some(op) = op {
            body.insert("op", op);
        }

        if let Some(key) = key {
            body.insert("key", key);
        }

        body.insert("ts", ts);

        TardisFuns::web_client().post_obj_to_str(&format!("{log_url}/ci/item"), &body, headers.clone()).await?;
        Ok(())
    }
}
