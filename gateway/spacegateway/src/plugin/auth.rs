use std::{collections::HashMap, str::FromStr};

use async_trait::async_trait;
use bios_auth::{
    dto::auth_kernel_dto::{AuthReq, AuthResp},
    serv::auth_kernel_serv,
};
use serde::{Deserialize, Serialize};
use spacegate_kernel::{
    config::http_route_dto::SgHttpRouteRule,
    functions::http_route::SgHttpRouteMatchInst,
    http::{self, uri::Port, HeaderMap, HeaderName, HeaderValue},
    plugins::{
        context::SgRoutePluginContext,
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
        let result = auth_kernel_serv::auth(&mut ctx_to_authReq(&mut ctx).await?, false).await?;
        Ok((true, ctx))
    }

    async fn resp_filter(&self, _: &str, ctx: SgRoutePluginContext, _: Option<&SgHttpRouteMatchInst>) -> TardisResult<(bool, SgRoutePluginContext)> {
        Ok((true, ctx))
    }
}
async fn ctx_to_authReq(ctx: &mut SgRoutePluginContext) -> TardisResult<AuthReq> {
    let url = ctx.get_req_uri().clone();
    let scheme = url.scheme().map(|s| s.to_string()).unwrap_or("http".to_string());
    let mut headers = HashMap::new();
    let req_headers = ctx.get_req_headers();
    for header_name in req_headers.keys().cloned() {
        headers.insert(
            header_name.to_string(),
            req_headers.get(header_name).clone().expect("").to_str().map_err(|e| TardisError::format_error("[auth] request header error", ""))?.to_string(),
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
    let a = HeaderMap::new();
    for header in auth_resp.headers {
        a.insert(
            HeaderName::from_str(&header.0).map_err(|e| TardisError::format_error("[auth] request header error", ""))?,
            HeaderValue::from_str(&header.1).map_err(|e| TardisError::format_error("[auth] request header error", ""))?,
        );
    }

    ctx.set_resp_headers(a);
    ctx.set_req_body(auth_resp.body);
    ctx
}
