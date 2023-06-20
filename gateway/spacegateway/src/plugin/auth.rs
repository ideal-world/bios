use std::{collections::HashMap, str::FromStr};

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
    config::config_dto::{CacheConfig, FrameworkConfig, TardisConfig, WebServerConfig, WebServerModuleConfig},
    serde_json::{self, Value},
    TardisFuns,
};

pub const CODE: &str = "auth";
pub struct SgFilterAuthDef;

impl SgPluginFilterDef for SgFilterAuthDef {
    fn inst(&self, spec: serde_json::Value) -> TardisResult<BoxSgPluginFilter> {
        let filter = TardisFuns::json.json_to_obj::<SgFilterAuth>(spec)?;
        Ok(filter.boxed())
    }
}

#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct SgFilterAuth {
    auth_config: AuthConfig,
    port: u16,
    cache_url: String,
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
        auth_initializer::init(web_server).await
    }

    async fn destroy(&self) -> TardisResult<()> {
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
