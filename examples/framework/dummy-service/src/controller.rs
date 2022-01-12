/*
 * Copyright 2022. gudaoxuri
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

use std::str::FromStr;
use std::sync::Mutex;

use actix_web::http::Uri;
use actix_web::{post, put, web, HttpRequest, HttpResponse};
use rand::Rng;
use serde::{Deserialize, Serialize};

use bios::basic::dto::BIOSResp;
use bios::web::resp_handler::BIOSResponse;
use bios::BIOSFuns;

pub struct AppStateContainer {
    pub err_rate: Mutex<u8>,
}

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct NormalQuery {
    size: u32,
    forward: String,
}

impl Default for NormalQuery {
    fn default() -> Self {
        NormalQuery {
            size: 0,
            forward: String::from(""),
        }
    }
}

#[post("/normal/{id}")]
pub async fn normal(query: web::Query<NormalQuery>, body: web::Bytes, data: web::Data<AppStateContainer>, req: HttpRequest) -> BIOSResponse {
    /*   req.headers().into_iter().for_each(|(k, v)| {
        println!("Header:{}-{:?}", k, v)
    });*/

    if !query.forward.is_empty() {
        let forwarded_req = BIOSFuns::web_client().raw().request_from(Uri::from_str(&query.forward).unwrap(), req.head()).no_decompress();
        let forwarded_req = if let Some(addr) = req.head().peer_addr {
            forwarded_req.insert_header(("x-forwarded-for", format!("{}", addr.ip())))
        } else {
            forwarded_req
        };

        let mut forwarded_resp = forwarded_req.send_body(body).await.unwrap();
        let mut resp = HttpResponse::build(forwarded_resp.status());
        for (header_name, header_value) in forwarded_resp.headers().iter().filter(|(h, _)| *h != "connection") {
            resp.insert_header((header_name.clone(), header_value.clone()));
        }
        Ok(resp.body(forwarded_resp.body().await.unwrap()))
    } else {
        let err_rate_conf = data.err_rate.lock().unwrap();

        let code = if *err_rate_conf > 0 && *err_rate_conf >= rand::thread_rng().gen_range(0..100) {
            // 随机值小于等于配置错误率时返回错误
            500u16
        } else {
            200u16
        };

        let body = if query.size <= 0 {
            String::from("")
        } else {
            String::from_utf8(vec![b'X'; query.size as usize]).unwrap()
        };

        let resp = BIOSResp::<String> {
            code: code.to_string(),
            msg: String::from(""),
            body: Some(body),
            trace_id: None,
            trace_app: None,
            trace_inst: None,
            ctx: None,
        };
        BIOSResp::resp(code, &resp)?
    }
}

#[post("/fallback/{id}")]
pub async fn fallback() -> BIOSResponse {
    BIOSResp::ok("This is a fallback result", None)
}

#[put("/conf/err_rate/{err_rate}")]
pub async fn conf_err_rate(path: web::Path<u8>, data: web::Data<AppStateContainer>) -> BIOSResponse {
    let mut err_rate_conf = data.err_rate.lock().unwrap();
    *err_rate_conf = path.into_inner();
    BIOSResp::ok("", None)
}
