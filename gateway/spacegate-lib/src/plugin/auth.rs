use async_trait::async_trait;
use bios_auth::{
    auth_config::AuthConfig,
    auth_initializer,
    dto::{
        auth_crypto_dto::AuthEncryptReq,
        auth_kernel_dto::{AuthReq, AuthResp, AuthResult, MixRequestBody},
    },
    serv::{auth_crypto_serv, auth_kernel_serv, auth_res_serv},
};

use serde::{Deserialize, Serialize};
use spacegate_kernel::{
    def_filter,
    http::{self, HeaderMap, HeaderName, HeaderValue},
    hyper,
    hyper::{body::Bytes, Body, Method},
    plugins::{
        context::{SgRouteFilterRequestAction, SgRoutePluginContext},
        filters::{SgPluginFilter, SgPluginFilterAccept},
    },
};
use spacegate_kernel::{
    hyper::StatusCode,
    plugins::{
        context::{SGIdentInfo, SGRoleInfo},
        filters::SgPluginFilterInitDto,
    },
};
use std::{
    collections::HashMap,
    str::FromStr,
    sync::{Arc, OnceLock},
};
use tardis::{
    async_trait,
    basic::{error::TardisError, result::TardisResult},
    config::config_dto::{AppConfig, CacheModuleConfig, FrameworkConfig, TardisConfig},
    log,
    serde_json::{self, json, Value},
    tokio::{sync::RwLock, task::JoinHandle},
    url::Url,
    web::web_resp::TardisResp,
    TardisFuns,
};
use tardis::{basic::tracing::Directive, tracing, web::poem_openapi::types::Type};

use super::plugin_constants;

#[allow(clippy::type_complexity)]
static INSTANCE: OnceLock<Arc<RwLock<Option<(String, JoinHandle<()>)>>>> = OnceLock::new();

def_filter!("auth", SgFilterAuthDef, SgFilterAuth);

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct SgFilterAuth {
    auth_config: AuthConfig,
    cache_url: String,
    header_is_mix_req: String,
    cors_allow_origin: String,
    cors_allow_methods: String,
    cors_allow_headers: String,
    fetch_server_config_path: String,
    /// Specify the part of the mix request url that needs to be replaced.
    /// Default is `apis`
    ///
    /// e.g
    ///
    /// |request mix url|replace_url|        result       |
    /// |---------------|-----------|---------------------|
    /// |   `/apis`     |  `apis`   |    `/{true_url}`    |
    /// |`/prefix/apis` |  `apis`   |`/prefix/{true_url}` |
    mix_replace_url: String,
    /// Remove prefix of AuthReq path.
    /// use for [ctx_to_auth_req]
    /// Used to remove a fixed prefix from the original URL.
    auth_path_ignore_prefix: String,
}

impl Default for SgFilterAuth {
    fn default() -> Self {
        Self {
            auth_config: Default::default(),
            cache_url: "".to_string(),
            cors_allow_origin: "*".to_string(),
            cors_allow_methods: "*".to_string(),
            cors_allow_headers: "*".to_string(),
            header_is_mix_req: "IS_MIX_REQ".to_string(),
            fetch_server_config_path: "/starsysApi/apis".to_string(),
            mix_replace_url: "apis".to_string(),
            auth_path_ignore_prefix: "/starsysApi".to_string(),
        }
    }
}
impl SgFilterAuth {
    fn cors(&self, ctx: &mut SgRoutePluginContext) -> TardisResult<()> {
        ctx.response.set_header(http::header::ACCESS_CONTROL_ALLOW_ORIGIN, &self.cors_allow_origin)?;
        ctx.response.set_header(http::header::ACCESS_CONTROL_ALLOW_METHODS, &self.cors_allow_methods)?;
        ctx.response.set_header(http::header::ACCESS_CONTROL_ALLOW_HEADERS, &self.cors_allow_headers)?;
        ctx.response.set_header(http::header::ACCESS_CONTROL_MAX_AGE, "3600000")?;
        ctx.response.set_header(http::header::ACCESS_CONTROL_ALLOW_CREDENTIALS, "true")?;
        ctx.response.set_header(http::header::CONTENT_TYPE, "application/json")?;
        Ok(())
    }
    fn get_is_true_mix_req_from_header(&self, header_map: &HeaderMap<HeaderValue>) -> bool {
        header_map
            .get(&self.header_is_mix_req)
            .map(|v: &HeaderValue| {
                bool::from_str(v.to_str().map_err(|e| TardisError::custom("502", &format!("[Plugin.Auth] parse header IS_MIX_REQ error:{e}"), ""))?)
                    .map_err(|e| TardisError::custom("502", &format!("[Plugin.Auth] parse header IS_MIX_REQ error:{e}"), ""))
            })
            .transpose()
            .ok()
            .flatten()
            .unwrap_or(false)
    }
}

#[async_trait]
impl SgPluginFilter for SgFilterAuth {
    fn accept(&self) -> SgPluginFilterAccept {
        SgPluginFilterAccept::default()
    }

    async fn init(&mut self, init_dto: &SgPluginFilterInitDto) -> TardisResult<()> {
        if let Some(log_level) = &init_dto.gateway_parameters.log_level {
            let mut log_config = TardisFuns::fw_config().log().clone();
            fn directive(path: &str, lvl: &str) -> Directive {
                let s = format!("{path}={lvl}");
                format!("{path}={lvl}").parse().unwrap_or_else(|e| {
                    tracing::error!("[SG.Filter.Auth] failed to parse directive {:?}: {}", s, e);
                    Default::default()
                })
            }
            log_config.directives.push(directive(crate::PACKAGE_NAME, log_level));
            log_config.directives.push(directive(bios_auth::auth_constants::PACKAGE_NAME, log_level));
            TardisFuns::tracing().update_config(&log_config)?;
        }

        let config_md5 = TardisFuns::crypto.digest.md5(TardisFuns::json.obj_to_string(self)?)?;

        let mut instance = INSTANCE.get_or_init(Default::default).write().await;
        if let Some((md5, handle)) = instance.as_ref() {
            if config_md5.eq(md5) {
                log::trace!("[SG.Filter.Auth] have not found config change");
                return Ok(());
            } else {
                handle.abort();
            }
        }

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
                cache: Some(
                    CacheModuleConfig::builder()
                        .url(
                            if self.cache_url.is_empty() {
                                init_dto.gateway_parameters.redis_url.as_deref().unwrap_or("redis://127.0.0.1:6379")
                            } else {
                                self.cache_url.as_str()
                            }
                            .parse()
                            .map_err(|e| TardisError::internal_error(&format!("[Plugin.Auth]invalid redis url: {e:?}"), "-1"))?,
                        )
                        .build()
                        .into(),
                ),
                ..Default::default()
            },
        })
        .await?;

        let handle = auth_initializer::init().await?;
        *instance = Some((config_md5, handle));
        log::info!("[SG.Filter.Auth] init done");

        Ok(())
    }

    async fn destroy(&self) -> TardisResult<()> {
        Ok(())
    }

    async fn req_filter(&self, _: &str, mut ctx: SgRoutePluginContext) -> TardisResult<(bool, SgRoutePluginContext)> {
        if ctx.request.get_method() == http::Method::OPTIONS {
            return Ok((true, ctx));
        }

        log::trace!("[Plugin.Auth] request filter info: request path is {}", ctx.request.get_uri().path());
        if ctx.request.get_method().eq(&Method::GET) && ctx.request.get_uri().path() == self.fetch_server_config_path.as_str() {
            log::debug!("[Plugin.Auth] request path hit fetch server config path: {}", self.fetch_server_config_path);
            ctx.set_action(SgRouteFilterRequestAction::Response);
            let mut headers = HeaderMap::new();
            headers.insert(http::header::CONTENT_TYPE, HeaderValue::from_static("application/json"));
            let mut ctx = ctx.resp(StatusCode::OK, headers, Body::default());
            ctx.response.set_body(
                serde_json::to_string(&TardisResp {
                    code: "200".to_string(),
                    msg: "".to_string(),
                    data: Some(auth_res_serv::get_apis_json()?),
                })
                .map_err(|e| TardisError::bad_request(&format!("[Plugin.Auth] fetch_server_config serde error: {e}"), ""))?,
            );
            return Ok((true, ctx));
        }

        let is_true_mix_req = self.get_is_true_mix_req_from_header(ctx.request.get_headers());

        if self.auth_config.strict_security_mode && !is_true_mix_req {
            log::debug!("[Plugin.Auth] handle mix request");
            let mut ctx = mix_req_to_ctx(&self.auth_config, &self.mix_replace_url, ctx).await?;
            ctx.request.set_header_str(&self.header_is_mix_req, "true")?;
            return Ok((false, ctx));
        }
        ctx.request.set_header_str(&self.header_is_mix_req, "false")?;
        let (mut auth_req, req_body) = ctx_to_auth_req(&self.auth_path_ignore_prefix, &mut ctx).await?;

        match auth_kernel_serv::auth(&mut auth_req, is_true_mix_req).await {
            Ok(auth_result) => {
                if log::level_enabled!(log::Level::TRACE) {
                    log::trace!("[Plugin.Auth] auth return ok {:?}", auth_result);
                } else if log::level_enabled!(log::Level::DEBUG) {
                    if let Some(ctx) = &auth_result.ctx {
                        log::debug!("[Plugin.Auth] auth return ok ctx:{ctx}",);
                    } else {
                        log::debug!("[Plugin.Auth] auth return ok ctx:None",);
                    };
                }

                if auth_result.e.is_none() {
                    ctx = success_auth_result_to_ctx(auth_result, req_body.into(), ctx)?;
                } else if let Some(e) = auth_result.e {
                    log::info!("[Plugin.Auth] auth failed:{e}");
                    ctx.set_action(SgRouteFilterRequestAction::Response);
                    ctx.response.set_status_code(StatusCode::from_str(&e.code).unwrap_or(StatusCode::BAD_GATEWAY));
                    ctx.response.set_body(json!({"code":format!("{}-gateway-cert-error",e.code),"message":e.message}).to_string());
                    return Ok((false, ctx));
                };
                Ok((true, ctx))
            }
            Err(e) => {
                log::info!("[Plugin.Auth] auth return error {:?}", e);
                ctx.set_action(SgRouteFilterRequestAction::Response);
                ctx.response.set_body(format!("[Plugin.Auth] auth return error:{e}"));
                Ok((false, ctx))
            }
        }
    }

    async fn resp_filter(&self, _: &str, mut ctx: SgRoutePluginContext) -> TardisResult<(bool, SgRoutePluginContext)> {
        let head_key_crypto = self.auth_config.head_key_crypto.clone();

        // Return encryption will be skipped in three cases: there is no encryption header,
        // the http code request is unsuccessful, and it is the return of the mix request
        // (the inner request return has been encrypted once)
        if ctx.request.get_headers().get(&head_key_crypto).is_none() || !ctx.response.status_code.is_success() || self.get_is_true_mix_req_from_header(ctx.request.get_headers()) {
            return Ok((true, ctx));
        }

        let crypto_value = ctx.request.get_headers().get(&head_key_crypto).expect("").clone();
        let ctx_resp_headers = ctx.response.get_headers_mut();
        ctx_resp_headers.insert(
            HeaderName::try_from(head_key_crypto.clone()).map_err(|e| TardisError::internal_error(&format!("[Plugin.Auth] get header error: {e:?}"), ""))?,
            crypto_value,
        );

        for exclude_path in self.auth_config.exclude_encrypt_decrypt_path.clone() {
            if ctx.request.get_uri().path().starts_with(&exclude_path) {
                return Ok((true, ctx));
            }
        }
        let encrypt_resp = auth_crypto_serv::encrypt_body(&ctx_to_auth_encrypt_req(&mut ctx).await?).await?;
        ctx.response.get_headers_mut().extend(hashmap_header_to_headermap(encrypt_resp.headers)?);
        ctx.response.headers.remove(hyper::header::TRANSFER_ENCODING);
        ctx.response.set_body(encrypt_resp.body);
        self.cors(&mut ctx)?;

        Ok((true, ctx))
    }
}

async fn mix_req_to_ctx(auth_config: &AuthConfig, mix_replace_url: &str, mut ctx: SgRoutePluginContext) -> TardisResult<SgRoutePluginContext> {
    let body = ctx.request.take_body_into_bytes().await?;
    let string_body = String::from_utf8_lossy(&body).trim_matches('"').to_string();
    if string_body.is_empty() {
        TardisError::custom("502", "[Plugin.Auth.MixReq] body can't be empty", "502-parse_mix_req-parse-error");
    }
    let mut req_headers = ctx.request.get_headers().iter().map(|(k, v)| (k.as_str().to_string(), v.to_str().expect("error parse header value to str").to_string())).collect();
    let (body, crypto_headers) = auth_crypto_serv::decrypt_req(&req_headers, &Some(string_body), true, true, auth_config).await?;
    req_headers.remove(&auth_config.head_key_crypto);
    req_headers.remove(&auth_config.head_key_crypto.to_ascii_lowercase());

    let body = body.ok_or_else(|| TardisError::custom("502", "[Plugin.Auth.MixReq] decrypt body can't be empty", "502-parse_mix_req-parse-error"))?;

    let mix_body = TardisFuns::json.str_to_obj::<MixRequestBody>(&body)?;
    ctx.set_action(SgRouteFilterRequestAction::Redirect);
    let mut true_uri = Url::from_str(&ctx.request.get_uri().to_string().replace(mix_replace_url, &mix_body.uri))
        .map_err(|e| TardisError::custom("502", &format!("[Plugin.Auth.MixReq] url parse err {e}"), "502-parse_mix_req-url-error"))?;
    true_uri.set_path(&true_uri.path().replace("//", "/"));
    true_uri.set_query(Some(&if let Some(old_query) = true_uri.query() {
        format!("{}&_t={}", old_query, mix_body.ts)
    } else {
        format!("_t={}", mix_body.ts)
    }));
    ctx.request.set_uri(true_uri.as_str().parse().map_err(|e| TardisError::custom("502", &format!("[Plugin.Auth.MixReq] uri parse error: {}", e), ""))?);
    ctx.request.set_method(
        Method::from_str(&mix_body.method.to_ascii_uppercase())
            .map_err(|e| TardisError::custom("502", &format!("[Plugin.Auth.MixReq] method parse err {e}"), "502-parse_mix_req-method-error"))?,
    );

    let mut headers = req_headers;
    headers.extend(mix_body.headers);
    headers.extend(crypto_headers.unwrap_or_default());

    ctx.request.set_headers(
        headers
            .into_iter()
            .map(|(k, v)| {
                Ok::<_, TardisError>((
                    HeaderName::from_str(&k).map_err(|e| TardisError::format_error(&format!("[Plugin.Auth] error parse str {k} to header name :{e}"), ""))?,
                    HeaderValue::from_str(&v).map_err(|e| TardisError::format_error(&format!("[Plugin.Auth] error parse str {v} to header value :{e}"), ""))?,
                ))
            })
            .collect::<TardisResult<HeaderMap<HeaderValue>>>()?,
    );

    let real_ip = ctx.request.get_remote_addr().ip().to_string();
    let forwarded_for = match ctx.request.get_headers().get("X-Forwarded-For") {
        Some(forwarded) => {
            format!(
                "{},{}",
                forwarded.to_str().map_err(|e| TardisError::custom(
                    "502",
                    &format!("[Plugin.Auth.MixReq] X-Forwarded-For header value parse err {e}"),
                    "502-parse_mix_req-url-error"
                ))?,
                real_ip
            )
        }
        None => real_ip,
    };
    ctx.request.set_header_str("X-Forwarded-For", &forwarded_for)?;
    ctx.request.set_header_str(hyper::header::CONTENT_LENGTH.as_str(), mix_body.body.as_bytes().len().to_string().as_str())?;
    ctx.request.set_body(mix_body.body);
    Ok(ctx)
}

/// # Convert SgRoutePluginContext to AuthReq
/// Prepare the AuthReq required for the authentication process.
/// The authentication process requires a URL that has not been rewritten.
async fn ctx_to_auth_req(ignore_prefix: &str, ctx: &mut SgRoutePluginContext) -> TardisResult<(AuthReq, Bytes)> {
    let url = ctx.request.get_uri_raw().clone();
    let scheme = url.scheme().map(|s| s.to_string()).unwrap_or("http".to_string());
    let headers = headermap_header_to_hashmap(ctx.request.get_headers())?;
    let req_body = ctx.request.take_body_into_bytes().await?;
    let body = String::from_utf8_lossy(&req_body).trim_matches('"').to_string();
    Ok((
        AuthReq {
            scheme: scheme.clone(),
            path: url.path().replace(ignore_prefix, "").to_string(),
            query: url
                .query()
                .filter(|q| !q.is_empty())
                .map(|q| {
                    q.split('&')
                        .filter(|s| !s.is_empty())
                        .map(|s| s.split('=').collect::<Vec<_>>())
                        .filter(|a| a.len() == 2)
                        .map(|a| (a[0].to_string(), a[1].to_string()))
                        .collect()
                })
                .unwrap_or_default(),
            method: ctx.request.get_method().to_string(),
            host: url.host().unwrap_or("127.0.0.1").to_string(),
            port: url.port().map(|p| p.as_u16()).unwrap_or_else(|| if scheme == "https" { 443 } else { 80 }),
            headers,
            body: if body.is_empty() { None } else { Some(body) },
        },
        req_body,
    ))
}

fn success_auth_result_to_ctx(auth_result: AuthResult, old_req_body: Body, mut ctx: SgRoutePluginContext) -> TardisResult<SgRoutePluginContext> {
    ctx.set_cert_info(SGIdentInfo {
        id: auth_result.ctx.as_ref().and_then(|ctx| ctx.account_id.clone()).unwrap_or_default(),
        name: None,
        roles: auth_result
            .ctx
            .as_ref()
            .and_then(|ctx| ctx.roles.clone())
            .map(|role| role.iter().map(|r| SGRoleInfo { id: r.to_string(), name: None }).collect::<Vec<_>>())
            .unwrap_or_default(),
    });
    let auth_resp = AuthResp::from_result(auth_result);
    let new_headers = hashmap_header_to_headermap(auth_resp.headers.clone())?;
    ctx.request.set_headers(new_headers);
    if let Some(new_body) = auth_resp.body {
        ctx.request.set_body(new_body);
    } else {
        ctx.request.set_body(old_req_body);
    }
    Ok(ctx)
}

async fn ctx_to_auth_encrypt_req(ctx: &mut SgRoutePluginContext) -> TardisResult<AuthEncryptReq> {
    let headers = headermap_header_to_hashmap(ctx.response.get_headers())?;
    let body_as_bytes = ctx.response.take_body_into_bytes().await?;
    let body = String::from_utf8_lossy(&body_as_bytes);
    if !body.is_empty() {
        ctx.set_ext(plugin_constants::BEFORE_ENCRYPT_BODY, body.to_string());
    }
    log::trace!("[Plugin.Auth] Before Encrypt Body {}", body.to_string());
    Ok(AuthEncryptReq {
        headers,
        body: String::from(body),
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

fn headermap_header_to_hashmap(old_headers: &HeaderMap) -> TardisResult<HashMap<String, String>> {
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
#[allow(clippy::unwrap_used)]
mod tests;
