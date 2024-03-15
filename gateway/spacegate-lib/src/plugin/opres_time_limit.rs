use std::sync::Arc;

use http::StatusCode;
use serde::{Deserialize, Serialize};
use spacegate_shell::{
    hyper::body::Bytes,
    kernel::{
        extension::GatewayName,
        helper_layers::check::{
            redis::{RedisCheck, RedisCheckScript},
            CheckLayer,
        },
        BoxResult,
    },
    plugin::{def_plugin, MakeSgLayer},
    spacegate_ext_redis::{global_repo, redis::AsyncCommands, Connection, RedisClientRepoError},
    SgBoxLayer,
};
use tardis::{
    chrono,
    futures::{future::BoxFuture, FutureExt},
    log::debug,
};

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct OpresTimeLimitConfig {
    prefix: String,
}

impl Default for OpresTimeLimitConfig {
    fn default() -> Self {
        Self {
            prefix: crate::consts::OP_RES_HEADER_DEFAULT.into(),
        }
    }
}

impl OpresTimeLimitConfig {
    pub fn create_check(&self, gateway_name: &str) -> BoxResult<RedisCheck> {
        let check_script = |mut conn: Connection, key: String| -> BoxFuture<bool> {
            let task = async move {
                let Result::<String, _>::Ok(time_range) = conn.get(&key).await else {
                    debug!("[Sg.Plugin.opres-time-limit] fail to get time range with key {key}");
                    return false;
                };
                let Some((from, to)) = time_range.split_once(',') else {
                    debug!("[Sg.Plugin.opres-time-limit] fail to parse time range with key {key}, expect a ',' spliter");
                    return false;
                };
                let (Ok(from), Ok(to)) = (chrono::DateTime::parse_from_rfc3339(from), chrono::DateTime::parse_from_rfc3339(to)) else {
                    debug!("[Sg.Plugin.opres-time-limit] fail to parse time range ({from},{to}) with key {key}");
                    return false;
                };
                let utc_now = chrono::Utc::now();
                from.to_utc() < utc_now && to.to_utc() >= utc_now
            };
            task.boxed()
        };
        let check = RedisCheck {
            check_script: Some(RedisCheckScript::Rust(Arc::new(check_script))),
            response_script: None,
            key_prefix: <Arc<str>>::from(format!("{}:date-time-range", self.prefix)),
            client: global_repo().get(gateway_name).ok_or(RedisClientRepoError::new(gateway_name, "missing redis client"))?,
            on_fail: Some((StatusCode::FORBIDDEN, Bytes::from_static(b"service expired"))),
        };
        Ok(check)
    }
    pub fn make_layer_with_gateway_name(&self, gateway_name: &str) -> BoxResult<spacegate_shell::SgBoxLayer> {
        let layer = CheckLayer::<_, crate::marker::OpresKey>::new(self.create_check(gateway_name.as_ref())?);
        Ok(SgBoxLayer::new(layer))
    }
}

impl MakeSgLayer for OpresTimeLimitConfig {
    fn make_layer(&self) -> BoxResult<spacegate_shell::SgBoxLayer> {
        self.make_layer_with_gateway_name("")
    }
    fn install_on_backend(&self, backend: &mut spacegate_shell::kernel::layers::http_route::builder::SgHttpBackendLayerBuilder) -> Result<(), spacegate_shell::BoxError> {
        let Some(gateway_name) = backend.extensions.get::<GatewayName>() else { return Ok(()) };
        backend.plugins.push(self.make_layer_with_gateway_name(gateway_name.as_ref())?);
        Ok(())
    }
    fn install_on_gateway(&self, gateway: &mut spacegate_shell::kernel::layers::gateway::builder::SgGatewayLayerBuilder) -> Result<(), spacegate_shell::BoxError> {
        let Some(gateway_name) = gateway.extension.get::<GatewayName>() else { return Ok(()) };
        gateway.http_plugins.push(self.make_layer_with_gateway_name(gateway_name.as_ref())?);
        Ok(())
    }
    fn install_on_route(&self, route: &mut spacegate_shell::kernel::layers::http_route::builder::SgHttpRouteLayerBuilder) -> Result<(), spacegate_shell::BoxError> {
        let Some(gateway_name) = route.extensions.get::<GatewayName>() else { return Ok(()) };
        route.plugins.push(self.make_layer_with_gateway_name(gateway_name.as_ref())?);
        Ok(())
    }
    fn install_on_rule(&self, rule: &mut spacegate_shell::kernel::layers::http_route::builder::SgHttpRouteRuleLayerBuilder) -> Result<(), spacegate_shell::BoxError> {
        let Some(gateway_name) = rule.extensions.get::<GatewayName>() else { return Ok(()) };
        rule.plugins.push(self.make_layer_with_gateway_name(gateway_name.as_ref())?);
        Ok(())
    }
}

def_plugin!("opres-time-limit", OpresTimeLimitPlugin, OpresTimeLimitConfig);

#[cfg(test)]
mod test {
    use http::Request;
    use spacegate_shell::{
        hyper::service::HttpService,
        kernel::{
            extension::MatchedSgRouter,
            layers::http_route::match_request::{SgHttpMethodMatch, SgHttpPathMatch, SgHttpRouteMatch},
            service::get_echo_service,
            Layer,
        },
        plugin::Plugin,
        SgBody,
    };
    use tardis::{basic::tracing::TardisTracing, serde_json::json, testcontainers, tokio};
    use testcontainers_modules::redis::REDIS_PORT;

    use super::*;
    #[tokio::test]
    async fn test_op_res_time_limit() {
        const GW_NAME: &str = "DEFAULT";
        std::env::set_var("RUST_LOG", "trace");
        let _ = TardisTracing::initializer().with_fmt_layer().with_env_layer().init_standalone();

        let docker = testcontainers::clients::Cli::default();
        let redis_container = docker.run(testcontainers_modules::redis::Redis);
        let host_port = redis_container.get_host_port_ipv4(REDIS_PORT);

        let url = format!("redis://127.0.0.1:{host_port}");
        let config = OpresTimeLimitPlugin::create(json! {
            {
                "prefix": "bios:limit"
            }
        })
        .expect("invalid config");
        global_repo().add(GW_NAME, url.as_str());
        let client = global_repo().get(GW_NAME).expect("missing client");
        let mut conn = client.get_conn().await;
        let _: () = conn.set("bios:limit:date-time-range:*:op-res:ak-not-pass", "2025-01-01T00:00:00-08:00,2026-01-01T00:00:00-08:00").await.expect("fail to set");
        let _: () = conn.set("bios:limit:date-time-range:*:op-res:ak-pass", "2024-01-01T00:00:00-08:00,2025-01-01T00:00:00-08:00").await.expect("fail to set");
        let layer = config.make_layer_with_gateway_name(GW_NAME).expect("fail to make layer");
        let backend_service = get_echo_service();
        let mut service = layer.layer(backend_service);
        {
            let req = Request::builder()
                .uri("http://localhost/op-res/example")
                .method("GET")
                .extension(GatewayName::new(GW_NAME))
                .extension(MatchedSgRouter(
                    SgHttpRouteMatch {
                        path: Some(SgHttpPathMatch::Prefix("op-res".to_string())),
                        ..Default::default()
                    }
                    .into(),
                ))
                .header("Bios-Authorization", "ak-pass:sign")
                .body(SgBody::empty())
                .expect("fail to build");
            let resp = service.call(req).await.expect("infallible");
            let (parts, body) = resp.into_parts();
            let body = body.dump().await.expect("fail to dump");
            println!("body: {body:?}, parts: {parts:?}");
            assert!(parts.status.is_success());
        }
        {
            let req = Request::builder()
                .uri("http://localhost/op-res/example")
                .method("GET")
                .extension(GatewayName::new(GW_NAME))
                .extension(MatchedSgRouter(
                    SgHttpRouteMatch {
                        path: Some(SgHttpPathMatch::Prefix("op-res".to_string())),
                        ..Default::default()
                    }
                    .into(),
                ))
                .header("Bios-Authorization", "ak-not-pass:sign")
                .body(SgBody::empty())
                .expect("fail to build");
            let resp = service.call(req).await.expect("infallible");
            let (parts, body) = resp.into_parts();
            println!("body: {body:?}, parts: {parts:?}");
            assert!(parts.status.is_client_error());
        }
        {
            let req = Request::builder()
                .uri("http://localhost/op-res/example")
                .method("POST")
                .extension(GatewayName::new(GW_NAME))
                .extension(MatchedSgRouter(
                    SgHttpRouteMatch {
                        path: Some(SgHttpPathMatch::Prefix("op-res".to_string())),
                        ..Default::default()
                    }
                    .into(),
                ))
                .body(SgBody::empty())
                .expect("fail to build");
            let resp = service.call(req).await.expect("infallible");
            let (parts, body) = resp.into_parts();
            println!("body: {body:?}, parts: {parts:?}");
            assert!(parts.status.is_client_error());
        }
        {
            let req = Request::builder()
                .uri("http://localhost/op-res/example")
                .method("DELETE")
                .extension(GatewayName::new(GW_NAME))
                .extension(MatchedSgRouter(
                    SgHttpRouteMatch {
                        path: Some(SgHttpPathMatch::Prefix("op-res".to_string())),
                        method: Some(vec![SgHttpMethodMatch("DELETE".into())]),
                        ..Default::default()
                    }
                    .into(),
                ))
                .header("Bios-Authorization", "ak-pass:sign")
                .body(SgBody::empty())
                .expect("fail to build");
            let resp = service.call(req).await.expect("infallible");
            let (parts, body) = resp.into_parts();
            println!("body: {body:?}, parts: {parts:?}");
            assert!(parts.status.is_client_error());
        }
    }
}
