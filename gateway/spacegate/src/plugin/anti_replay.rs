use std::time::Duration;

use async_trait::async_trait;

use serde::{Deserialize, Serialize};
use spacegate_kernel::plugins::filters::SgPluginFilterInitDto;
use spacegate_kernel::plugins::{
    context::SgRoutePluginContext,
    filters::{BoxSgPluginFilter, SgPluginFilter, SgPluginFilterAccept, SgPluginFilterDef},
};
use tardis::cache::cache_client::TardisCacheClient;

use tardis::{
    async_trait,
    basic::{error::TardisError, result::TardisResult},
    serde_json::{self},
    tokio::{self},
    TardisFuns,
};

pub const CODE: &str = "anti_replay";
pub struct SgFilterAntiReplayDef;

impl SgPluginFilterDef for SgFilterAntiReplayDef {
    fn inst(&self, spec: serde_json::Value) -> TardisResult<BoxSgPluginFilter> {
        let filter = TardisFuns::json.json_to_obj::<SgFilterAntiReplay>(spec)?;
        Ok(filter.boxed())
    }
}

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct SgFilterAntiReplay {
    cache_key: String,
    // millisecond
    time: u64,
}

impl Default for SgFilterAntiReplay {
    fn default() -> Self {
        Self {
            cache_key: "spacegate:cache:plugin:anti_replay".to_string(),
            time: 5000,
        }
    }
}

#[async_trait]
impl SgPluginFilter for SgFilterAntiReplay {
    fn accept(&self) -> SgPluginFilterAccept {
        SgPluginFilterAccept::default()
    }

    async fn init(&mut self, _: &SgPluginFilterInitDto) -> TardisResult<()> {
        Ok(())
    }

    async fn destroy(&self) -> TardisResult<()> {
        Ok(())
    }

    async fn req_filter(&self, _: &str, mut ctx: SgRoutePluginContext) -> TardisResult<(bool, SgRoutePluginContext)> {
        let md5 = get_md5(&mut ctx)?;
        if get_status(md5.clone(), &self.cache_key, ctx.cache()?).await? {
            Err(TardisError::forbidden(
                "[SG.Plugin.Anti_Replay] Request denied due to replay attack. Please refresh and resubmit the request.",
                "",
            ))
        } else {
            set_status(md5, &self.cache_key, true, ctx.cache()?).await?;
            Ok((true, ctx))
        }
    }

    async fn resp_filter(&self, _: &str, mut ctx: SgRoutePluginContext) -> TardisResult<(bool, SgRoutePluginContext)> {
        let md5 = get_md5(&mut ctx)?;
        let cache_key = self.cache_key.clone();
        let name = ctx.get_gateway_name();
        let time = self.time;
        let _ = tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(time)).await;
            let cache_client = spacegate_kernel::functions::cache_client::get(&name).expect("get cache client error!");
            let _ = set_status(md5, &cache_key, false, cache_client).await;
        })
        .await;
        Ok((true, ctx))
    }
}

fn get_md5(ctx: &mut SgRoutePluginContext) -> TardisResult<String> {
    let req = &ctx.request;
    let data = format!(
        "{}{}{}",
        req.get_req_uri_raw(),
        req.get_req_method_raw(),
        req.get_req_headers_raw().iter().map(|h| h.0.as_str().to_owned() + h.1.to_str().unwrap_or_default()).collect::<Vec<String>>().join(""),
    );
    tardis::crypto::crypto_digest::TardisCryptoDigest {}.md5(&data)
}

async fn set_status(md5: String, cache_key: &str, status: bool, cache_client: &TardisCacheClient) -> TardisResult<()> {
    let (split1, split2) = md5.split_at(16);
    let split1 = u128::from_str_radix(split1, 16)? as u32;
    let split2 = u128::from_str_radix(split2, 16)? as u32;
    cache_client.setbit(&format!("{cache_key}:1"), split1 as usize, status).await?;
    cache_client.setbit(&format!("{cache_key}:2"), split2 as usize, status).await?;
    Ok(())
}

async fn get_status(md5: String, cache_key: &str, cache_client: &TardisCacheClient) -> TardisResult<bool> {
    let (split1, split2) = md5.split_at(16);
    let split1 = u128::from_str_radix(split1, 16)? as u32;
    let split2 = u128::from_str_radix(split2, 16)? as u32;
    let status1 = cache_client.getbit(&format!("{cache_key}:1"), split1 as usize).await?;
    let status2 = cache_client.getbit(&format!("{cache_key}:2"), split2 as usize).await?;
    Ok(status1 && status2)
}
