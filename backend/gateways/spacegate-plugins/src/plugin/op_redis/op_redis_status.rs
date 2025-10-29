use crate::plugin::op_redis::redis_format_key;
use serde::{Deserialize, Serialize};
use spacegate_shell::{
    ext_redis::{global_repo, Connection},
    hyper::{Request, Response},
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
spacegate_shell::plugin::schema!(OpRedisStatusPlugin, OpRedisStatusPlugin);
#[derive(Serialize, Deserialize, Clone, schemars::JsonSchema)]
#[serde(default)]
pub struct OpRedisStatusPlugin {
    pub cache_prefix_key: String,
    pub header: String,
}

impl Default for OpRedisStatusPlugin {
    fn default() -> Self {
        Self {
            cache_prefix_key: "sg:plugin:redis-status".to_string(),
            header: "X-Request-ID".to_string(),
        }
    }
}

async fn redis_call(mut conn: Connection, redis_key: String) -> Result<bool, RedisError> {
    let Result::<String, _>::Ok(status) = conn.get(&redis_key).await else {
        debug!("fail to get status with key {redis_key}");
        return Ok(true);
    };
    match status.as_str() {
        "Disabled" => Ok(false),
        "Enabled" => Ok(true),
        _ => Ok(true),
    }
}

impl Plugin for OpRedisStatusPlugin {
    const CODE: &'static str = "op-redis-status";

    fn meta() -> spacegate_shell::model::PluginMetaData {
        spacegate_shell::model::plugin_meta!(
            description: "Build for open platform, Control request access based on Redis-stored status."
        )
    }

    fn create(config: PluginConfig) -> Result<Self, BoxError> {
        let config: OpRedisStatusPlugin = serde_json::from_value(config.spec)?;
        Ok(config)
    }

    async fn call(&self, req: Request<SgBody>, inner: Inner) -> Result<Response<SgBody>, BoxError> {
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
        let pass: bool = redis_call(client.get_conn().await, format!("{}:{}", self.cache_prefix_key, key)).await?;
        if !pass {
            return Ok(PluginError::status::<OpRedisStatusPlugin, { code::FORBIDDEN }>("request denied").into());
        }
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

    use super::OpRedisStatusPlugin;

    #[tokio::test]
    async fn test_status() {
        const GW_NAME: &str = "REDIS-STATUS-TEST";
        const AK: &str = "ak-status";
        let _ = tracing_subscriber::fmt().with_env_filter(EnvFilter::from_default_env()).try_init();

        let redis_container = TardisTestContainer::redis_custom().await.expect("failed to create redis container");
        let host_port = redis_container.get_host_port_ipv4(REDIS_PORT).await.expect("failed to get redis port");
        let url = format!("redis://127.0.0.1:{host_port}");
        let plugin = OpRedisStatusPlugin::create_by_spec(
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
        let inner = Inner::new(get_echo_service());
        
        // Test case 1: enabled status - should allow request
        let _: () = conn.set(format!("sg:plugin:redis-status:test:*:op-res:{AK}"), "enabled").await.expect("fail to set");
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
            let (parts, _body) = resp.into_parts();
            println!("[Test 1] Enabled status - Status: {}", parts.status);
            assert!(parts.status.is_success(), "Expected success when status is 'enabled'");
        }
        
        // Test case 2: disabled status - should deny request
        let _: () = conn.set(format!("sg:plugin:redis-status:test:*:op-res:{AK}"), "disabled").await.expect("fail to set");
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
            let (parts, _body) = resp.into_parts();
            println!("[Test 2] Disabled status - Status: {}", parts.status);
            assert!(parts.status.is_client_error(), "Expected 403 when status is 'disabled'");
        }
        
        // Test case 3: missing key in Redis - should allow request (default behavior)
        let _: () = conn.del(format!("sg:plugin:redis-status:test:*:op-res:{AK}")).await.expect("fail to delete");
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
            let (parts, _body) = resp.into_parts();
            println!("[Test 3] Missing key - Status: {}", parts.status);
            assert!(parts.status.is_success(), "Expected success when key is missing (default allow)");
        }
        
        // Test case 4: unknown status value - should allow request (default behavior)
        let _: () = conn.set(format!("sg:plugin:redis-status:test:*:op-res:{AK}"), "unknown").await.expect("fail to set");
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
            let (parts, _body) = resp.into_parts();
            println!("[Test 4] Unknown status - Status: {}", parts.status);
            assert!(parts.status.is_success(), "Expected success when status is unknown (default allow)");
        }
        
        // Test case 5: missing authorization header - should return 401
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
                // No AUTHORIZATION header
                .body(SgBody::empty())
                .expect("fail to build");
            let resp = plugin.call(req, inner.clone()).await.expect("infallible");
            let (parts, _body) = resp.into_parts();
            println!("[Test 5] Missing header - Status: {}", parts.status);
            assert_eq!(parts.status.as_u16(), 401, "Expected 401 when authorization header is missing");
        }
    }
}
