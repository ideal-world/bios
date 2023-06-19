use std::{collections::HashMap, str::FromStr};

use async_trait::async_trait;
use bios_auth::{
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
    http::{self, uri::Port, HeaderMap, HeaderName, HeaderValue},
    plugins::{
        context::{SgRouteFilterRequestAction, SgRoutePluginContext},
        filters::{BoxSgPluginFilter, SgPluginFilter, SgPluginFilterDef, SgPluginFilterKind},
    },
};
use tardis::{
    async_trait,
    basic::{error::TardisError, result::TardisResult},
    serde_json, TardisFuns,
};

pub const CODE: &str = "auth";
pub struct SgFilterAuthDef;

impl SgPluginFilterDef for SgFilterAuthDef {
    fn inst(&self, spec: serde_json::Value) -> TardisResult<BoxSgPluginFilter> {
        let filter = TardisFuns::json.json_to_obj::<SgFilterAuth>(spec)?;
        Ok(filter.boxed())
    }
}

#[derive(Serialize, Deserialize)]
pub struct SgFilterAuth {
    is_enabled: bool,
    title: Option<String>,
    msg: Option<String>,
}

#[async_trait]
impl SgPluginFilter for SgFilterAuth {
    fn kind(&self) -> SgPluginFilterKind {
        SgPluginFilterKind::Http
    }

    async fn init(&self, _: &[SgHttpRouteRule]) -> TardisResult<()> {
        Ok(())
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
        match auth_crypto_serv::encrypt_body(&ctx_to_auth_encrypt_req(&mut ctx).await?).await {
            Ok(encrypt_resp) => {
                ctx.set_resp_headers();
                ctx.set_resp_body(encrypt_resp.body.into_bytes())?;
            }
            Err(e) => {}
        };
        Ok((true, ctx))
    }
}
async fn ctx_to_auth_req(ctx: &mut SgRoutePluginContext) -> TardisResult<AuthReq> {
    let url = ctx.get_req_uri().clone();
    let scheme = url.scheme().map(|s| s.to_string()).unwrap_or("http".to_string());
    let mut headers = HashMap::new();
    let req_headers = ctx.get_req_headers();
    for header_name in req_headers.keys().cloned() {
        headers.insert(
            header_name.to_string(),
            req_headers.get(header_name).clone().expect("").to_str().map_err(|e| TardisError::format_error(&format!("[Plugin.Auth] request header error :{e}"), ""))?.to_string(),
        );
    }
    Ok(AuthReq {
        scheme: scheme.clone(),
        path: url.path().to_string(),
        query: url
            .query()
            .map(|q| {
                q.split("&")
                    .into_iter()
                    .map(|s| {
                        let a: Vec<_> = s.split("=").collect();
                        (a[0].to_string(), a[1].to_string())
                    })
                    .collect()
            })
            .unwrap_or_default(),
        method: ctx.get_req_method().to_string(),
        host: url.host().unwrap_or_else(|| "127.0.0.1").to_string(),
        port: url.port().map(|p| p.as_u16()).unwrap_or_else(|| if scheme == "https" { 443 } else { 80 }),
        headers,
        body: ctx.pop_req_body().await?.map(|s| String::from_utf8_lossy(&s).to_string()),
    })
}

fn success_auth_resp_to_ctx(auth_resp: AuthResp, mut ctx: SgRoutePluginContext) -> TardisResult<SgRoutePluginContext> {
    let mut new_headers = HeaderMap::new();
    for header in auth_resp.headers {
        new_headers.insert(
            HeaderName::from_str(&header.0).map_err(|e| TardisError::format_error(&format!("[Plugin.Auth] request header error :{e}"), ""))?,
            HeaderValue::from_str(&header.1).map_err(|e| TardisError::format_error(&format!("[Plugin.Auth] request header error :{e}"), ""))?,
        );
    }

    ctx.set_resp_headers(new_headers);
    ctx.set_req_body(auth_resp.body.map(|s| s.into_bytes()).unwrap_or_default())?;
    Ok(ctx)
}

async fn ctx_to_auth_encrypt_req(ctx: &mut SgRoutePluginContext) -> TardisResult<AuthEncryptReq> {
    let mut headers = HashMap::new();
    let resp_headers = ctx.get_resp_headers();
    for header_name in resp_headers.keys().cloned() {
        headers.insert(
            header_name.to_string(),
            resp_headers.get(header_name).clone().expect("").to_str().map_err(|e| TardisError::format_error(&format!("[Plugin.Auth] response header error :{e}"), ""))?.to_string(),
        );
    }

    Ok(AuthEncryptReq {
        headers,
        body: ctx.pop_resp_body().await?.map(|s| String::from_utf8_lossy(&s).to_string()).unwrap_or_default(),
    })
}
