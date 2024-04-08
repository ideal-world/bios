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
use spacegate_shell::{
    hyper::{
        self, header,
        http::{self, HeaderMap, HeaderName, HeaderValue, StatusCode},
        Method, Request, Response,
    },
    kernel::{extension::Reflect, helper_layers::function::Inner},
    plugin::{Plugin, PluginConfig, PluginError},
    BoxError, SgBody,
};
use std::{
    collections::HashMap,
    str::FromStr,
    sync::{Arc, Once, OnceLock},
};
use tardis::{
    basic::{error::TardisError, result::TardisResult},
    config::config_dto::CacheModuleConfig,
    log::{self, warn},
    serde_json::{self, json},
    tokio::{sync::RwLock, task::JoinHandle},
    url::Url,
    web::web_resp::TardisResp,
    TardisFuns,
};
use tardis::{config::config_dto::TardisComponentConfig, web::poem_openapi::types::Type};

use crate::extension::{
    before_encrypt_body::BeforeEncryptBody,
    cert_info::{CertInfo, RoleInfo},
    request_crypto_status::{HeadCryptoKey, RequestCryptoParam},
};

pub const CODE: &str = "auth";
#[allow(clippy::type_complexity)]
static INSTANCE: OnceLock<Arc<RwLock<Option<(String, JoinHandle<()>)>>>> = OnceLock::new();

#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct SgPluginAuthConfig {
    pub auth_config: AuthConfig,
    pub cache_url: String,
    pub cors_allow_origin: String,
    pub cors_allow_methods: String,
    pub cors_allow_headers: String,
    pub header_is_mix_req: String,
    pub fetch_server_config_path: String,
    pub mix_replace_url: String,
    pub auth_path_ignore_prefix: String,
}

impl SgPluginAuthConfig {
    pub async fn setup_tardis(&self) -> Result<(), BoxError> {
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

        let cache_url: Url = if self.cache_url.is_empty() {
            //todo get from gateways
            // init_dto.gateway_parameters.redis_url.as_deref().unwrap_or("redis://127.0.0.1:6379")
            "redis://127.0.0.1:6379"
        } else {
            self.cache_url.as_str()
        }
        .parse()
        .map_err(|e| TardisError::internal_error(&format!("[SG.Filter.Auth]invalid redis url: {e:?}"), "-1"))?;

        let mut tardis_config = tardis::TardisFuns::clone_config();
        tardis_config.cs.insert(bios_auth::auth_constants::DOMAIN_CODE.to_string(), serde_json::to_value(self.auth_config.clone())?);
        match tardis_config.fw.cache {
            Some(ref mut cache_config) => {
                cache_config.modules.insert(bios_auth::auth_constants::DOMAIN_CODE.to_string(), CacheModuleConfig::builder().url(cache_url).build());
            }
            None => {
                tardis_config.fw.cache = Some(
                    <TardisComponentConfig<CacheModuleConfig>>::builder()
                        .default(CacheModuleConfig::builder().url(cache_url.clone()).build())
                        .modules([(bios_auth::auth_constants::DOMAIN_CODE.to_string(), CacheModuleConfig::builder().url(cache_url).build())])
                        .build(),
                );
            }
        }
        tardis::TardisFuns::hot_reload(tardis_config).await?;
        let handle = auth_initializer::init_without_webserver().await?;
        *instance = Some((config_md5, handle));
        log::info!("[SG.Filter.Auth] init done");
        Ok(())
    }
}

impl Default for SgPluginAuthConfig {
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
#[derive(Clone)]
pub struct AuthPlugin {
    auth_config: AuthConfig,
    cors_allow_origin: HeaderValue,
    cors_allow_methods: HeaderValue,
    cors_allow_headers: HeaderValue,
    header_is_mix_req: HeaderName,
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

impl From<SgPluginAuthConfig> for AuthPlugin {
    fn from(value: SgPluginAuthConfig) -> Self {
        AuthPlugin {
            auth_config: value.auth_config,
            cors_allow_origin: HeaderValue::from_str(&value.cors_allow_origin).expect("cors_allow_origin is invalid"),
            cors_allow_methods: HeaderValue::from_str(&value.cors_allow_methods).expect("cors_allow_methods is invalid"),
            cors_allow_headers: HeaderValue::from_str(&value.cors_allow_headers).expect("cors_allow_headers is invalid"),
            header_is_mix_req: HeaderName::from_str(&value.header_is_mix_req).expect("header_is_mix_req is invalid"),
            fetch_server_config_path: value.fetch_server_config_path,
            mix_replace_url: value.mix_replace_url,
            auth_path_ignore_prefix: value.auth_path_ignore_prefix,
        }
    }
}

impl<'de> Deserialize<'de> for AuthPlugin {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        SgPluginAuthConfig::deserialize(deserializer).map(|config| config.into())
    }
}

impl AuthPlugin {
    fn cors(&self, resp: &mut Response<SgBody>) -> TardisResult<()> {
        resp.headers_mut().insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, self.cors_allow_origin.clone());
        resp.headers_mut().insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, self.cors_allow_origin.clone());
        resp.headers_mut().insert(header::ACCESS_CONTROL_ALLOW_METHODS, self.cors_allow_methods.clone());
        resp.headers_mut().insert(header::ACCESS_CONTROL_ALLOW_HEADERS, self.cors_allow_headers.clone());
        resp.headers_mut().insert(header::ACCESS_CONTROL_MAX_AGE, HeaderValue::from_static("3600000"));
        resp.headers_mut().insert(header::ACCESS_CONTROL_ALLOW_CREDENTIALS, HeaderValue::from_static("TRUE"));
        resp.headers_mut().insert(header::CONTENT_TYPE, HeaderValue::from_static("application/json"));
        Ok(())
    }

    fn is_mix_req(&self, header_map: &HeaderMap<HeaderValue>) -> bool {
        header_map
            .get(&self.header_is_mix_req)
            .map(|v: &HeaderValue| {
                bool::from_str(v.to_str().map_err(|e| TardisError::custom("500", &format!("[SG.Filter.Auth] parse header IS_MIX_REQ error:{e}"), ""))?)
                    .map_err(|e| TardisError::custom("500", &format!("[SG.Filter.Auth] parse header IS_MIX_REQ error:{e}"), ""))
            })
            .transpose()
            .ok()
            .flatten()
            .unwrap_or(false)
    }
}

impl AuthPlugin {
    async fn req(&self, mut req: Request<SgBody>) -> Result<Request<SgBody>, Response<SgBody>> {
        req.extensions_mut().get_mut::<Reflect>().expect("missing reflect").insert(RequestCryptoParam::default());

        for exclude_path in self.auth_config.exclude_encrypt_decrypt_path.clone() {
            if req.uri().path().starts_with(&exclude_path) {
                req.extensions_mut().get_mut::<Reflect>().expect("missing reflect").get_mut::<RequestCryptoParam>().expect("missing request crypto status").is_skip_crypto = true;
            }
        }

        let method = req.method().clone();
        if method == http::Method::OPTIONS {
            return Ok(req);
        }

        log::trace!("[SG.Filter.Auth] request filter info: request path is {}", req.uri().path());
        if method == http::Method::GET && req.uri().path().trim_matches('/') == self.fetch_server_config_path.as_str().trim_matches('/') {
            log::debug!("[SG.Filter.Auth] request path hit fetch server config path: {}", self.fetch_server_config_path);
            let mock_resp = Response::builder()
                .header(http::header::CONTENT_TYPE, HeaderValue::from_static("application/json"))
                .status(http::StatusCode::OK)
                .body(SgBody::full(
                    serde_json::to_vec(&TardisResp {
                        code: "200".to_string(),
                        msg: "".to_string(),
                        data: Some(auth_res_serv::get_apis_json().map_err(PluginError::internal_error::<AuthPlugin>)?),
                    })
                    .expect("TardisResp should be a valid json"),
                ))
                .map_err(PluginError::internal_error::<AuthPlugin>)?;
            return Err(mock_resp);
        }

        let is_true_mix_req = self.is_mix_req(req.headers());

        if self.auth_config.strict_security_mode && !is_true_mix_req {
            log::debug!("[SG.Filter.Auth] handle mix request");
            return Ok(handle_mix_req(&self.auth_config, &self.mix_replace_url, req).await.map_err(PluginError::internal_error::<AuthPlugin>)?);
        }
        req.headers_mut().append(&self.header_is_mix_req, HeaderValue::from_static("false"));

        let (mut auth_req, mut req) = req_to_auth_req(&self.auth_path_ignore_prefix, req).await.map_err(PluginError::internal_error::<AuthPlugin>)?;

        match auth_kernel_serv::auth(&mut auth_req, is_true_mix_req).await {
            Ok(auth_result) => {
                if log::level_enabled!(log::Level::TRACE) {
                    log::trace!("[SG.Filter.Auth] auth return ok {:?}", auth_result);
                } else if log::level_enabled!(log::Level::DEBUG) {
                    if let Some(ctx) = &auth_result.ctx {
                        log::debug!("[SG.Filter.Auth] auth return ok ctx:{ctx}",);
                    } else {
                        log::debug!("[SG.Filter.Auth] auth return ok ctx:None",);
                    };
                }

                if auth_result.e.is_none() {
                    req = success_auth_result_to_req(auth_result, &self.auth_config, req).map_err(PluginError::internal_error::<AuthPlugin>)?;
                } else if let Some(e) = auth_result.e {
                    log::info!("[SG.Filter.Auth] auth failed:{e}");
                    let err_resp = Response::builder()
                        .header(http::header::CONTENT_TYPE, HeaderValue::from_static("application/json"))
                        .status(StatusCode::from_str(&e.code).unwrap_or(StatusCode::BAD_GATEWAY))
                        .body(SgBody::full(json!({"code":format!("{}-gateways-cert-error",e.code),"message":e.message}).to_string()))
                        .map_err(PluginError::internal_error::<AuthPlugin>)?;
                    return Err(err_resp);
                };
                Ok(req)
            }
            Err(e) => {
                log::info!("[SG.Filter.Auth] auth return error {:?}", e);
                let err_resp = Response::builder()
                    .header(http::header::CONTENT_TYPE, HeaderValue::from_static("application/json"))
                    .status(StatusCode::from_str(&e.code).unwrap_or(StatusCode::BAD_GATEWAY))
                    .body(SgBody::full(format!("[SG.Filter.Auth] auth return error:{e}")))
                    .map_err(PluginError::internal_error::<AuthPlugin>)?;
                Err(err_resp)
            }
        }
    }

    async fn resp(&self, mut resp: Response<SgBody>) -> Result<Response<SgBody>, Response<SgBody>> {
        let head_key_crypto = self.auth_config.head_key_crypto.clone();

        let req_crypto = resp.extensions_mut().remove::<RequestCryptoParam>().unwrap_or_else(RequestCryptoParam::default);

        // Return encryption will be skipped in three cases: there is no encryption header,
        // the http code request is unsuccessful, and it is the return of the mix request
        // (the inner request return has been encrypted once)
        if req_crypto.head_crypto_key.is_none() || !resp.status().is_success() || req_crypto.is_mix {
            return Ok(resp);
        }

        if let HeadCryptoKey::Some(crypto_value) = &req_crypto.head_crypto_key {
            resp.headers_mut().insert(
                HeaderName::try_from(head_key_crypto.clone()).map_err(PluginError::internal_error::<AuthPlugin>)?,
                crypto_value.clone(),
            );
        }

        if req_crypto.is_skip_crypto {
            return Ok(resp);
        }

        let (encrypt_req, mut resp) = resp_to_auth_encrypt_req(resp).await.map_err(PluginError::internal_error::<AuthPlugin>)?;
        let (mut parts, _) = resp.into_parts();
        let encrypt_resp = auth_crypto_serv::encrypt_body(&encrypt_req).await.map_err(PluginError::internal_error::<AuthPlugin>)?;
        parts.headers.extend(hashmap_header_to_headermap(encrypt_resp.headers).map_err(PluginError::internal_error::<AuthPlugin>)?);
        parts.headers.remove(hyper::header::TRANSFER_ENCODING);
        let body = SgBody::full(encrypt_resp.body);
        resp = Response::from_parts(parts, body);
        self.cors(&mut resp).map_err(PluginError::internal_error::<AuthPlugin>)?;

        Ok(resp)
    }
}

async fn handle_mix_req(auth_config: &AuthConfig, mix_replace_url: &str, req: Request<SgBody>) -> Result<Request<SgBody>, BoxError> {
    let (mut parts, mut body) = req.into_parts();
    if !body.is_dumped() {
        body = body.dump().await?;
    }
    let string_body = String::from_utf8_lossy(body.get_dumped().expect("not expect code")).trim_matches('"').to_string();
    if string_body.is_empty() {
        TardisError::custom("500", "[SG.Filter.Auth.MixReq] body can't be empty", "500-parse_mix_req-parse-error");
    }
    let mut req_headers = parts.headers.iter().map(|(k, v)| (k.as_str().to_string(), v.to_str().expect("error parse header value to str").to_string())).collect();
    let (body, crypto_headers) = auth_crypto_serv::decrypt_req(&req_headers, &Some(string_body), true, true, auth_config).await?;
    req_headers.remove(&auth_config.head_key_crypto);
    req_headers.remove(&auth_config.head_key_crypto.to_ascii_lowercase());

    let body = body.ok_or_else(|| TardisError::custom("500", "[SG.Filter.Auth.MixReq] decrypt body can't be empty", "500-parse_mix_req-parse-error"))?;

    let mix_body = TardisFuns::json.str_to_obj::<MixRequestBody>(&body)?;
    // ctx.set_action(SgRouteFilterRequestAction::Redirect);
    let mut true_uri = Url::from_str(&parts.uri.to_string().replace(mix_replace_url, &mix_body.uri))
        .map_err(|e| TardisError::custom("500", &format!("[SG.Filter.Auth.MixReq] url parse err {e}"), "500-parse_mix_req-url-error"))?;
    true_uri.set_path(&true_uri.path().replace("//", "/"));
    true_uri.set_query(Some(&if let Some(old_query) = true_uri.query() {
        format!("{}&_t={}", old_query, mix_body.ts)
    } else {
        format!("_t={}", mix_body.ts)
    }));
    parts.uri = true_uri.as_str().parse().map_err(|e| TardisError::custom("500", &format!("[SG.Filter.Auth.MixReq] uri parse error: {}", e), ""))?;
    parts.method = Method::from_str(&mix_body.method.to_ascii_uppercase())
        .map_err(|e| TardisError::custom("500", &format!("[SG.Filter.Auth.MixReq] method parse err {e}"), "500-parse_mix_req-method-error"))?;

    let mut headers = req_headers;
    headers.extend(mix_body.headers);
    headers.extend(crypto_headers.unwrap_or_default());

    parts.headers.extend(
        headers
            .into_iter()
            .map(|(k, v)| {
                Ok::<_, TardisError>((
                    HeaderName::from_str(&k).map_err(|e| TardisError::format_error(&format!("[SG.Filter.Auth] error parse str {k} to header name :{e}"), ""))?,
                    HeaderValue::from_str(&v).map_err(|e| TardisError::format_error(&format!("[SG.Filter.Auth] error parse str {v} to header value :{e}"), ""))?,
                ))
            })
            .collect::<TardisResult<HeaderMap<HeaderValue>>>()?,
    );

    let new_body = SgBody::full(mix_body.body);

    // ctx.request.set_header_str(&self.header_is_mix_req, "true")?;
    Ok(Request::from_parts(parts, new_body))
}

/// # Convert Request to AuthReq
/// Prepare the AuthReq required for the authentication process.
/// The authentication process requires a URL that has not been rewritten.
async fn req_to_auth_req(ignore_prefix: &str, req: Request<SgBody>) -> TardisResult<(AuthReq, Request<SgBody>)> {
    let (parts, mut body) = req.into_parts();
    if !body.is_dumped() {
        body = body.dump().await.map_err(|e| TardisError::wrap(&format!("[SG.Filter.Auth] dump body error: {e}"), ""))?;
    }
    let url = parts.uri.clone();
    let scheme = url.scheme().map(|s| s.to_string()).unwrap_or("http".to_string());
    let headers = headermap_to_hashmap(&parts.headers)?;
    let req_body = body.get_dumped().expect("missing dump body");
    let string_body = String::from_utf8_lossy(req_body).trim_matches('"').to_string();

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
            method: parts.method.to_string(),
            host: url.host().unwrap_or("127.0.0.1").to_string(),
            port: url.port().map(|p| p.as_u16()).unwrap_or_else(|| if scheme == "https" { 443 } else { 80 }),
            headers,
            body: if string_body.is_empty() { None } else { Some(string_body) },
        },
        Request::from_parts(parts, body),
    ))
}

fn success_auth_result_to_req(auth_result: AuthResult, config: &AuthConfig, req: Request<SgBody>) -> TardisResult<Request<SgBody>> {
    let (mut parts, mut body) = req.into_parts();
    let cert_info = CertInfo {
        id: auth_result.ctx.as_ref().and_then(|ctx| ctx.account_id.clone().or(ctx.ak.clone())).unwrap_or_default(),
        own_paths: auth_result.ctx.as_ref().and_then(|ctx| ctx.own_paths.clone()),
        name: None,
        roles: auth_result
            .ctx
            .as_ref()
            .and_then(|ctx| ctx.roles.clone())
            .map(|role| role.iter().map(|r| RoleInfo { id: r.to_string(), name: None }).collect::<Vec<_>>())
            .unwrap_or_default(),
    };
    parts.extensions.get_mut::<Reflect>().expect("missing reflect").insert(cert_info);

    if let Some(mut resp_headers) = auth_result.resp_headers.clone() {
        if resp_headers.contains_key(&config.head_key_crypto) || resp_headers.contains_key(&config.head_key_crypto.to_lowercase()) {
            parts.extensions.get_mut::<Reflect>().expect("missing reflect").get_mut::<RequestCryptoParam>().expect("missing request crypto status").head_crypto_key =
                HeadCryptoKey::Some(
                    resp_headers
                        .remove(&config.head_key_crypto)
                        .unwrap_or_else(|| resp_headers.remove(&config.head_key_crypto.to_ascii_lowercase()).expect("missing header"))
                        .parse()
                        .map_err(|e| TardisError::format_error(&format!("[SG.Filter.Auth] error parse str :{e}"), ""))?,
                );
        }
    }

    let auth_resp: AuthResp = auth_result.into();
    parts.headers.extend(hashmap_header_to_headermap(auth_resp.headers.clone())?);
    if let Some(new_body) = auth_resp.body {
        parts.headers.insert(
            header::CONTENT_LENGTH,
            HeaderValue::from_str(&format!("{}", new_body.as_bytes().len())).map_err(|e| TardisError::format_error(&format!("[SG.Filter.Auth] error parse str :{e}"), ""))?,
        );
        body = SgBody::full(new_body);
    };
    Ok(Request::from_parts(parts, body))
}

async fn resp_to_auth_encrypt_req(resp: Response<SgBody>) -> TardisResult<(AuthEncryptReq, Response<SgBody>)> {
    let (mut parts, mut body) = resp.into_parts();
    let headers = headermap_to_hashmap(&parts.headers)?;
    if !body.is_dumped() {
        body = body.dump().await.map_err(|e| TardisError::wrap(&format!("[SG.Filter.Auth] dump body error: {e}"), ""))?;
    }
    let resp_body = body.get_dumped().expect("missing dump body");
    if !resp_body.is_empty() {
        parts.extensions.insert(BeforeEncryptBody::new(resp_body.clone()));
    }
    let string_body = String::from_utf8_lossy(resp_body);
    log::trace!("[SG.Filter.Auth] Before Encrypt Body {}", string_body);
    Ok((
        AuthEncryptReq {
            headers,
            body: string_body.to_string(),
        },
        Response::from_parts(parts, body),
    ))
}

fn hashmap_header_to_headermap(old_headers: HashMap<String, String>) -> TardisResult<HeaderMap<HeaderValue>> {
    let mut new_headers = HeaderMap::new();
    for header in old_headers {
        new_headers.insert(
            HeaderName::from_str(&header.0).map_err(|e| TardisError::format_error(&format!("[SG.Filter.Auth] request header error :{e}"), ""))?,
            HeaderValue::from_str(&header.1).map_err(|e| TardisError::format_error(&format!("[SG.Filter.Auth] request header error :{e}"), ""))?,
        );
    }
    Ok(new_headers)
}

fn headermap_to_hashmap(old_headers: &HeaderMap<HeaderValue>) -> TardisResult<HashMap<String, String>> {
    let mut new_headers = HashMap::new();
    for header_name in old_headers.keys() {
        new_headers.insert(
            header_name.to_string(),
            old_headers.get(header_name).expect("").to_str().map_err(|e| TardisError::format_error(&format!("[SG.Filter.Auth] response header error :{e}"), ""))?.to_string(),
        );
    }
    Ok(new_headers)
}

impl Plugin for AuthPlugin {
    const CODE: &'static str = CODE;
    // type MakeLayer = SgPluginAuth;
    fn create(plugin_config: PluginConfig) -> Result<Self, BoxError> {
        let config: SgPluginAuthConfig = serde_json::from_value(plugin_config.spec.clone())?;
        let plugin: AuthPlugin = config.clone().into();
        let tardis_init = Arc::new(Once::new());
        {
            let tardis_init = tardis_init.clone();
            if !tardis_init.is_completed() {
                tardis::tokio::task::spawn(async move {
                    if config.setup_tardis().await.is_ok() {
                        tardis_init.call_once(|| {});
                    } else {
                        warn!("[SG.Filter.Auth] tardis init failed");
                        panic!("tardis init failed")
                    };
                });
            }
        }
        while !tardis_init.is_completed() {
            // blocking wait tardis setup
        }
        Ok(plugin)
    }
    async fn call(&self, req: Request<SgBody>, inner: Inner) -> Result<Response<SgBody>, BoxError> {
        let req = match self.req(req).await {
            Ok(req) => req,
            Err(resp) => return Ok(resp),
        };
        let resp = inner.call(req).await;
        Ok(match self.resp(resp).await {
            Ok(resp) => resp,
            Err(resp) => resp,
        })
    }
    // fn create(_: Option<String>, value: JsonValue) -> Result<Self::MakeLayer, BoxError> {
    // }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests;
