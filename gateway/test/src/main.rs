use std::collections::HashMap;
use std::env;

use http_backend::{TestAddReq, TestApi, TestDetailResp};
use serde::{Deserialize, Serialize};
use tardis::basic::dto::TardisContext;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::config::config_dto::{FrameworkConfig, TardisConfig, WebServerConfig, WebServerModuleConfig};
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::payload::Json;
use tardis::web::web_resp::{TardisApiResult, TardisResp};
use tardis::{log, testcontainers, tokio, TardisFuns};
mod http_backend;
mod init_apisix;

#[tokio::main]
async fn main() -> TardisResult<()> {
    env::set_var("RUST_LOG", "info");
    // Prepare
    log::info!("Init http server");
    tokio::spawn(async move { start_serv().await });
    log::info!("Init gateway server");
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
    let resp: TardisResp<TestDetailResp> = TardisFuns::web_client()
        .post(
            &format!("{gateway_url}/test/echo/2"),
            &TestAddReq {
                code: TrimString("c001".to_string()),
                description: "测试002".to_string(),
                done: false,
            },
            Some(header),
        )
        .await?
        .body
        .unwrap();
    let resp = resp.data.unwrap();
    assert_eq!(&resp.code, "c001");
    assert_eq!(&resp.description, "测试002");
    assert!(!resp.done);

    log::info!("\r\n=============\r\nTest Success\r\n=============");

    Ok(())
}

pub struct AuthApi;

/// Auth API
#[poem_openapi::OpenApi(prefix_path = "/auth")]
impl AuthApi {
    /// Auth
    #[oai(path = "/", method = "put")]
    async fn auth(&self, req: Json<AuthReq>) -> TardisApiResult<AuthResp> {
        let req = req.0;
        if req.path == "/test/echo/1" {
            assert!(req.body.is_none());
        }
        let mut headers = req.headers;
        if req.path == "/test/echo/2" {
            assert!(req.body.is_some());
            assert!(req.body.clone().unwrap().contains("测试002"));
            headers.insert("Bios-Crypto".to_string(), "".to_string());
        }
        headers.insert(
            "Tardis-Context".to_string(),
            TardisFuns::crypto.base64.encode(&TardisFuns::json.obj_to_string(&TardisContext::default()).unwrap()),
        );
        TardisResp::ok(AuthResp {
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
            web_server: WebServerConfig {
                enabled: true,
                port: 8080,
                modules: HashMap::from([
                    ("auth".to_string(), WebServerModuleConfig { ..Default::default() }),
                    ("test".to_string(), WebServerModuleConfig { ..Default::default() }),
                ]),
                ..Default::default()
            },
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
