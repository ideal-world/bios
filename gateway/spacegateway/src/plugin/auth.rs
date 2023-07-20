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
        filters::{BoxSgPluginFilter, SgPluginFilter, SgPluginFilterAccept, SgPluginFilterDef},
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
    fn accept(&self) -> SgPluginFilterAccept {
        SgPluginFilterAccept::default()
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
        let (mut auth_req, req_body) = ctx_to_auth_req(&mut ctx).await?;
        match auth_kernel_serv::auth(&mut auth_req, false).await {
            Ok(auth_resp) => {
                if auth_resp.allow {
                    ctx = success_auth_resp_to_ctx(auth_resp, req_body, ctx)?;
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
        let head_key_crypto = self.auth_config.head_key_crypto.clone();
        if ctx.get_req_headers().get(&head_key_crypto).is_none() {
            return Ok((true, ctx));
        }
        let crypto_value = ctx.get_req_headers().get(&head_key_crypto).expect("").clone();
        let ctx_resp_headers = ctx.get_resp_headers_mut();
        ctx_resp_headers.insert(
            HeaderName::try_from(head_key_crypto.clone()).map_err(|e| TardisError::internal_error(&format!("[Plugin.Auth] get header error: {e:?}"), ""))?,
            crypto_value,
        );
        let encrypt_resp = auth_crypto_serv::encrypt_body(&ctx_to_auth_encrypt_req(&mut ctx).await?).await?;
        ctx.set_resp_headers(hashmap_header_to_headermap(encrypt_resp.headers)?);
        ctx.set_resp_body(encrypt_resp.body.into_bytes())?;

        Ok((true, ctx))
    }
}
async fn ctx_to_auth_req(ctx: &mut SgRoutePluginContext) -> TardisResult<(AuthReq, Vec<u8>)> {
    let url = ctx.get_req_uri().clone();
    let scheme = url.scheme().map(|s| s.to_string()).unwrap_or("http".to_string());
    let headers = headermap_header_to_hashmap(ctx.get_req_headers().clone())?;
    let req_body = ctx.pop_req_body().await?;

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
            method: ctx.get_req_method().to_string(),
            host: url.host().unwrap_or("127.0.0.1").to_string(),
            port: url.port().map(|p| p.as_u16()).unwrap_or_else(|| if scheme == "https" { 443 } else { 80 }),
            headers,
            body: req_body.clone().map(|s| String::from_utf8_lossy(&s).to_string()),
        },
        req_body.unwrap_or_default(),
    ))
}

fn success_auth_resp_to_ctx(auth_resp: AuthResp, old_req_body: Vec<u8>, mut ctx: SgRoutePluginContext) -> TardisResult<SgRoutePluginContext> {
    let new_headers = hashmap_header_to_headermap(auth_resp.headers.clone())?;

    ctx.set_req_headers(new_headers);
    if let Some(new_body) = auth_resp.body {
        ctx.set_req_body(new_body.into_bytes())?;
    } else {
        ctx.set_req_body(old_req_body)?;
    }
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

    use spacegate_kernel::http::{Method, Uri, Version};
    use spacegate_kernel::hyper::{Body, StatusCode};
    use tardis::crypto::crypto_sm2_4::{TardisCryptoSm2, TardisCryptoSm2PrivateKey};
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
        let data = apis.data.unwrap();
        let pub_key = data["pub_key"].as_str().unwrap();
        let server_sm2 = TardisCryptoSm2 {};
        let server_public_key = server_sm2.new_public_key_from_public_key(pub_key).unwrap();

        let front_pri_key = TardisFuns::crypto.sm2.new_private_key().unwrap();
        let front_pub_key = TardisFuns::crypto.sm2.new_public_key(&front_pri_key).unwrap();

        let test_body_value = r##"test_body_value!@#$%^&*():"中文测试"##;
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
        let (is_ok, mut before_filter_ctx) = filter_auth.req_filter("", ctx, None).await.unwrap();
        assert!(is_ok);
        let req_body = before_filter_ctx.pop_req_body().await.unwrap();
        assert!(req_body.is_some());
        let req_body = req_body.unwrap();
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
        let (is_ok, mut before_filter_ctx) = filter_auth.req_filter("", ctx, None).await.unwrap();
        assert!(is_ok);
        let req_body = before_filter_ctx.pop_req_body().await.unwrap();
        assert!(req_body.is_some());
        let req_body = req_body.unwrap();
        let req_body = String::from_utf8(req_body).unwrap();
        assert_eq!(req_body, test_body_value.to_string());

        //======response============
        let mock_resp = r##"mock_resp:test_body_value!@#$%^&*():"中文测试"##;
        let mut header = HeaderMap::new();
        header.insert("Test_Header", "test_header".parse().unwrap());
        let ctx = before_filter_ctx.resp(StatusCode::OK, header, Body::from(mock_resp));

        let (is_ok, mut before_filter_ctx) = filter_auth.resp_filter("", ctx, None).await.unwrap();
        assert!(is_ok);
        let resp_body = before_filter_ctx.pop_resp_body().await.unwrap();
        assert!(resp_body.is_some());
        let resp_body = resp_body.unwrap();
        let resp_body = String::from_utf8(resp_body).unwrap();
        let resp_body = crypto_resp(
            &resp_body,
            &before_filter_ctx.get_resp_headers().get("Bios-Crypto").unwrap().to_str().unwrap(),
            &front_pri_key,
        );
        println!("req_body:{req_body} mock_resp:{mock_resp}");
        assert_eq!(resp_body, mock_resp.to_string());

        filter_auth.destroy().await.unwrap();

        let test_result = TardisFuns::web_client().get_to_str(&format!("http://127.0.0.1:{}/auth/auth/apis", filter_auth.port), None).await;
        assert!(test_result.is_err() || test_result.unwrap().code == 502);
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
        let base64_encrypt = TardisFuns::crypto.base64.encode(&sm4_encrypt);
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
        let gen_sign_data = TardisFuns::crypto.digest.sm3(&body).unwrap();
        assert_eq!(sign_data, gen_sign_data);
        TardisFuns::crypto.sm4.decrypt_cbc(&body, sm4_key, sm4_iv).unwrap()
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
