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
    http::{self, HeaderMap, HeaderName, HeaderValue},
    hyper::{body::Bytes, Body, Method},
    plugins::{
        context::{SgRouteFilterRequestAction, SgRoutePluginContext},
        filters::{BoxSgPluginFilter, SgPluginFilter, SgPluginFilterAccept, SgPluginFilterDef},
    },
};
use spacegate_kernel::{
    hyper::StatusCode,
    plugins::{
        context::{SGIdentInfo, SGRoleInfo},
        filters::SgPluginFilterInitDto,
    },
};
use std::{collections::HashMap, str::FromStr, sync::{Arc, OnceLock}};
use tardis::{
    async_trait,
    basic::{error::TardisError, result::TardisResult, tracing::TardisTracing},
    config::config_dto::{AppConfig, CacheConfig, DBConfig, FrameworkConfig, LogConfig, TardisConfig, WebServerConfig},
    log,
    serde_json::{self, json, Value},
    tokio::{sync::RwLock, task::JoinHandle},
    url::Url,
    web::web_resp::TardisResp,
    TardisFuns,
};

use super::plugin_constants;
#[allow(clippy::type_complexity)]
static INSTANCE: OnceLock<Arc<RwLock<Option<(String, JoinHandle<()>)>>>> = OnceLock::new();

pub const CODE: &str = "auth";
pub struct SgFilterAuthDef;

impl SgPluginFilterDef for SgFilterAuthDef {
    fn get_code(&self) -> &str {
        CODE
    }
    fn inst(&self, spec: serde_json::Value) -> TardisResult<BoxSgPluginFilter> {
        let filter = TardisFuns::json.json_to_obj::<SgFilterAuth>(spec)?;
        Ok(filter.boxed())
    }

    fn get_code(&self) -> &str {
        CODE
    }
}

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
            fetch_server_config_path: "/starsysApi/auth/auth/apis".to_string(),
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
        let config_md5 = TardisFuns::crypto.digest.md5(TardisFuns::json.obj_to_string(self)?)?;

        let mut instance = INSTANCE.get_or_init(Default::default).write().await;
        if let Some((md5, handle)) = instance.as_ref() {
            if config_md5.eq(md5) {
                log::debug!("[SG.Filter.Auth] have not found config change");
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
                web_server: WebServerConfig {
                    enabled: false,
                    ..Default::default()
                },
                db: DBConfig {
                    enabled: false,
                    ..Default::default()
                },
                cache: CacheConfig {
                    enabled: true,
                    url: if self.cache_url.is_empty() {
                        if let Some(redis_url) = init_dto.gateway_parameters.redis_url.clone() {
                            redis_url
                        } else {
                            "redis://127.0.0.1:6379".to_string()
                        }
                    } else {
                        self.cache_url.clone()
                    },
                    ..Default::default()
                },
                log: init_dto.gateway_parameters.log_level.as_ref().map(|l| LogConfig { level: l.clone() }),
                ..Default::default()
            },
        })
        .await?;

        let handle = auth_initializer::init().await?;
        *instance = Some((config_md5, handle));
        log::info!("[SG.Filter.Auth] init done");
        if let Some(log_level) = &init_dto.gateway_parameters.log_level {
            let _ = TardisTracing::update_log_level_by_domain_code(crate::DOMAIN_CODE, log_level);
        }
        Ok(())
    }

    async fn destroy(&self) -> TardisResult<()> {
        Ok(())
    }

    async fn req_filter(&self, _: &str, mut ctx: SgRoutePluginContext) -> TardisResult<(bool, SgRoutePluginContext)> {
        if ctx.request.get_method() == http::Method::OPTIONS {
            return Ok((true, ctx));
        }

        if ctx.request.get_method().eq(&Method::GET) && ctx.request.get_uri_raw().path() == self.fetch_server_config_path.as_str() {
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
            let mut ctx = mix_req_to_ctx(&self.auth_config, ctx).await?;
            ctx.request.set_header_str(&self.header_is_mix_req, "true")?;
            return Ok((false, ctx));
        }
        ctx.request.set_header_str(&self.header_is_mix_req, "false")?;
        let (mut auth_req, req_body) = ctx_to_auth_req(&mut ctx).await?;

        match auth_kernel_serv::auth(&mut auth_req, is_true_mix_req).await {
            Ok(auth_result) => {
                log::debug!("[Plugin.Auth] auth return ok {:?}", auth_result);
                if auth_result.e.is_none() {
                    ctx = success_auth_result_to_ctx(auth_result, req_body.into(), ctx)?;
                } else if let Some(e) = auth_result.e {
                    ctx.set_action(SgRouteFilterRequestAction::Response);
                    ctx.response.set_status_code(StatusCode::from_str(&e.code).unwrap_or(StatusCode::BAD_GATEWAY));
                    ctx.response.set_body(json!({"code":format!("{}-gateway-cert-error",e.code),"message":e.message}).to_string());
                    return Ok((false, ctx));
                };
                Ok((true, ctx))
            }
            Err(e) => {
                log::warn!("[Plugin.Auth] auth return error {:?}", e);
                ctx.set_action(SgRouteFilterRequestAction::Response);
                ctx.response.set_body(format!("[Plugin.Auth] auth return error:{e}"));
                Ok((false, ctx))
            }
        }
    }

    async fn resp_filter(&self, _: &str, mut ctx: SgRoutePluginContext) -> TardisResult<(bool, SgRoutePluginContext)> {
        let head_key_crypto = self.auth_config.head_key_crypto.clone();

        if ctx.request.get_headers().get(&head_key_crypto).is_none() || self.get_is_true_mix_req_from_header(ctx.request.get_headers()) {
            return Ok((true, ctx));
        }

        let crypto_value = ctx.request.get_headers().get(&head_key_crypto).expect("").clone();
        let ctx_resp_headers = ctx.response.get_headers_mut();
        ctx_resp_headers.insert(
            HeaderName::try_from(head_key_crypto.clone()).map_err(|e| TardisError::internal_error(&format!("[Plugin.Auth] get header error: {e:?}"), ""))?,
            crypto_value,
        );
        let encrypt_resp = auth_crypto_serv::encrypt_body(&ctx_to_auth_encrypt_req(&mut ctx).await?).await?;
        ctx.response.get_headers_mut().extend(hashmap_header_to_headermap(encrypt_resp.headers)?);
        ctx.response.set_body(encrypt_resp.body);
        self.cors(&mut ctx)?;

        Ok((true, ctx))
    }
}

async fn mix_req_to_ctx(auth_config: &AuthConfig, mut ctx: SgRoutePluginContext) -> TardisResult<SgRoutePluginContext> {
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
    let mut true_uri = Url::from_str(&ctx.request.get_uri().to_string().replace("apis", &mix_body.uri))
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
    ctx.request.set_body(mix_body.body);
    Ok(ctx)
}

async fn ctx_to_auth_req(ctx: &mut SgRoutePluginContext) -> TardisResult<(AuthReq, Bytes)> {
    let url = ctx.request.get_uri().clone();
    let scheme = url.scheme().map(|s| s.to_string()).unwrap_or("http".to_string());
    let headers = headermap_header_to_hashmap(ctx.request.get_headers())?;
    let req_body = ctx.request.take_body_into_bytes().await?;
    let body = String::from_utf8_lossy(&req_body).trim_matches('"').to_string();
    Ok((
        AuthReq {
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
            method: ctx.request.get_method().to_string(),
            host: url.host().unwrap_or("127.0.0.1").to_string(),
            port: url.port().map(|p| p.as_u16()).unwrap_or_else(|| if scheme == "https" { 443 } else { 80 }),
            headers,
            body: Some(body),
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
mod tests {
    use std::env;

    use bios_auth::auth_constants;

    use spacegate_kernel::config::gateway_dto::SgParameters;
    use spacegate_kernel::http::{Method, Uri, Version};
    use spacegate_kernel::hyper::{self, Body, StatusCode};
    use tardis::basic::dto::TardisContext;
    use tardis::crypto::crypto_sm2_4::{TardisCryptoSm2, TardisCryptoSm2PrivateKey};
    use tardis::{
        test::test_container::TardisTestContainer,
        testcontainers::{self, clients::Cli, images::redis::Redis, Container},
        tokio,
    };

    use super::*;

    #[tokio::test]
    async fn test_auth_plugin_ctx() {
        env::set_var("RUST_LOG", "debug,bios_auth=trace,tardis=trace");
        tracing_subscriber::fmt::init();

        let docker = testcontainers::clients::Cli::default();
        let _x = docker_init(&docker).await.unwrap();

        let mut filter_auth = SgFilterAuth {
            cache_url: env::var("TARDIS_FW.CACHE.URL").unwrap(),
            ..Default::default()
        };

        filter_auth
            .init(&SgPluginFilterInitDto {
                gateway_name: "".to_string(),
                gateway_parameters: SgParameters {
                    redis_url: None,
                    log_level: None,
                    lang: None,
                },
                http_route_rules: vec![],
                attached_level: spacegate_kernel::plugins::filters::SgAttachedLevel::Gateway,
            })
            .await
            .unwrap();

        let cache_client = TardisFuns::cache_by_module_or_default(auth_constants::DOMAIN_CODE);

        let mut header = HeaderMap::new();
        header.insert("Bios-Token", "aaa".parse().unwrap());
        let ctx = SgRoutePluginContext::new_http(
            Method::POST,
            Uri::from_static("http://sg.idealworld.group/test1"),
            Version::HTTP_11,
            header,
            Body::from("test"),
            "127.0.0.1:8080".parse().unwrap(),
            "".to_string(),
            None,
        );
        let (is_ok, mut before_filter_ctx) = filter_auth.req_filter("", ctx).await.unwrap();
        assert!(!is_ok);
        let req_body = before_filter_ctx.response.take_body_into_bytes().await.unwrap();
        let req_body = String::from_utf8_lossy(&req_body).to_string();
        assert!(!req_body.is_empty());
        assert_eq!(req_body, "[Auth] Token [aaa] is not legal");

        cache_client.set(&format!("{}tokenxxx", filter_auth.auth_config.cache_key_token_info), "default,accountxxx").await.unwrap();
        cache_client
            .hset(
                &format!("{}accountxxx", filter_auth.auth_config.cache_key_account_info),
                "",
                "{\"own_paths\":\"\",\"owner\":\"account1\",\"roles\":[\"r001\"],\"groups\":[\"g001\"]}",
            )
            .await
            .unwrap();

        let mut header = HeaderMap::new();
        header.insert("Bios-Token", "tokenxxx".parse().unwrap());
        let ctx = SgRoutePluginContext::new_http(
            Method::POST,
            Uri::from_static("http://sg.idealworld.group/test1"),
            Version::HTTP_11,
            header,
            Body::from("test"),
            "127.0.0.1:8080".parse().unwrap(),
            "".to_string(),
            None,
        );
        let (is_ok, mut before_filter_ctx) = filter_auth.req_filter("", ctx).await.unwrap();
        assert!(is_ok);
        let ctx = decode_context(before_filter_ctx.request.get_headers());

        assert_eq!(ctx.own_paths, "");
        assert_eq!(ctx.owner, "account1");
        assert_eq!(ctx.roles, vec!["r001"]);
        assert_eq!(ctx.groups, vec!["g001"]);

        cache_client.set(&format!("{}tokenxxx", filter_auth.auth_config.cache_key_token_info), "default,accountxxx").await.unwrap();
        cache_client
            .hset(
                &format!("{}accountxxx", filter_auth.auth_config.cache_key_account_info),
                "",
                "{\"own_paths\":\"tenant1\",\"owner\":\"account1\",\"roles\":[\"r001\"],\"groups\":[\"g001\"]}",
            )
            .await
            .unwrap();
        let mut header = HeaderMap::new();
        header.insert("Bios-Token", "tokenxxx".parse().unwrap());
        let ctx = SgRoutePluginContext::new_http(
            Method::POST,
            Uri::from_static("http://sg.idealworld.group/test1"),
            Version::HTTP_11,
            header,
            Body::from("test"),
            "127.0.0.1:8080".parse().unwrap(),
            "".to_string(),
            None,
        );
        let (is_ok, mut before_filter_ctx) = filter_auth.req_filter("", ctx).await.unwrap();
        assert!(is_ok);
        let ctx = decode_context(before_filter_ctx.request.get_headers());

        assert_eq!(ctx.own_paths, "tenant1");
        assert_eq!(ctx.owner, "account1");
        assert_eq!(ctx.roles, vec!["r001"]);
        assert_eq!(ctx.groups, vec!["g001"]);
    }

    fn decode_context(headers: &HeaderMap) -> TardisContext {
        let config = TardisFuns::cs_config::<AuthConfig>(auth_constants::DOMAIN_CODE);
        let ctx = headers.get(&config.head_key_context).unwrap();
        let ctx = TardisFuns::crypto.base64.decode(ctx.to_str().unwrap()).unwrap();
        TardisFuns::json.str_to_obj(&ctx).unwrap()
    }

    #[tokio::test]
    async fn test_auth_plugin_crypto() {
        env::set_var("RUST_LOG", "debug,bios_auth=trace,tardis=trace");
        tracing_subscriber::fmt::init();

        let docker = testcontainers::clients::Cli::default();
        let _x = docker_init(&docker).await.unwrap();

        let mut filter_auth = SgFilterAuth {
            cache_url: env::var("TARDIS_FW.CACHE.URL").unwrap(),
            ..Default::default()
        };

        filter_auth
            .init(&SgPluginFilterInitDto {
                gateway_name: "".to_string(),
                gateway_parameters: SgParameters {
                    redis_url: None,
                    log_level: None,
                    lang: None,
                },
                http_route_rules: vec![],
                attached_level: spacegate_kernel::plugins::filters::SgAttachedLevel::Gateway,
            })
            .await
            .unwrap();

        let ctx = SgRoutePluginContext::new_http(
            Method::GET,
            Uri::from_str(&format!("http://sg.idealworld.group{}", filter_auth.fetch_server_config_path)).unwrap(),
            Version::HTTP_11,
            HeaderMap::new(),
            Body::from(""),
            "127.0.0.1:8080".parse().unwrap(),
            "".to_string(),
            None,
        );
        let (_, mut before_filter_ctx) = filter_auth.req_filter("", ctx).await.unwrap();
        let mut server_config_resp = before_filter_ctx.build_response().await.unwrap();
        let data: Value = serde_json::from_str(&String::from_utf8_lossy(
            &hyper::body::to_bytes(server_config_resp.body_mut()).await.unwrap().iter().cloned().collect::<Vec<u8>>(),
        ))
        .unwrap();

        let pub_key = data["pub_key"].as_str().unwrap();
        let server_sm2 = TardisCryptoSm2 {};
        let server_public_key = server_sm2.new_public_key_from_public_key(pub_key).unwrap();

        let front_pri_key = TardisFuns::crypto.sm2.new_private_key().unwrap();
        let front_pub_key = TardisFuns::crypto.sm2.new_public_key(&front_pri_key).unwrap();

        let test_body_value = r#"test_body_value!@#$%^&*():"中文测试"#;
        //dont need to decrypt
        let header = HeaderMap::new();
        let ctx = SgRoutePluginContext::new_http(
            Method::POST,
            Uri::from_static("http://sg.idealworld.group/test1"),
            Version::HTTP_11,
            header,
            Body::from(test_body_value),
            "127.0.0.1:8080".parse().unwrap(),
            "".to_string(),
            None,
        );
        let (is_ok, mut before_filter_ctx) = filter_auth.req_filter("", ctx).await.unwrap();
        assert!(is_ok);
        let req_body = before_filter_ctx.request.dump_body().await.unwrap();
        assert!(!req_body.is_empty());
        let req_body = req_body.to_vec();
        let req_body = String::from_utf8(req_body).unwrap();
        assert_eq!(req_body, test_body_value.to_string());

        //=========request============
        let mut header = HeaderMap::new();
        let (crypto_data, bios_crypto_value) = crypto_req(
            test_body_value,
            server_public_key.serialize().unwrap().as_ref(),
            front_pub_key.serialize().unwrap().as_ref(),
            true,
        );
        header.insert("Bios-Crypto", bios_crypto_value.parse().unwrap());
        let ctx = SgRoutePluginContext::new_http(
            Method::POST,
            Uri::from_static("http://sg.idealworld.group/test1"),
            Version::HTTP_11,
            header,
            Body::from(crypto_data),
            "127.0.0.1:8080".parse().unwrap(),
            "".to_string(),
            None,
        );
        let (is_ok, mut before_filter_ctx) = filter_auth.req_filter("", ctx).await.unwrap();
        assert!(is_ok);
        let req_body = before_filter_ctx.request.dump_body().await.unwrap();
        assert!(!req_body.is_empty());
        let req_body = req_body.to_vec();
        let req_body = String::from_utf8(req_body).unwrap();
        assert_eq!(req_body, test_body_value.to_string());

        //======response============
        let mock_resp = r#"mock_resp:test_body_value!@#$%^&*():"中文测试"#;
        let mut header = HeaderMap::new();
        header.insert("Test_Header", "test_header".parse().unwrap());
        let ctx = before_filter_ctx.resp(StatusCode::OK, header, Body::from(mock_resp));

        let (is_ok, mut before_filter_ctx) = filter_auth.resp_filter("", ctx).await.unwrap();
        assert!(is_ok);
        let resp_body = before_filter_ctx.response.dump_body().await.unwrap();
        assert!(!resp_body.is_empty());
        let resp_body = resp_body.to_vec();
        let resp_body = String::from_utf8(resp_body).unwrap();
        let resp_body = crypto_resp(
            &resp_body,
            before_filter_ctx.response.get_headers().get("Bios-Crypto").unwrap().to_str().unwrap(),
            &front_pri_key,
        );
        println!("req_body:{req_body} mock_resp:{mock_resp}");
        assert_eq!(resp_body, mock_resp.to_string());

        filter_auth.destroy().await.unwrap();
    }

    fn crypto_req(body: &str, serv_pub_key: &str, front_pub_key: &str, need_crypto_resp: bool) -> (String, String) {
        let pub_key = TardisFuns::crypto.sm2.new_public_key_from_public_key(serv_pub_key).unwrap();

        let sm4_key = TardisFuns::crypto.key.rand_16_hex().unwrap();
        let sm4_iv = TardisFuns::crypto.key.rand_16_hex().unwrap();

        let data = TardisFuns::crypto.sm4.encrypt_cbc(body, &sm4_key, &sm4_iv).unwrap();
        let sign_data = TardisFuns::crypto.digest.sm3(&data).unwrap();

        let sm4_encrypt = if need_crypto_resp {
            pub_key.encrypt(&format!("{sign_data} {sm4_key} {sm4_iv} {front_pub_key}",)).unwrap()
        } else {
            pub_key.encrypt(&format!("{sign_data} {sm4_key} {sm4_iv}",)).unwrap()
        };
        let base64_encrypt = TardisFuns::crypto.base64.encode(sm4_encrypt);
        (data, base64_encrypt)
    }

    fn crypto_resp(body: &str, crypto_header: &str, front_pri_key: &TardisCryptoSm2PrivateKey) -> String {
        let decode_base64 = TardisFuns::crypto.base64.decode(crypto_header).unwrap();
        let decrypt_key = front_pri_key.decrypt(&decode_base64).unwrap();
        let splits: Vec<_> = decrypt_key.split(' ').collect();
        if splits.len() != 3 {
            panic!("splits:{:?}", splits);
        }

        let sign_data = splits[0];
        let sm4_key = splits[1];
        let sm4_iv = splits[2];
        let gen_sign_data = TardisFuns::crypto.digest.sm3(body).unwrap();
        assert_eq!(sign_data, gen_sign_data);
        TardisFuns::crypto.sm4.decrypt_cbc(body, sm4_key, sm4_iv).unwrap()
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
