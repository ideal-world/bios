use std::sync::Arc;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use spacegate_shell::hyper::{Request, Response, StatusCode};
use spacegate_shell::kernel::extension::{PeerAddr, Reflect};
use spacegate_shell::kernel::helper_layers::bidirection_filter::{Bdf, BdfLayer, BoxReqFut, BoxRespFut};
use spacegate_shell::plugin::{def_plugin, MakeSgLayer, PluginError};
use spacegate_shell::spacegate_ext_redis::{redis::AsyncCommands, RedisClient};
use spacegate_shell::{SgBody, SgBoxLayer, SgRequestExt, SgResponseExt};

use tardis::{
    basic::result::TardisResult,
    tokio::{self},
};

def_plugin!("anti_replay", AntiReplayPlugin, SgFilterAntiReplay);

#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct SgFilterAntiReplay {
    cache_key: String,
    // millisecond
    time: u64,
}

impl Default for SgFilterAntiReplay {
    fn default() -> Self {
        Self {
            cache_key: "sg:plugin:anti_replay".to_string(),
            time: 5000,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AntiReplayDigest {
    md5: Arc<str>,
    client: RedisClient,
}

impl Bdf for SgFilterAntiReplay {
    type FutureReq = BoxReqFut;
    type FutureResp = BoxRespFut;
    fn on_req(self: Arc<Self>, mut req: Request<SgBody>) -> Self::FutureReq {
        Box::pin(async move {
            if let Some(client) = req.get_redis_client_by_gateway_name() {
                let md5 = get_md5(&req).map_err(PluginError::bad_gateway::<AntiReplayPlugin>)?;
                let digest = AntiReplayDigest {
                    md5: Arc::from(md5),
                    client: client.clone(),
                };
                if get_status(&digest.md5, &self.cache_key, &client).await.map_err(PluginError::bad_gateway::<AntiReplayPlugin>)? {
                    return Err(Response::with_code_message(
                        StatusCode::TOO_MANY_REQUESTS,
                        "[SG.Plugin.Anti_Replay] Request denied due to replay attack. Please refresh and resubmit the request.",
                    ));
                } else {
                    set_status(&digest.md5, &self.cache_key, true, &client).await.map_err(PluginError::bad_gateway::<AntiReplayPlugin>)?;
                }
                req.extensions_mut().get_mut::<Reflect>().expect("missing reflect").insert(digest);
            }
            Ok(req)
        })
    }
    fn on_resp(self: Arc<Self>, resp: Response<SgBody>) -> Self::FutureResp {
        Box::pin(async move {
            if let Some(digest) = resp.extensions().get::<AntiReplayDigest>() {
                let digest = digest.clone();
                tokio::spawn(async move {
                    tokio::time::sleep(Duration::from_millis(self.time)).await;
                    let _ = set_status(&digest.md5, self.cache_key.as_ref(), false, &digest.client).await;
                });
            }
            resp
        })
    }
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

async fn set_status(md5: &str, cache_key: &str, status: bool, cache_client: &RedisClient) -> TardisResult<()> {
    let (split1, split2) = md5.split_at(16);
    let split1 = u128::from_str_radix(split1, 16)? as u32;
    let split2 = u128::from_str_radix(split2, 16)? as u32;
    let mut conn = cache_client.get_conn().await;
    conn.setbit(&format!("{cache_key}:1"), split1 as usize, status).await?;
    conn.setbit(&format!("{cache_key}:2"), split2 as usize, status).await?;
    Ok(())
}

async fn get_status(md5: &str, cache_key: &str, cache_client: &RedisClient) -> TardisResult<bool> {
    let (split1, split2) = md5.split_at(16);
    let split1 = u128::from_str_radix(split1, 16)? as u32;
    let split2 = u128::from_str_radix(split2, 16)? as u32;
    let mut conn = cache_client.get_conn().await;
    let status1 = conn.getbit(&format!("{cache_key}:1"), split1 as usize).await?;
    let status2 = conn.getbit(&format!("{cache_key}:2"), split2 as usize).await?;
    Ok(status1 && status2)
}

impl MakeSgLayer for SgFilterAntiReplay {
    fn make_layer(&self) -> Result<spacegate_shell::SgBoxLayer, spacegate_shell::BoxError> {
        Ok(SgBoxLayer::new(BdfLayer::new(self.clone())))
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
//         let docker = testcontainers::clients::Cli::default();
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
