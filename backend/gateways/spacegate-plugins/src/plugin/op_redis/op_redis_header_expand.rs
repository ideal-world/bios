use std::collections::HashMap;

use crate::plugin::op_redis::redis_format_key;
use http::HeaderName;
use serde::{Deserialize, Serialize};
use spacegate_shell::{
    ext_redis::{global_repo, Connection},
    hyper::{self, Request, Response},
    kernel::extension::{GatewayName, MatchedSgRouter},
    model::PluginConfig,
    plugin::{error::code, schemars, Inner, Plugin, PluginError},
    BoxError, SgBody,
};
use tardis::{
    cache::{AsyncCommands, RedisError},
    log::debug,
    serde_json,
};
spacegate_shell::plugin::schema!(OpRedisHeaderExpandPlugin, OpRedisHeaderExpandPlugin);
#[derive(Serialize, Deserialize, Clone, schemars::JsonSchema)]
#[serde(default)]
pub struct OpRedisHeaderExpandPlugin {
    pub cache_prefix_key: String,
    pub header: String,
}

impl Default for OpRedisHeaderExpandPlugin {
    fn default() -> Self {
        Self {
            cache_prefix_key: "sg:plugin:redis-header-expand".to_string(),
            header: "X-Request-ID".to_string(),
        }
    }
}

async fn redis_call(mut conn: Connection, redis_key: String) -> Result<HashMap<String, String>, RedisError> {
    let Result::<String, _>::Ok(header_json) = conn.get(&redis_key).await else {
        debug!("fail to get status with key {redis_key}");
        return Ok(HashMap::new());
    };
    let header_map: HashMap<String, String> = match serde_json::from_str(&header_json) {
        Ok(map) => map,
        Err(e) => {
            debug!("fail to parse header json with key {redis_key}: {}", e);
            return Ok(HashMap::new());
        }
    };
    Ok(header_map)
}

impl Plugin for OpRedisHeaderExpandPlugin {
    const CODE: &'static str = "redis-header-expand";

    fn meta() -> spacegate_shell::model::PluginMetaData {
        spacegate_shell::model::plugin_meta!(
            description: "Build for open platform, Expand request headers based on Redis-stored data."
        )
    }

    fn create(config: PluginConfig) -> Result<Self, BoxError> {
        let config: OpRedisHeaderExpandPlugin = serde_json::from_value(config.spec)?;
        Ok(config)
    }

    async fn call(&self, mut req: Request<SgBody>, inner: Inner) -> Result<Response<SgBody>, BoxError> {
        let _header = self.header.clone();
        let Some(gateway_name) = req.extensions().get::<GatewayName>() else {
            return Err("missing gateway name".into());
        };
        let Some(client) = global_repo().get(gateway_name) else {
            return Err("missing redis client".into());
        };
        let Some(matched) = req.extensions().get::<MatchedSgRouter>() else {
            return Err("missing matched router".into());
        };
        let Some(key) = redis_format_key(&req, matched, &self.header) else {
            return Ok(PluginError::status::<Self, { code::UNAUTHORIZED }>(format!("missing header {}", self.header.as_str())).into());
        };
        let header_map: HashMap<String, String> = redis_call(client.get_conn().await, format!("{}:{}", self.cache_prefix_key, key)).await?;
        req.headers_mut().extend(
            header_map
                .into_iter()
                .filter_map(|(k, v)| {
                    let name = HeaderName::from_bytes(k.as_bytes()).ok()?;
                    Some((name, v.parse().ok()?))
                })
                .collect::<Vec<(HeaderName, hyper::header::HeaderValue)>>(),
        );
        Ok(inner.call(req).await)
    }
}

#[cfg(test)]
mod test {
    use http::{header::AUTHORIZATION, Request};
    use spacegate_shell::{
        ext_redis::global_repo,
        kernel::{
            backend_service::get_echo_service,
            extension::{GatewayName, MatchedSgRouter},
            service::http_route::match_request::{HttpPathMatchRewrite, HttpRouteMatch},
        },
        plugin::{spacegate_model, Inner, Plugin},
        SgBody,
    };
    use tardis::{cache::AsyncCommands, serde_json::json, test::test_container::TardisTestContainer, tokio};
    use testcontainers_modules::redis::REDIS_PORT;
    use tracing_subscriber::EnvFilter;

    use super::OpRedisHeaderExpandPlugin;

    #[tokio::test]
    async fn test_status() {
        const GW_NAME: &str = "REDIS-STATUS-TEST";
        const AK: &str = "ak-status";
        std::env::set_var("RUST_LOG", "trace");
        tracing_subscriber::fmt().with_env_filter(EnvFilter::from_default_env()).init();

        let redis_container = TardisTestContainer::redis_custom().await.expect("failed to create redis container");
        let host_port = redis_container.get_host_port_ipv4(REDIS_PORT).await.expect("failed to get redis port");
        let url = format!("redis://127.0.0.1:{host_port}");
        let plugin = OpRedisHeaderExpandPlugin::create_by_spec(
            json! {
                {
                    "cache_prefix_key": "sg:plugin:redis-status:test",
                    "header": AUTHORIZATION.as_str(),
                }
            },
            spacegate_model::PluginInstanceName::named("test"),
        )
        .expect("invalid config");
        global_repo().add(GW_NAME, url.as_str());
        let client = global_repo().get(GW_NAME).expect("missing client");
        let mut conn = client.get_conn().await;
        let _: () = conn.set(format!("sg:plugin:redis-header-expand:test:*:op-res:{AK}"), json!({ "foo": "bar" }).to_string()).await.expect("fail to set");
        let inner = Inner::new(get_echo_service());
        {
            let req = Request::builder()
                .uri("http://127.0.0.1/op-res/example")
                .method("GET")
                .extension(GatewayName::new(GW_NAME))
                .extension(MatchedSgRouter(
                    HttpRouteMatch {
                        path: Some(HttpPathMatchRewrite::prefix("op-res")),
                        ..Default::default()
                    }
                    .into(),
                ))
                .header(AUTHORIZATION, AK)
                .body(SgBody::empty())
                .expect("fail to build");
            let resp = plugin.call(req, inner.clone()).await.expect("infallible");
            let (parts, body) = resp.into_parts();
            let body = body.dump().await.expect("fail to dump");
            println!("body: {body:?}, parts: {parts:?}");
            assert!(parts.status.is_success());
        }
        let _: () = conn.set(format!("sg:plugin:redis-header-expand:test:*:op-res:{AK}"), json!({ "foo": "bar2" }).to_string()).await.expect("fail to set");
        {
            let req = Request::builder()
                .uri("http://127.0.0.1/op-res/example")
                .method("GET")
                .extension(GatewayName::new(GW_NAME))
                .extension(MatchedSgRouter(
                    HttpRouteMatch {
                        path: Some(HttpPathMatchRewrite::prefix("op-res")),
                        ..Default::default()
                    }
                    .into(),
                ))
                .header(AUTHORIZATION, AK)
                .body(SgBody::empty())
                .expect("fail to build");
            let resp = plugin.call(req, inner.clone()).await.expect("infallible");
            let (parts, body) = resp.into_parts();
            println!("body: {body:?}, parts: {parts:?}");
            assert!(parts.status.is_success());
        }
    }
}
