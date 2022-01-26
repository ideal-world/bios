/*
 * Copyright 2022. the original author or authors.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

// https://github.com/poem-web/poem

extern crate core;

use std::time::Duration;

use poem_openapi::{param::Path, payload::Json, Object, OpenApi, Tags};
use serde::{Deserialize, Serialize};
use tokio::time::sleep;

use bios::basic::config::{BIOSConfig, CacheConfig, DBConfig, FrameworkConfig, MQConfig, NoneConfig, WebServerConfig, WebServerModuleConfig};
use bios::basic::error::BIOSError;
use bios::basic::result::{BIOSResult, StatusCodeKind};
use bios::web::web_resp::BIOSResp;
use bios::BIOSFuns;

#[tokio::test]
async fn test_web_server() -> BIOSResult<()> {
    let url = "https://127.0.0.1:8080";

    // start_serv(url).await?;
    tokio::spawn(async { start_serv(url).await });
    sleep(Duration::from_millis(500)).await;

    test_basic(url).await?;
    test_validate(url).await?;

    Ok(())
}

async fn start_serv(url: &str) -> BIOSResult<()> {
    BIOSFuns::init_conf(BIOSConfig {
        ws: NoneConfig {},
        fw: FrameworkConfig {
            app: Default::default(),
            web_server: WebServerConfig {
                enabled: true,
                modules: vec![
                    WebServerModuleConfig {
                        code: "todo".to_string(),
                        title: "todo app".to_string(),
                        doc_urls: [("test env".to_string(), url.to_string()), ("prod env".to_string(), "http://127.0.0.1".to_string())].iter().cloned().collect(),
                        ..Default::default()
                    },
                    WebServerModuleConfig {
                        code: "other".to_string(),
                        title: "other app".to_string(),
                        ..Default::default()
                    },
                ],
                tls_key: Some(
                    r#"
-----BEGIN RSA PRIVATE KEY-----
MIIEpAIBAAKCAQEAqVYYdfxTT9qr1np22UoIWq4v1E4cHncp35xxu4HNyZsoJBHR
K1gTvwh8x4LMe24lROW/LGWDRAyhaI8qDxxlitm0DPxU8p4iQoDQi3Z+oVKqsSwJ
pd3MRlu+4QFrveExwxgdahXvnhYgFJw5qG/IDWbQM0+ism/yRiXaxFNMI/kXe8FG
+JKSyJzR/yXPqM9ootgIzWxjmV50c+4eyr97DvbwAQcmHi3Ao96p4XoxzKlYWwE9
TA+s0NvmCgYxOdjLEClP8YVKbvSpFMi4dHMZId86xYioeFbr7XPp+2njr9oyZjpd
Xa9Fy5UhwZZqCqh+nQk0m3XUC5pSu3ZrPLxNNQIDAQABAoIBAFKtZJgGsK6md4vq
kyiYSufrcBLaaEQ/rkQtYCJKyC0NAlZKFLRy9oEpJbNLm4cQSkYPXn3Qunx5Jj2k
2MYz+SgIDy7f7KHgr52Ew020dzNQ52JFvBgt6NTZaqL1TKOS1fcJSSNIvouTBerK
NCSXHzfb4P+MfEVe/w1c4ilE+kH9SzdEo2jK/sRbzHIY8TX0JbmQ4SCLLayr22YG
usIxtIYcWt3MMP/G2luRnYzzBCje5MXdpAhlHLi4TB6x4h5PmBKYc57uOVNngKLd
YyrQKcszW4Nx5v0a4HG3A5EtUXNCco1+5asXOg2lYphQYVh2R+1wgu5WiDjDVu+6
EYgjFSkCgYEA0NBk6FDoxE/4L/4iJ4zIhu9BptN8Je/uS5c6wRejNC/VqQyw7SHb
hRFNrXPvq5Y+2bI/DxtdzZLKAMXOMjDjj0XEgfOIn2aveOo3uE7zf1i+njxwQhPu
uSYA9AlBZiKGr2PCYSDPnViHOspVJjxRuAgyWM1Qf+CTC0D95aj0oz8CgYEAz5n4
Cb3/WfUHxMJLljJ7PlVmlQpF5Hk3AOR9+vtqTtdxRjuxW6DH2uAHBDdC3OgppUN4
CFj55kzc2HUuiHtmPtx8mK6G+otT7Lww+nLSFL4PvZ6CYxqcio5MPnoYd+pCxrXY
JFo2W7e4FkBOxb5PF5So5plg+d0z/QiA7aFP1osCgYEAtgi1rwC5qkm8prn4tFm6
hkcVCIXc+IWNS0Bu693bXKdGr7RsmIynff1zpf4ntYGpEMaeymClCY0ppDrMYlzU
RBYiFNdlBvDRj6s/H+FTzHRk2DT/99rAhY9nzVY0OQFoQIXK8jlURGrkmI/CYy66
XqBmo5t4zcHM7kaeEBOWEKkCgYAYnO6VaRtPNQfYwhhoFFAcUc+5t+AVeHGW/4AY
M5qlAlIBu64JaQSI5KqwS0T4H+ZgG6Gti68FKPO+DhaYQ9kZdtam23pRVhd7J8y+
xMI3h1kiaBqZWVxZ6QkNFzizbui/2mtn0/JB6YQ/zxwHwcpqx0tHG8Qtm5ZAV7PB
eLCYhQKBgQDALJxU/6hMTdytEU5CLOBSMby45YD/RrfQrl2gl/vA0etPrto4RkVq
UrkDO/9W4mZORClN3knxEFSTlYi8YOboxdlynpFfhcs82wFChs+Ydp1eEsVHAqtu
T+uzn0sroycBiBfVB949LExnzGDFUkhG0i2c2InarQYLTsIyHCIDEA==
-----END RSA PRIVATE KEY-----
"#
                    .to_string(),
                ),
                tls_cert: Some(
                    r#"
-----BEGIN CERTIFICATE-----
MIIEADCCAmigAwIBAgICAcgwDQYJKoZIhvcNAQELBQAwLDEqMCgGA1UEAwwhcG9u
eXRvd24gUlNBIGxldmVsIDIgaW50ZXJtZWRpYXRlMB4XDTE2MDgxMzE2MDcwNFoX
DTIyMDIwMzE2MDcwNFowGTEXMBUGA1UEAwwOdGVzdHNlcnZlci5jb20wggEiMA0G
CSqGSIb3DQEBAQUAA4IBDwAwggEKAoIBAQCpVhh1/FNP2qvWenbZSghari/UThwe
dynfnHG7gc3JmygkEdErWBO/CHzHgsx7biVE5b8sZYNEDKFojyoPHGWK2bQM/FTy
niJCgNCLdn6hUqqxLAml3cxGW77hAWu94THDGB1qFe+eFiAUnDmob8gNZtAzT6Ky
b/JGJdrEU0wj+Rd7wUb4kpLInNH/Jc+oz2ii2AjNbGOZXnRz7h7Kv3sO9vABByYe
LcCj3qnhejHMqVhbAT1MD6zQ2+YKBjE52MsQKU/xhUpu9KkUyLh0cxkh3zrFiKh4
Vuvtc+n7aeOv2jJmOl1dr0XLlSHBlmoKqH6dCTSbddQLmlK7dms8vE01AgMBAAGj
gb4wgbswDAYDVR0TAQH/BAIwADALBgNVHQ8EBAMCBsAwHQYDVR0OBBYEFMeUzGYV
bXwJNQVbY1+A8YXYZY8pMEIGA1UdIwQ7MDmAFJvEsUi7+D8vp8xcWvnEdVBGkpoW
oR6kHDAaMRgwFgYDVQQDDA9wb255dG93biBSU0EgQ0GCAXswOwYDVR0RBDQwMoIO
dGVzdHNlcnZlci5jb22CFXNlY29uZC50ZXN0c2VydmVyLmNvbYIJbG9jYWxob3N0
MA0GCSqGSIb3DQEBCwUAA4IBgQBsk5ivAaRAcNgjc7LEiWXFkMg703AqDDNx7kB1
RDgLalLvrjOfOp2jsDfST7N1tKLBSQ9bMw9X4Jve+j7XXRUthcwuoYTeeo+Cy0/T
1Q78ctoX74E2nB958zwmtRykGrgE/6JAJDwGcgpY9kBPycGxTlCN926uGxHsDwVs
98cL6ZXptMLTR6T2XP36dAJZuOICSqmCSbFR8knc/gjUO36rXTxhwci8iDbmEVaf
BHpgBXGU5+SQ+QM++v6bHGf4LNQC5NZ4e4xvGax8ioYu/BRsB/T3Lx+RlItz4zdU
XuxCNcm3nhQV2ZHquRdbSdoyIxV5kJXel4wCmOhWIq7A2OBKdu5fQzIAzzLi65EN
RPAKsKB4h7hGgvciZQ7dsMrlGw0DLdJ6UrFyiR5Io7dXYT/+JP91lP5xsl6Lhg9O
FgALt7GSYRm2cZdgi9pO9rRr83Br1VjQT1vHz6yoZMXSqc4A2zcN2a2ZVq//rHvc
FZygs8miAhWPzqnpmgTj1cPiU1M=
-----END CERTIFICATE-----
"#
                    .to_string(),
                ),
                ..Default::default()
            },
            web_client: Default::default(),
            cache: CacheConfig {
                enabled: false,
                ..Default::default()
            },
            db: DBConfig {
                enabled: false,
                ..Default::default()
            },
            mq: MQConfig {
                enabled: false,
                ..Default::default()
            },
            adv: Default::default(),
        },
    })
    .await?;
    BIOSFuns::web_server().add_module("todo", (TodosApi)).add_module("other", OtherApi).start().await
}

async fn test_basic(url: &str) -> BIOSResult<()> {
    // Normal
    let response = BIOSFuns::web_client().get::<BIOSResp<TodoResp>>(format!("{}/todo/todos/1", url).as_str(), None).await?.body.unwrap();
    assert_eq!(response.code, StatusCodeKind::Success.to_string());
    assert_eq!(response.data.unwrap().description, "测试");

    // Business Error
    let response = BIOSFuns::web_client().get::<BIOSResp<TodoResp>>(format!("{}/todo/todos/1/err", url).as_str(), None).await?.body.unwrap();
    assert_eq!(response.code, BIOSError::Conflict("异常".to_string()).code());
    assert_eq!(response.msg, BIOSError::Conflict("异常".to_string()).message());

    // Not Found
    let response = BIOSFuns::web_client().get::<BIOSResp<TodoResp>>(format!("{}/todo/todos/1/ss", url).as_str(), None).await?.body.unwrap();
    assert_eq!(response.code, StatusCodeKind::NotFound.to_string());
    assert_eq!(response.msg, "not found");

    Ok(())
}

async fn test_validate(url: &str) -> BIOSResult<()> {
    let response = BIOSFuns::web_client().get::<BIOSResp<TodoResp>>(format!("{}/todo/todos/ss", url).as_str(), None).await?.body.unwrap();
    assert_eq!(response.code, StatusCodeKind::BadRequest.to_string());
    assert_eq!(
        response.msg,
        r#"failed to parse parameter `id`: failed to parse "integer(int64)": invalid digit found in string"#
    );

    let response = BIOSFuns::web_client()
        .post::<ValidateReq, BIOSResp<String>>(
            format!("{}/other/validate", url).as_str(),
            &ValidateReq {
                len: "".to_string(),
                eq: "".to_string(),
                range: 0,
                mail: "".to_string(),
                contain: "".to_string(),
                phone: "".to_string(),
                item_len: vec![],
                item_unique: vec![],
            },
            None,
        )
        .await?
        .body
        .unwrap();
    assert_eq!(response.code, StatusCodeKind::BadRequest.to_string());
    assert_eq!(
        response.msg,
        r#"parse JSON error: failed to parse "ValidateReq": field `len` verification failed. minLength(1)"#
    );

    let response = BIOSFuns::web_client()
        .post::<ValidateReq, BIOSResp<String>>(
            format!("{}/other/validate", url).as_str(),
            &ValidateReq {
                len: "1".to_string(),
                eq: "".to_string(),
                range: 0,
                mail: "".to_string(),
                contain: "".to_string(),
                phone: "".to_string(),
                item_len: vec![],
                item_unique: vec![],
            },
            None,
        )
        .await?
        .body
        .unwrap();
    assert_eq!(response.code, StatusCodeKind::BadRequest.to_string());
    assert_eq!(
        response.msg,
        r#"parse JSON error: failed to parse "ValidateReq": field `eq` verification failed. minLength(5)"#
    );

    let response = BIOSFuns::web_client()
        .post::<ValidateReq, BIOSResp<String>>(
            format!("{}/other/validate", url).as_str(),
            &ValidateReq {
                len: "1".to_string(),
                eq: "11111".to_string(),
                range: 0,
                mail: "".to_string(),
                contain: "".to_string(),
                phone: "".to_string(),
                item_len: vec![],
                item_unique: vec![],
            },
            None,
        )
        .await?
        .body
        .unwrap();
    assert_eq!(response.code, StatusCodeKind::BadRequest.to_string());
    assert_eq!(
        response.msg,
        r#"parse JSON error: failed to parse "ValidateReq": field `range` verification failed. minimum(1, exclusive: false)"#
    );

    let response = BIOSFuns::web_client()
        .post::<ValidateReq, BIOSResp<String>>(
            format!("{}/other/validate", url).as_str(),
            &ValidateReq {
                len: "1".to_string(),
                eq: "11111".to_string(),
                range: 444,
                mail: "ss.ss".to_string(),
                contain: "".to_string(),
                phone: "".to_string(),
                item_len: vec![],
                item_unique: vec![],
            },
            None,
        )
        .await?
        .body
        .unwrap();
    assert_eq!(response.code, StatusCodeKind::BadRequest.to_string());
    assert_eq!(
        response.msg,
        r#"parse JSON error: failed to parse "ValidateReq": field `mail` verification failed. Invalid mail format"#
    );

    let response = BIOSFuns::web_client()
        .post::<ValidateReq, BIOSResp<String>>(
            format!("{}/other/validate", url).as_str(),
            &ValidateReq {
                len: "1".to_string(),
                eq: "11111".to_string(),
                range: 444,
                mail: "ss@ss.ss".to_string(),
                contain: "".to_string(),
                phone: "".to_string(),
                item_len: vec![],
                item_unique: vec![],
            },
            None,
        )
        .await?
        .body
        .unwrap();
    assert_eq!(response.code, StatusCodeKind::BadRequest.to_string());
    assert_eq!(
        response.msg,
        r#"parse JSON error: failed to parse "ValidateReq": field `contain` verification failed. pattern(".*gmail.*")"#
    );

    let response = BIOSFuns::web_client()
        .post::<ValidateReq, BIOSResp<String>>(
            format!("{}/other/validate", url).as_str(),
            &ValidateReq {
                len: "1".to_string(),
                eq: "11111".to_string(),
                range: 444,
                mail: "ss@ss.ss".to_string(),
                contain: "gmail".to_string(),
                phone: "".to_string(),
                item_len: vec![],
                item_unique: vec![],
            },
            None,
        )
        .await?
        .body
        .unwrap();
    assert_eq!(response.code, StatusCodeKind::BadRequest.to_string());
    assert_eq!(
        response.msg,
        r#"parse JSON error: failed to parse "ValidateReq": field `phone` verification failed. Invalid phone number format"#
    );

    let response = BIOSFuns::web_client()
        .post::<ValidateReq, BIOSResp<String>>(
            format!("{}/other/validate", url).as_str(),
            &ValidateReq {
                len: "1".to_string(),
                eq: "11111".to_string(),
                range: 444,
                mail: "ss@ss.ss".to_string(),
                contain: "gmail".to_string(),
                phone: "18654110201".to_string(),
                item_len: vec![],
                item_unique: vec![],
            },
            None,
        )
        .await?
        .body
        .unwrap();
    assert_eq!(response.code, StatusCodeKind::BadRequest.to_string());
    assert_eq!(
        response.msg,
        r#"parse JSON error: failed to parse "ValidateReq": field `item_len` verification failed. minItems(1)"#
    );

    let response = BIOSFuns::web_client()
        .post::<ValidateReq, BIOSResp<String>>(
            format!("{}/other/validate", url).as_str(),
            &ValidateReq {
                len: "1".to_string(),
                eq: "11111".to_string(),
                range: 444,
                mail: "ss@ss.ss".to_string(),
                contain: "gmail".to_string(),
                phone: "18654110201".to_string(),
                item_len: vec!["ddd".to_string()],
                item_unique: vec!["ddd".to_string(), "ddd".to_string()],
            },
            None,
        )
        .await?
        .body
        .unwrap();
    assert_eq!(response.code, StatusCodeKind::BadRequest.to_string());
    assert_eq!(
        response.msg,
        r#"parse JSON error: failed to parse "ValidateReq": field `item_unique` verification failed. uniqueItems()"#
    );

    let response = BIOSFuns::web_client()
        .post::<ValidateReq, BIOSResp<String>>(
            format!("{}/other/validate", url).as_str(),
            &ValidateReq {
                len: "1".to_string(),
                eq: "11111".to_string(),
                range: 444,
                mail: "ss@ss.ss".to_string(),
                contain: "gmail".to_string(),
                phone: "18654110201".to_string(),
                item_len: vec!["ddd".to_string()],
                item_unique: vec!["ddd1".to_string(), "ddd2".to_string()],
            },
            None,
        )
        .await?
        .body
        .unwrap();
    assert_eq!(response.code, StatusCodeKind::Success.to_string());

    Ok(())
}

#[derive(Tags)]
enum FunTags {
    #[oai(rename = "Todo1测试")]
    Todo1,
    #[oai(rename = "Todo2测试")]
    Todo2,
}

#[derive(Object, Serialize, Deserialize, Debug)]
struct TodoResp {
    id: i64,
    description: String,
    done: bool,
}

#[derive(Object, Serialize, Deserialize, Debug)]
struct TodoAddReq {
    description: String,
    done: bool,
}

#[derive(Object, Serialize, Deserialize, Debug)]
struct TodoModifyReq {
    description: Option<String>,
    done: Option<bool>,
}

#[derive(Object, Serialize, Deserialize, Debug)]
struct ValidateReq {
    #[oai(validator(min_length = "1", max_length = "10"))]
    len: String,
    #[oai(validator(min_length = "5", max_length = "5"))]
    eq: String,
    #[oai(validator(minimum(value = "1", exclusive = "false"), maximum(value = "500", exclusive)))]
    range: u32,
    #[oai(validator(custom = "bios::web::web_validation::Mail"))]
    mail: String,
    #[oai(validator(pattern = r".*gmail.*"))]
    contain: String,
    #[oai(validator(custom = "bios::web::web_validation::Phone"))]
    phone: String,
    #[oai(validator(min_items = "1", max_items = "3"))]
    item_len: Vec<String>,
    #[oai(validator(unique_items))]
    item_unique: Vec<String>,
}

struct TodosApi;

#[OpenApi(tag = "FunTags::Todo1")]
impl TodosApi {
    #[oai(path = "/todos", method = "post")]
    async fn create(&self, todo_add_req: Json<TodoAddReq>) -> BIOSResp<String> {
        BIOSResp::ok("0".into())
    }

    #[oai(path = "/todos/:id", method = "get")]
    async fn get(&self, id: Path<i64>) -> BIOSResp<TodoResp> {
        BIOSResp::ok(TodoResp {
            id: id.0,
            description: "测试".to_string(),
            done: false,
        })
    }

    #[oai(path = "/todos/:id/err", method = "get")]
    async fn get_by_error(&self, id: Path<i64>) -> BIOSResp<TodoResp> {
        BIOSResp::err(BIOSError::Conflict("异常".to_string()))
    }
}

struct OtherApi;

#[OpenApi]
impl OtherApi {
    #[oai(path = "/validate", method = "post")]
    async fn test(&self, _req: Json<ValidateReq>) -> BIOSResp<String> {
        BIOSResp::ok("".into())
    }
}
