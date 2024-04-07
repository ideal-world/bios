use std::collections::HashMap;
use std::env;

use http_backend::{TestAddReq, TestApi, TestDetailResp};
use serde::{Deserialize, Serialize};
use tardis::basic::dto::TardisContext;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::config::config_dto::{FrameworkConfig, TardisConfig, WebServerCommonConfig, WebServerConfig, WebServerModuleConfig};
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::payload::Json;
use tardis::web::web_resp::{TardisApiResult, TardisResp};
use tardis::{log, testcontainers, tokio, TardisFuns};
mod http_backend;
mod init_apisix;

#[tokio::main]
async fn main() -> TardisResult<()> {
    env::set_var("RUST_LOG", "info,tardis=trace");
    // Prepare
    log::info!("Init http server");
    start_serv().await?;
    log::info!("Init gateways server");
    let docker = testcontainers::clients::Cli::default();
    let (gateway_url, _life_hold) = init_apisix::init(&docker).await?;

    log::info!("\r\n=============\r\nStart test\r\n=============");
    // Test
    let resp: TardisResp<TestDetailResp> = TardisFuns::web_client()
        .post(
            &format!("{gateway_url}/test/echo/1"),
            &TestAddReq {
                code: TrimString("c001".to_string()),
                description: "测试001".to_string(),
                done: false,
            },
            None,
        )
        .await?
        .body
        .unwrap();
    let resp = resp.data.unwrap();
    assert_eq!(&resp.code, "c001");
    assert_eq!(&resp.description, "测试001");
    assert!(!resp.done);

    let header: Vec<(String, String)> = vec![("Bios-Crypto".to_string(), "".to_string())];
    //有加密头的请求会被替换"成空 ,所以不能传json串需要base64
    let body = TardisFuns::crypto.base64.encode(
        TardisFuns::json
            .obj_to_string(&TestAddReq {
                code: TrimString("c001".to_string()),
                description: "测试002".to_string(),
                done: false,
            })
            .unwrap(),
    );
    let resp: TardisResp<TestDetailResp> = TardisFuns::web_client().post(&format!("{gateway_url}/test/echo/2"), &body, header).await?.body.unwrap();
    let resp = resp.data.unwrap();
    assert_eq!(&resp.code, "c001");
    assert_eq!(&resp.description, "测试002");
    assert!(!resp.done);

    let header: Vec<(String, String)> = vec![("Bios-Crypto".to_string(), "".to_string())];
    let resp: TardisResp<String> = TardisFuns::web_client().get(&format!("{gateway_url}/test/echo/get/3"), header).await?.body.unwrap();
    let resp = resp.data.unwrap();
    assert_eq!(resp, "3".to_string());

    let body = TardisFuns::crypto.base64.encode(
        TardisFuns::json
            .obj_to_string(&TestAddReq {
                code: TrimString("c001".to_string()),
                description: "测试003".to_string(),
                done: false,
            })
            .unwrap(),
    );
    let header: Vec<(String, String)> = vec![("Bios-Crypto".to_string(), "".to_string())];
    let resp: TardisResp<TestDetailResp> = TardisFuns::web_client().post(&format!("{gateway_url}/apis"), &body, header).await?.body.unwrap();
    let resp = resp.data.unwrap();
    assert_eq!(&resp.code, "c001");
    assert_eq!(&resp.description, "测试003");
    assert!(!resp.done);

    log::info!("\r\n=============\r\nTest Success\r\n=============");

    Ok(())
}
#[derive(Clone, Default)]
pub struct AuthApi;

/// Auth API
#[poem_openapi::OpenApi(prefix_path = "/auth")]
impl AuthApi {
    /// Auth
    #[oai(path = "/", method = "put")]
    async fn auth(&self, req: Json<AuthReq>) -> TardisApiResult<AuthResp> {
        let mut req = req.0;
        if req.path == "/test/echo/1" {
            assert!(req.body.is_none());
        }
        let mut headers = req.headers;
        if req.path == "/test/echo/2" {
            assert!(req.body.is_some());
            let req_body = TardisFuns::crypto.base64.decode_to_string(req.body.clone().unwrap())?;
            assert!(req_body.contains("测试002"));
            req.body = Some(req_body);
            headers.insert("Bios-Crypto".to_string(), "".to_string());
        }
        if req.path == "/test/echo/get/3" {
            assert_eq!(req.body, None);
            headers.insert("Bios-Crypto".to_string(), "".to_string());
        }
        headers.insert(
            "Tardis-Context".to_string(),
            TardisFuns::crypto.base64.encode(TardisFuns::json.obj_to_string(&TardisContext::default()).unwrap()),
        );
        TardisResp::ok(AuthResp {
            allow: true,
            status_code: 200,
            reason: None,
            headers,
            body: req.body,
        })
    }

    /// Auth
    #[oai(path = "/apis", method = "put")]
    async fn apis(&self, req: Json<AuthReq>) -> TardisApiResult<MixAuthResp> {
        let mut req = req.0;
        let mut headers = req.headers;
        if req.path.contains("/apis") {
            assert!(req.body.is_some());
            let req_body = TardisFuns::crypto.base64.decode_to_string(req.body.clone().unwrap())?;
            assert!(req_body.contains("测试003"));
            req.body = Some(req_body);
            headers.insert("Bios-Crypto".to_string(), "".to_string());
        }
        headers.insert(
            "Tardis-Context".to_string(),
            TardisFuns::crypto.base64.encode(TardisFuns::json.obj_to_string(&TardisContext::default()).unwrap()),
        );
        TardisResp::ok(MixAuthResp {
            url: "/test/echo/4".to_string(),
            method: "POST".to_string(),
            allow: true,
            status_code: 200,
            reason: None,
            headers,
            body: req.body,
        })
    }
}

async fn start_serv() -> TardisResult<()> {
    TardisFuns::init_conf(TardisConfig {
        cs: Default::default(),
        fw: FrameworkConfig {
            web_server: Some(
                WebServerConfig::builder()
                    .common(WebServerCommonConfig::builder().port(8080).build())
                    .default(WebServerModuleConfig::default())
                    .modules([
                        ("auth".to_string(), WebServerModuleConfig::default()),
                        ("test".to_string(), WebServerModuleConfig::default()),
                    ])
                    .build(),
            ),
            ..Default::default()
        },
    })
    .await?;
    TardisFuns::web_server().add_module("auth", AuthApi).await.add_module("test", TestApi).await.start().await
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct AuthReq {
    pub scheme: String,
    pub path: String,
    pub query: HashMap<String, String>,
    pub method: String,
    pub host: String,
    pub port: u16,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct AuthResp {
    pub allow: bool,
    pub status_code: u16,
    pub reason: Option<String>,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct MixAuthResp {
    pub url: String,
    pub method: String,
    pub allow: bool,
    pub status_code: u16,
    pub reason: Option<String>,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
}
