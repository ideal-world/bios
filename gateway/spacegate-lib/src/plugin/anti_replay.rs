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
    fn get_code(&self) -> &str {
        CODE
    }

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
        let cache = ctx.cache().await?;
        if get_status(md5.clone(), &self.cache_key, &cache).await? {
            Err(TardisError::forbidden(
                "[SG.Plugin.Anti_Replay] Request denied due to replay attack. Please refresh and resubmit the request.",
                "",
            ))
        } else {
            set_status(md5, &self.cache_key, true, &cache).await?;
            Ok((true, ctx))
        }
    }

    async fn resp_filter(&self, _: &str, mut ctx: SgRoutePluginContext) -> TardisResult<(bool, SgRoutePluginContext)> {
        let md5 = get_md5(&mut ctx)?;
        let cache_key = self.cache_key.clone();
        let name = ctx.get_gateway_name();
        let time = self.time;
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(time)).await;
            let cache_client = spacegate_kernel::functions::cache_client::get(&name).await.expect("get cache client error!");
            let _ = set_status(md5, &cache_key, false, &cache_client).await;
        });
        Ok((true, ctx))
    }
}

fn get_md5(ctx: &mut SgRoutePluginContext) -> TardisResult<String> {
    let req = &mut ctx.request;
    let data = format!(
        "{}{}{}{}",
        req.get_remote_addr(),
        req.get_uri_raw(),
        req.method.get(),
        req.get_headers_raw().iter().fold(String::new(), |mut c, (key, value)| {
            c.push_str(key.as_str());
            c.push_str(&String::from_utf8_lossy(value.as_bytes()));
            c
        }),
    );
    tardis::crypto::crypto_digest::TardisCryptoDigest {}.md5(data)
}

async fn set_status(md5: String, cache_key: &str, status: bool, cache_client: impl AsRef<TardisCacheClient>) -> TardisResult<()> {
    let (split1, split2) = md5.split_at(16);
    let split1 = u128::from_str_radix(split1, 16)? as u32;
    let split2 = u128::from_str_radix(split2, 16)? as u32;
    cache_client.as_ref().setbit(&format!("{cache_key}:1"), split1 as usize, status).await?;
    cache_client.as_ref().setbit(&format!("{cache_key}:2"), split2 as usize, status).await?;
    Ok(())
}

async fn get_status(md5: String, cache_key: &str, cache_client: impl AsRef<TardisCacheClient>) -> TardisResult<bool> {
    let (split1, split2) = md5.split_at(16);
    let split1 = u128::from_str_radix(split1, 16)? as u32;
    let split2 = u128::from_str_radix(split2, 16)? as u32;
    let status1 = cache_client.as_ref().getbit(&format!("{cache_key}:1"), split1 as usize).await?;
    let status2 = cache_client.as_ref().getbit(&format!("{cache_key}:2"), split2 as usize).await?;
    Ok(status1 && status2)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {

    use std::env;

    use spacegate_kernel::{
        http::Uri,
        hyper::{Body, HeaderMap, Method, Version},
    };
    use tardis::{
        test::test_container::TardisTestContainer,
        testcontainers::{self, clients::Cli, Container},
    };
    use testcontainers_modules::redis::Redis;
    use super::*;

    #[tokio::test]
    async fn test_anti_replay() {
        let docker = testcontainers::clients::Cli::default();
        let _x = docker_init(&docker).await.unwrap();
        let gateway_name = "gateway_aaa_1";
        spacegate_kernel::functions::cache_client::init(gateway_name, &env::var("TARDIS_FW.CACHE.URL").unwrap()).await.unwrap();

        let sg_filter_anti_replay = SgFilterAntiReplay { ..Default::default() };
        let ctx = SgRoutePluginContext::new_http(
            Method::POST,
            Uri::from_static("http://sg.idealworld.group/test1"),
            Version::HTTP_11,
            HeaderMap::new(),
            Body::from("test"),
            "127.0.0.1:8080".parse().unwrap(),
            gateway_name.to_string(),
            None,
        );
        let first_req = sg_filter_anti_replay.req_filter("", ctx).await;
        assert!(first_req.is_ok());
        assert!(first_req.as_ref().unwrap().0);
        sg_filter_anti_replay.resp_filter("", first_req.unwrap().1).await.unwrap();
        let ctx = SgRoutePluginContext::new_http(
            Method::POST,
            Uri::from_static("http://sg.idealworld.group/test1"),
            Version::HTTP_11,
            HeaderMap::new(),
            Body::from("test"),
            "127.0.0.1:8080".parse().unwrap(),
            gateway_name.to_string(),
            None,
        );
        assert!(sg_filter_anti_replay.req_filter("", ctx).await.is_err());
        let ctx = SgRoutePluginContext::new_http(
            Method::POST,
            Uri::from_static("http://sg.idealworld.group/test1"),
            Version::HTTP_11,
            HeaderMap::new(),
            Body::from("test"),
            "192.168.1.1:8080".parse().unwrap(),
            gateway_name.to_string(),
            None,
        );
        assert!(sg_filter_anti_replay.req_filter("", ctx).await.is_ok());
        tokio::time::sleep(Duration::from_millis(sg_filter_anti_replay.time)).await;
        let ctx = SgRoutePluginContext::new_http(
            Method::POST,
            Uri::from_static("http://sg.idealworld.group/test1"),
            Version::HTTP_11,
            HeaderMap::new(),
            Body::from("test"),
            "127.0.0.1:8080".parse().unwrap(),
            gateway_name.to_string(),
            None,
        );
        assert!(sg_filter_anti_replay.req_filter("", ctx).await.is_ok());
    }

    pub struct LifeHold<'a> {
        pub redis: Container<'a, Redis>,
    }

    async fn docker_init(docker: &Cli) -> TardisResult<LifeHold<'_>> {
        let redis_container = TardisTestContainer::redis_custom(docker);
        let port = redis_container.get_host_port_ipv4(6379);
        let url = format!("redis://127.0.0.1:{port}/0",);
        env::set_var("TARDIS_FW.CACHE.URL", url);

        Ok(LifeHold { redis: redis_container })
    }
}
