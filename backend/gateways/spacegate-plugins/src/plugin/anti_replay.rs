use std::sync::Arc;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use spacegate_shell::ext_redis::{redis::AsyncCommands, RedisClient};
use spacegate_shell::hyper::{Request, Response, StatusCode};
use spacegate_shell::kernel::extension::PeerAddr;
use spacegate_shell::kernel::helper_layers::function::Inner;
use spacegate_shell::plugin::{schemars, Plugin, PluginError};
use spacegate_shell::{BoxError, BoxResult, SgBody, SgRequestExt, SgResponseExt};

use tardis::serde_json;
use tardis::{basic::result::TardisResult, tokio};

spacegate_shell::plugin::schema!(AntiReplayPlugin, AntiReplayPlugin);
#[derive(Serialize, Deserialize, Clone, schemars::JsonSchema)]
#[serde(default)]
pub struct AntiReplayPlugin {
    cache_key: String,
    // millisecond
    time: u64,
}

impl Default for AntiReplayPlugin {
    fn default() -> Self {
        Self {
            cache_key: "sg:plugin:anti_replay".into(),
            time: 5000,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AntiReplayDigest {
    md5: Arc<str>,
    client: RedisClient,
}

fn get_md5(req: &Request<SgBody>) -> TardisResult<String> {
    let remote_addr = req.extensions().get::<PeerAddr>().expect("missing peer address").0;
    let uri = req.uri();
    let method = req.method();

    let data = format!(
        "{}{}{}{}",
        remote_addr,
        uri,
        method,
        req.headers().iter().fold(String::new(), |mut c, (key, value)| {
            c.push_str(key.as_str());
            c.push_str(&String::from_utf8_lossy(value.as_bytes()));
            c
        }),
    );
    tardis::crypto::crypto_digest::TardisCryptoDigest {}.md5(data)
}

async fn set_status(md5: &str, cache_key: &str, status: bool, cache_client: &RedisClient) -> BoxResult<()> {
    let (split1, split2) = md5.split_at(16);
    let split1 = u128::from_str_radix(split1, 16)? as u32;
    let split2 = u128::from_str_radix(split2, 16)? as u32;
    let mut conn = cache_client.get_conn().await;
    conn.setbit(&format!("{cache_key}:1"), split1 as usize, status).await?;
    conn.setbit(&format!("{cache_key}:2"), split2 as usize, status).await?;
    Ok(())
}

async fn get_status(md5: &str, cache_key: &str, cache_client: &RedisClient) -> BoxResult<bool> {
    let (split1, split2) = md5.split_at(16);
    let split1 = u128::from_str_radix(split1, 16)? as u32;
    let split2 = u128::from_str_radix(split2, 16)? as u32;
    let mut conn = cache_client.get_conn().await;
    let status1 = conn.getbit(&format!("{cache_key}:1"), split1 as usize).await?;
    let status2 = conn.getbit(&format!("{cache_key}:2"), split2 as usize).await?;
    Ok(status1 && status2)
}

impl Plugin for AntiReplayPlugin {
    const CODE: &'static str = "anti-replay";

    fn meta() -> spacegate_shell::model::PluginMetaData {
        spacegate_shell::model::plugin_meta!(
            description: "Anti-replay plugin for Spacegate. It can prevent replay attacks by checking the MD5 hash of the request."
        )
    }

    fn create(plugin_config: spacegate_shell::plugin::PluginConfig) -> Result<Self, BoxError> {
        let config: AntiReplayPlugin = serde_json::from_value(plugin_config.spec)?;
        Ok(config)
    }
    async fn call(&self, req: Request<SgBody>, inner: Inner) -> Result<Response<SgBody>, BoxError> {
        if let Some(client) = req.get_redis_client_by_gateway_name() {
            let md5 = get_md5(&req).map_err(PluginError::internal_error::<AntiReplayPlugin>)?;
            let digest = AntiReplayDigest {
                md5: Arc::from(md5),
                client: client.clone(),
            };
            if get_status(&digest.md5, &self.cache_key, &client).await? {
                return Ok(Response::with_code_message(
                    StatusCode::TOO_MANY_REQUESTS,
                    "[SG.Plugin.Anti_Replay] Request denied due to replay attack. Please refresh and resubmit the request.",
                ));
            } else {
                set_status(&digest.md5, &self.cache_key, true, &client).await?;
            }
            let resp = inner.call(req).await;
            let time = self.time;
            let cache_key = self.cache_key.clone();
            tokio::spawn(async move {
                tokio::time::sleep(Duration::from_millis(time)).await;
                let _ = set_status(&digest.md5, cache_key.as_ref(), false, &digest.client).await;
            });
            Ok(resp)
        } else {
            let resp = inner.call(req).await;
            Ok(resp)
        }
    }
}

// #[cfg(test)]
// #[allow(clippy::unwrap_used)]
// mod tests {

//     use std::env;

//     use super::*;
//     use spacegate_shell::{
//         http::Uri,
//         hyper::{Body, HeaderMap, Method, Version},
//     };
//     use tardis::{
//         test::test_container::TardisTestContainer,
//         testcontainers::{self, clients::Cli, Container},
//     };
//     use testcontainers_modules::redis::Redis;

//     #[tokio::test]
//     async fn test_anti_replay() {
//         let _x = docker_init(&docker).await.unwrap();
//         let gateway_name = "gateway_aaa_1";
//         spacegate_shell::functions::cache_client::init(gateway_name, &env::var("TARDIS_FW.CACHE.URL").unwrap()).await.unwrap();

//         let sg_filter_anti_replay = SgFilterAntiReplay { ..Default::default() };
//         let ctx = SgRoutePluginContext::new_http(
//             Method::POST,
//             Uri::from_static("http://sg.idealworld.group/test1"),
//             Version::HTTP_11,
//             HeaderMap::new(),
//             Body::from("test"),
//             "127.0.0.1:8080".parse().unwrap(),
//             gateway_name.to_string(),
//             None,
//         );
//         let first_req = sg_filter_anti_replay.req_filter("", ctx).await;
//         assert!(first_req.is_ok());
//         assert!(first_req.as_ref().unwrap().0);
//         sg_filter_anti_replay.resp_filter("", first_req.unwrap().1).await.unwrap();
//         let ctx = SgRoutePluginContext::new_http(
//             Method::POST,
//             Uri::from_static("http://sg.idealworld.group/test1"),
//             Version::HTTP_11,
//             HeaderMap::new(),
//             Body::from("test"),
//             "127.0.0.1:8080".parse().unwrap(),
//             gateway_name.to_string(),
//             None,
//         );
//         assert!(sg_filter_anti_replay.req_filter("", ctx).await.is_err());
//         let ctx = SgRoutePluginContext::new_http(
//             Method::POST,
//             Uri::from_static("http://sg.idealworld.group/test1"),
//             Version::HTTP_11,
//             HeaderMap::new(),
//             Body::from("test"),
//             "192.168.1.1:8080".parse().unwrap(),
//             gateway_name.to_string(),
//             None,
//         );
//         assert!(sg_filter_anti_replay.req_filter("", ctx).await.is_ok());
//         tokio::time::sleep(Duration::from_millis(sg_filter_anti_replay.time)).await;
//         let ctx = SgRoutePluginContext::new_http(
//             Method::POST,
//             Uri::from_static("http://sg.idealworld.group/test1"),
//             Version::HTTP_11,
//             HeaderMap::new(),
//             Body::from("test"),
//             "127.0.0.1:8080".parse().unwrap(),
//             gateway_name.to_string(),
//             None,
//         );
//         assert!(sg_filter_anti_replay.req_filter("", ctx).await.is_ok());
//     }

//     pub struct LifeHold<'a> {
//         pub redis: Container<'a, Redis>,
//     }

//     async fn docker_init(docker: &Cli) -> TardisResult<LifeHold<'_>> {
//         let redis_container = TardisTestContainer::redis_custom(docker);
//         let port = redis_container.get_host_port_ipv4(6379);
//         let url = format!("redis://127.0.0.1:{port}/0",);
//         env::set_var("TARDIS_FW.CACHE.URL", url);

//         Ok(LifeHold { redis: redis_container })
//     }
// }
