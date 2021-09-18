/*
 * Copyright 2021. gudaoxuri
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

use std::fmt::Display;

use actix_web::{http::StatusCode, HttpResponse};
use serde::Serialize;

use crate::basic::dto::{BIOSContext, BIOSResp};
use crate::basic::error::BIOSError;
use crate::basic::result::BIOSResult;

pub type BIOSResponse = BIOSResult<HttpResponse>;

impl<T: Serialize> BIOSResp<T> {
    pub fn ok(body: T, context: Option<&BIOSContext>) -> BIOSResponse {
        match context {
            Some(ctx) => Ok(HttpResponse::Ok().json(BIOSResp {
                code: "200".to_owned(),
                msg: "".to_owned(),
                body: Some(body),
                trace_id: Some(ctx.trace.id.to_string()),
                trace_app: Some(ctx.trace.app.to_string()),
                trace_inst: Some(ctx.trace.inst.to_string()),
            })),
            None => Ok(HttpResponse::Ok().json(BIOSResp {
                code: "200".to_owned(),
                msg: "".to_owned(),
                body: Some(body),
                trace_id: None,
                trace_app: None,
                trace_inst: None,
            })),
        }
    }

    pub fn resp(http_code: u16, resp: &BIOSResp<T>) -> BIOSResult<BIOSResponse> {
        match StatusCode::from_u16(http_code) {
            Ok(code) => Ok(Ok(HttpResponse::build(code).json(resp))),
            Err(e) => Err(BIOSError::Box(Box::new(e))),
        }
    }
}

impl BIOSResp<()> {
    pub fn error(bus_code: &str, message: &str, context: Option<&BIOSContext>) -> BIOSResponse {
        match context {
            Some(ctx) => Ok(HttpResponse::Ok().json(BIOSResp::<()> {
                code: bus_code.to_owned(),
                msg: message.to_owned(),
                body: None,
                trace_id: Some(ctx.trace.id.to_string()),
                trace_app: Some(ctx.trace.app.to_string()),
                trace_inst: Some(ctx.trace.inst.to_string()),
            })),
            None => Ok(HttpResponse::Ok().json(BIOSResp::<()> {
                code: bus_code.to_owned(),
                msg: message.to_owned(),
                body: None,
                trace_id: None,
                trace_app: None,
                trace_inst: None,
            })),
        }
    }

    pub fn err<E: Display>(error: E, context: Option<&BIOSContext>) -> BIOSResponse {
        match context {
            Some(ctx) => Ok(HttpResponse::Ok().json(crate::basic::result::output(error, ctx))),
            None => {
                let (code, msg) = crate::basic::result::parse(error);
                Ok(HttpResponse::Ok().json(BIOSResp::<()> {
                    code,
                    msg,
                    body: None,
                    trace_id: None,
                    trace_app: None,
                    trace_inst: None,
                }))
            }
        }
    }

    pub fn panic<E: Display>(error: E, context: Option<&BIOSContext>) -> BIOSResponse {
        let resp = match context {
            Some(ctx) => crate::basic::result::output(error, ctx),
            None => {
                let (code, msg) = crate::basic::result::parse(error);
                BIOSResp::<()> {
                    code,
                    msg,
                    body: None,
                    trace_id: None,
                    trace_app: None,
                    trace_inst: None,
                }
            }
        };
        Err(BIOSError::_Inner(crate::basic::json::obj_to_string(&resp).unwrap()))
    }

    pub fn to_log(self) -> String {
        format!(
            "{}|{}|{}[{}]{}",
            self.trace_app.unwrap_or_default(),
            self.trace_id.unwrap_or_default(),
            self.trace_inst.unwrap_or_default(),
            self.code,
            self.msg
        )
    }
}
