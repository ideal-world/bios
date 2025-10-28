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
spacegate_shell::plugin::schema!(OpRedisAllowApiPlugin, OpRedisAllowApiPlugin);
#[derive(Serialize, Deserialize, Clone, schemars::JsonSchema)]
#[serde(default)]
pub struct OpRedisAllowApiPlugin {
    pub cache_prefix_key: String,
    pub header: String,
}

impl Default for OpRedisAllowApiPlugin {
    fn default() -> Self {
        Self {
            cache_prefix_key: "sg:plugin:redis-allow-api".to_string(),
            header: "X-Request-ID".to_string(),
        }
    }
}

async fn redis_call(mut conn: Connection, paths: String, redis_key: String) -> Result<bool, RedisError> {
    let Result::<String, _>::Ok(apis_json) = conn.get(&redis_key).await else {
        debug!("fail to get status with key {redis_key}");
        // 没有匹配到任何 API，默认放行
        return Ok(true);
    };
    let apis: Vec<String> = match serde_json::from_str(&apis_json) {
        Ok(apis) => apis,
        Err(e) => {
            // 解析失败，默认放行
            debug!("fail to parse apis json with key {redis_key}: {}", e);
            return Ok(true);
        }
    };
    // 支持模糊匹配 例如 /op-res/eq,/op-res/one/*,/op-res/multiple/** 等
    // * 匹配单个路径段，** 匹配多个路径段
    // 遍历所有 API，检查是否有匹配的
    for api in apis {
        let api_parts: Vec<&str> = api.split('/').collect();
        let path_parts: Vec<&str> = paths.split('/').collect();
        let mut i = 0;
        let mut j = 0;
        while i < api_parts.len() && j < path_parts.len() {
            if api_parts[i] == "**" {
                if i + 1 == api_parts.len() {
                    return Ok(true);
                }
                i += 1;
                while j < path_parts.len() && path_parts[j] != api_parts[i] {
                    j += 1;
                }
            } else if api_parts[i] == "*" || api_parts[i] == path_parts[j] {
                i += 1;
                j += 1;
            } else {
                break;
            }
        }
        if i == api_parts.len() && j == path_parts.len() {
            return Ok(true);
        }
    }
    // 没有匹配到任何 API，拒绝访问
    Ok(false)
}

impl Plugin for OpRedisAllowApiPlugin {
    const CODE: &'static str = "redis-status";

    fn meta() -> spacegate_shell::model::PluginMetaData {
        spacegate_shell::model::plugin_meta!(
            description: "Build for open platform, Control request access based on Redis-stored status."
        )
    }

    fn create(config: PluginConfig) -> Result<Self, BoxError> {
        let config: OpRedisAllowApiPlugin = serde_json::from_value(config.spec)?;
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
        let pass: bool = redis_call(client.get_conn().await, req.uri().path().to_string(), format!("{}:{}", self.cache_prefix_key, key)).await?;
        if !pass {
            return Ok(PluginError::status::<OpRedisAllowApiPlugin, { code::FORBIDDEN }>("request denied").into());
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

    use super::OpRedisAllowApiPlugin;

    #[tokio::test]
    async fn test_allow_api() {
        const GW_NAME: &str = "REDIS-ALLOW-API-TEST";
        const AK: &str = "ak-test-allow";
        let _ = tracing_subscriber::fmt().with_env_filter(EnvFilter::from_default_env()).try_init();

        let redis_container = TardisTestContainer::redis_custom().await.expect("failed to create redis container");
        let host_port = redis_container.get_host_port_ipv4(REDIS_PORT).await.expect("failed to get redis port");
        let url = format!("redis://127.0.0.1:{host_port}");
        let plugin = OpRedisAllowApiPlugin::create_by_spec(
            json! {
                {
                    "cache_prefix_key": "sg:plugin:redis-allow-api:test",
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
        
        // Setup: 配置允许的 API 列表
        let _: () = conn
            .set(
                format!("sg:plugin:redis-allow-api:test:*:op-res:{AK}"),
                json!(["/op-res/eq", "/op-res/one/*", "/op-res/multiple/**", "/op-res/users/*/profile"]).to_string(),
            )
            .await
            .expect("fail to set");
        
        // Test case 1: Exact match - 精确匹配
        {
            let req = Request::builder()
                .uri("http://127.0.0.1/op-res/eq")
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
            println!("[Test 1] Exact match '/op-res/eq' - Status: {}", parts.status);
            assert!(parts.status.is_success(), "Expected success for exact match");
        }
        
        // Test case 2: Exact match failure - 精确匹配失败
        {
            let req = Request::builder()
                .uri("http://127.0.0.1/op-res/eq/extra")
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
            println!("[Test 2] Exact match fail '/op-res/eq/extra' - Status: {}", parts.status);
            assert!(parts.status.is_client_error(), "Expected 403 for non-matching path");
        }
        
        // Test case 3: Single wildcard match - 单层通配符匹配
        {
            let req = Request::builder()
                .uri("http://127.0.0.1/op-res/one/abc")
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
            println!("[Test 3] Single wildcard match '/op-res/one/*' -> '/op-res/one/abc' - Status: {}", parts.status);
            assert!(parts.status.is_success(), "Expected success for single wildcard match");
        }
        
        // Test case 4: Single wildcard mismatch - 单层通配符不匹配多层
        {
            let req = Request::builder()
                .uri("http://127.0.0.1/op-res/one/abc/def")
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
            println!("[Test 4] Single wildcard mismatch '/op-res/one/*' !-> '/op-res/one/abc/def' - Status: {}", parts.status);
            assert!(parts.status.is_client_error(), "Expected 403 when single wildcard doesn't match multiple segments");
        }
        
        // Test case 5: Double wildcard match - 多层通配符匹配
        {
            let req = Request::builder()
                .uri("http://127.0.0.1/op-res/multiple/a/b/c")
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
            println!("[Test 5] Double wildcard match '/op-res/multiple/**' -> '/op-res/multiple/a/b/c' - Status: {}", parts.status);
            assert!(parts.status.is_success(), "Expected success for double wildcard match with multiple segments");
        }
        
        // Test case 6: Double wildcard with single segment - 多层通配符匹配单层
        {
            let req = Request::builder()
                .uri("http://127.0.0.1/op-res/multiple/only-one")
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
            println!("[Test 6] Double wildcard single segment '/op-res/multiple/**' -> '/op-res/multiple/only-one' - Status: {}", parts.status);
            assert!(parts.status.is_success(), "Expected success for double wildcard with single segment");
        }
        
        // Test case 7: Wildcard in middle - 中间通配符
        {
            let req = Request::builder()
                .uri("http://127.0.0.1/op-res/users/123/profile")
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
            println!("[Test 7] Wildcard in middle '/op-res/users/*/profile' -> '/op-res/users/123/profile' - Status: {}", parts.status);
            assert!(parts.status.is_success(), "Expected success for wildcard in middle of path");
        }
        
        // Test case 8: Wildcard in middle mismatch - 中间通配符不匹配
        {
            let req = Request::builder()
                .uri("http://127.0.0.1/op-res/users/123/settings")
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
            println!("[Test 8] Wildcard in middle mismatch '/op-res/users/*/profile' !-> '/op-res/users/123/settings' - Status: {}", parts.status);
            assert!(parts.status.is_client_error(), "Expected 403 when path after wildcard doesn't match");
        }
        
        // Test case 9: Missing key in Redis - 默认允许
        let _: () = conn.del(format!("sg:plugin:redis-allow-api:test:*:op-res:{AK}")).await.expect("fail to delete");
        {
            let req = Request::builder()
                .uri("http://127.0.0.1/op-res/any/path")
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
            println!("[Test 9] Missing key in Redis - Status: {}", parts.status);
            assert!(parts.status.is_success(), "Expected success when key is missing (default allow)");
        }
        
        // Test case 10: Invalid JSON in Redis - 默认允许
        let _: () = conn.set(format!("sg:plugin:redis-allow-api:test:*:op-res:{AK}"), "invalid-json-data").await.expect("fail to set");
        {
            let req = Request::builder()
                .uri("http://127.0.0.1/op-res/any/path")
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
            println!("[Test 10] Invalid JSON in Redis - Status: {}", parts.status);
            assert!(parts.status.is_success(), "Expected success when JSON is invalid (graceful degradation)");
        }
        
        // Test case 11: Empty API list - 拒绝所有
        let _: () = conn.set(format!("sg:plugin:redis-allow-api:test:*:op-res:{AK}"), "[]").await.expect("fail to set");
        {
            let req = Request::builder()
                .uri("http://127.0.0.1/op-res/any/path")
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
            println!("[Test 11] Empty API list - Status: {}", parts.status);
            assert!(parts.status.is_client_error(), "Expected 403 when API list is empty");
        }
        
        // Test case 12: Missing authorization header - 401 错误
        {
            let req = Request::builder()
                .uri("http://127.0.0.1/op-res/any")
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
            println!("[Test 12] Missing authorization header - Status: {}", parts.status);
            assert_eq!(parts.status.as_u16(), 401, "Expected 401 when authorization header is missing");
        }
    }
}
