use std::{collections::HashMap, mem, str::FromStr, sync::Arc};

use async_trait::async_trait;
use bios_auth::{
    auth_config::AuthConfig,
    auth_initializer,
    dto::{
        auth_crypto_dto::AuthEncryptReq,
        auth_kernel_dto::{AuthReq, AuthResp},
    },
    serv::{auth_crypto_serv, auth_kernel_serv},
};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use spacegate_kernel::{
    config::http_route_dto::SgHttpRouteRule,
    functions::http_route::SgHttpRouteMatchInst,
    http::{self, HeaderMap, HeaderName, HeaderValue},
    plugins::{
        context::{SgRouteFilterRequestAction, SgRoutePluginContext},
        filters::{BoxSgPluginFilter, SgPluginFilter, SgPluginFilterDef, SgPluginFilterKind},
    },
};
use tardis::{
    async_trait,
    basic::{error::TardisError, result::TardisResult},
    config::config_dto::{AppConfig, CacheConfig, FrameworkConfig, TardisConfig, WebServerConfig, WebServerModuleConfig},
    log,
    serde_json::{self, Value},
    tokio::{self, sync::Mutex, task::JoinHandle},
    TardisFuns,
};

lazy_static! {
    static ref SHUTDOWN: Arc<Mutex<Option<JoinHandle<()>>>> = <_>::default();
}

pub const CODE: &str = "auth";
pub struct SgFilterAuthDef;

impl SgPluginFilterDef for SgFilterAuthDef {
    fn inst(&self, spec: serde_json::Value) -> TardisResult<BoxSgPluginFilter> {
        let filter = TardisFuns::json.json_to_obj::<SgFilterAuth>(spec)?;
        Ok(filter.boxed())
    }
}

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct SgFilterAuth {
    auth_config: AuthConfig,
    port: u16,
    cache_url: String,
}

impl Default for SgFilterAuth {
    fn default() -> Self {
        Self {
            auth_config: Default::default(),
            port: 8080,
            cache_url: "redis://127.0.0.1:6379".to_string(),
        }
    }
}

#[async_trait]
impl SgPluginFilter for SgFilterAuth {
    fn kind(&self) -> SgPluginFilterKind {
        SgPluginFilterKind::Http
    }

    async fn init(&self, _: &[SgHttpRouteRule]) -> TardisResult<()> {
        let mut cs = HashMap::<String, Value>::new();
        cs.insert(
            bios_auth::auth_constants::DOMAIN_CODE.to_string(),
            serde_json::to_value(self.auth_config.clone()).map_err(|e| TardisError::internal_error(&format!("[Plugin.Auth]init auth config error: {e:?}"), ""))?,
        );
        TardisFuns::init_conf(TardisConfig {
            cs,
            fw: FrameworkConfig {
                app: AppConfig {
                    name: "spacegate.plugin.auth".to_string(),
                    desc: "This is a spacegate plugin-auth".to_string(),
                    ..Default::default()
                },
                web_server: WebServerConfig {
                    enabled: true,
                    port: self.port,
                    modules: HashMap::from([("auth".to_string(), WebServerModuleConfig { ..Default::default() })]),
                    ..Default::default()
                },
                cache: CacheConfig {
                    enabled: true,
                    url: self.cache_url.clone(),
                    ..Default::default()
                },
                ..Default::default()
            },
        })
        .await?;
        let web_server = TardisFuns::web_server();
        auth_initializer::init(web_server).await?;
        let mut shut_down = SHUTDOWN.lock().await;
        let join_handle = tokio::spawn(async move {
            let _ = web_server.start().await;
        });
        *shut_down = Some(join_handle);
        Ok(())
    }

    async fn destroy(&self) -> TardisResult<()> {
        let mut shut_down = SHUTDOWN.lock().await;
        let mut swap_shutdown: Option<JoinHandle<()>> = None;
        mem::swap(&mut *shut_down, &mut swap_shutdown);
        if let Some(shutdown) = swap_shutdown {
            if !shutdown.is_finished() {
                shutdown.abort();
            };
            log::info!("[SG.Filter.Status] Server stopped");
        };
        Ok(())
    }

    async fn req_filter(&self, _: &str, mut ctx: SgRoutePluginContext, _matched_match_inst: Option<&SgHttpRouteMatchInst>) -> TardisResult<(bool, SgRoutePluginContext)> {
        if ctx.get_req_method() == &http::Method::OPTIONS {
            return Ok((true, ctx));
        }
        match auth_kernel_serv::auth(&mut ctx_to_auth_req(&mut ctx).await?, false).await {
            Ok(auth_resp) => {
                if auth_resp.allow {
                    ctx = success_auth_resp_to_ctx(auth_resp, ctx)?;
                } else {
                    ctx.set_action(SgRouteFilterRequestAction::Response);
                    ctx.set_resp_body(auth_resp.reason.map(|s| s.into_bytes()).unwrap_or_default())?;
                    return Ok((false, ctx));
                };
                Ok((true, ctx))
            }
            Err(e) => {
                ctx.set_action(SgRouteFilterRequestAction::Response);
                ctx.set_resp_body(format!("[Plugin.Auth] auth return error:{e}").into_bytes())?;
                Ok((false, ctx))
            }
        }
    }

    async fn resp_filter(&self, _: &str, mut ctx: SgRoutePluginContext, _: Option<&SgHttpRouteMatchInst>) -> TardisResult<(bool, SgRoutePluginContext)> {
        if ctx.get_resp_headers().get(&self.auth_config.head_key_crypto).is_none() {
            return Ok((true, ctx));
        }
        let encrypt_resp = auth_crypto_serv::encrypt_body(&ctx_to_auth_encrypt_req(&mut ctx).await?).await?;
        ctx.set_resp_headers(hashmap_header_to_headermap(encrypt_resp.headers)?);
        ctx.set_resp_body(encrypt_resp.body.into_bytes())?;

        Ok((true, ctx))
    }
}
async fn ctx_to_auth_req(ctx: &mut SgRoutePluginContext) -> TardisResult<AuthReq> {
    let url = ctx.get_req_uri().clone();
    let scheme = url.scheme().map(|s| s.to_string()).unwrap_or("http".to_string());
    let headers = headermap_header_to_hashmap(ctx.get_req_headers().clone())?;

    Ok(AuthReq {
        scheme: scheme.clone(),
        path: url.path().to_string(),
        query: url
            .query()
            .map(|q| {
                q.split('&')
                    .map(|s| {
                        let a: Vec<_> = s.split('=').collect();
                        (a[0].to_string(), a[1].to_string())
                    })
                    .collect()
            })
            .unwrap_or_default(),
        method: ctx.get_req_method().to_string(),
        host: url.host().unwrap_or("127.0.0.1").to_string(),
        port: url.port().map(|p| p.as_u16()).unwrap_or_else(|| if scheme == "https" { 443 } else { 80 }),
        headers,
        body: ctx.pop_req_body().await?.map(|s| String::from_utf8_lossy(&s).to_string()),
    })
}

fn success_auth_resp_to_ctx(auth_resp: AuthResp, mut ctx: SgRoutePluginContext) -> TardisResult<SgRoutePluginContext> {
    let new_headers = hashmap_header_to_headermap(auth_resp.headers.clone())?;

    ctx.set_resp_headers(new_headers);
    ctx.set_req_body(auth_resp.body.map(|s| s.into_bytes()).unwrap_or_default())?;
    Ok(ctx)
}

async fn ctx_to_auth_encrypt_req(ctx: &mut SgRoutePluginContext) -> TardisResult<AuthEncryptReq> {
    let headers = headermap_header_to_hashmap(ctx.get_resp_headers().clone())?;

    Ok(AuthEncryptReq {
        headers,
        body: ctx.pop_resp_body().await?.map(|s| String::from_utf8_lossy(&s).to_string()).unwrap_or_default(),
    })
}

fn hashmap_header_to_headermap(old_headers: HashMap<String, String>) -> TardisResult<HeaderMap> {
    let mut new_headers = HeaderMap::new();
    for header in old_headers {
        new_headers.insert(
            HeaderName::from_str(&header.0).map_err(|e| TardisError::format_error(&format!("[Plugin.Auth] request header error :{e}"), ""))?,
            HeaderValue::from_str(&header.1).map_err(|e| TardisError::format_error(&format!("[Plugin.Auth] request header error :{e}"), ""))?,
        );
    }
    Ok(new_headers)
}

fn headermap_header_to_hashmap(old_headers: HeaderMap) -> TardisResult<HashMap<String, String>> {
    let mut new_headers = HashMap::new();
    for header_name in old_headers.keys() {
        new_headers.insert(
            header_name.to_string(),
            old_headers.get(header_name).expect("").to_str().map_err(|e| TardisError::format_error(&format!("[Plugin.Auth] response header error :{e}"), ""))?.to_string(),
        );
    }
    Ok(new_headers)
}

#[cfg(test)]
mod tests {
    use std::env;

    use tardis::{
        test::test_container::TardisTestContainer,
        testcontainers::{self, clients::Cli, images::redis::Redis, Container},
        tokio,
        web::web_resp::TardisResp,
    };

    use super::*;
    #[tokio::test]
    async fn test_auth_plugin() {
        env::set_var("RUST_LOG", "debug,bios_auth=trace,tardis=trace");
        tracing_subscriber::fmt::init();

        let docker = testcontainers::clients::Cli::default();
        let _x = docker_init(&docker).await.unwrap();

        let filter_auth = SgFilterAuth {
            cache_url: env::var("TARDIS_FW.CACHE.URL").unwrap(),
            ..Default::default()
        };

        filter_auth.init(&[]).await.unwrap();

        let apis = TardisFuns::web_client().get::<TardisResp<Value>>(&format!("http://127.0.0.1:{}/auth/auth/apis", filter_auth.port), None).await.unwrap().body;
        assert!(apis.is_some());
        let apis = apis.unwrap();
        assert!(apis.code == 200.to_string());
        assert!(apis.data.is_some());

        //dont need to decrypt

        filter_auth.destroy().await.unwrap();

        let test_result = TardisFuns::web_client().get_to_str(&format!("http://127.0.0.1:{}/auth/auth/apis", filter_auth.port), None).await;
        assert!(test_result.is_err() || test_result.unwrap().code == 502);
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
