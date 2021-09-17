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

use std::task::{Context, Poll};

use actix_service::{Service, Transform};
use actix_web::dev::{AnyBody, ServiceRequest, ServiceResponse};
use actix_web::error::{Error, Result};
use actix_web::http::{HeaderValue, StatusCode};
use actix_web::web::Bytes;
use actix_web::{http, HttpResponse, HttpResponseBuilder, ResponseError};
use futures_util::future::{ok, FutureExt, LocalBoxFuture, Ready};
use log::{trace, warn};

use crate::basic::dto::BIOSResp;
use crate::basic::error::BIOSError;

pub struct WebErrorHandler;

impl<S> Transform<S, ServiceRequest> for WebErrorHandler
where
    S: Service<ServiceRequest, Response = ServiceResponse, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse;
    type Error = Error;
    type Transform = WebErrorMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(WebErrorMiddleware { service })
    }
}

pub struct WebErrorMiddleware<S> {
    service: S,
}

impl<S> Service<ServiceRequest> for WebErrorMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse, Error = Error>,
    S::Future: 'static,
{
    type Response = ServiceResponse;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let fut = self.service.call(req);
        async move {
            let mut res = fut.await?;
            let http_code = res.status().as_u16();
            if http_code >= 400 {
                let msg = match res.response().error() {
                    Some(e) => e.to_string(),
                    None => match http_code {
                        404 => format!("method:{}, url:{}", res.request().method().to_string(), res.request().uri().to_string()),
                        _ => "unknown error".to_string(),
                    },
                };
                let bios_resp = if msg.contains("code") && msg.contains("msg") && msg.contains("body") {
                    let try_convert_resp = crate::basic::json::str_to_obj::<BIOSResp<()>>(&msg);
                    if try_convert_resp.is_ok() {
                        try_convert_resp.unwrap()
                    } else {
                        convert_resp(http_code, msg)
                    }
                } else {
                    convert_resp(http_code, msg)
                };

                res.headers_mut().insert(http::header::CONTENT_TYPE, HeaderValue::from_static("application/json"));
                if http_code >= 500 {
                    warn!(
                        "[BIOS.Framework.WebServer] process error,request method:{}, url:{}, response code:{}, message:{}",
                        res.request().method().to_string(),
                        res.request().uri().to_string(),
                        bios_resp.code,
                        bios_resp.msg
                    );
                } else {
                    trace!(
                        "[BIOS.Framework.WebServer] process error,request method:{}, url:{}, response code:{}, message:{}",
                        res.request().method().to_string(),
                        res.request().uri().to_string(),
                        bios_resp.code,
                        bios_resp.msg
                    );
                    // 4xx error: Http status modified to 200, by bus_code to provide a unified error code
                    // 5xx error: Considering that all kinds of degradation components only provide processing of http status, so the 5xx error isn’t modified
                    *res.response_mut().status_mut() = StatusCode::from_u16(200).unwrap();
                }
                let res = res.map_body(|_, _| AnyBody::Bytes(Bytes::from(serde_json::json!(bios_resp).to_string())));
                Ok(res)
            } else {
                Ok(res)
            }
        }
        .boxed_local()
    }
}

fn convert_resp(http_status_code: u16, msg: String) -> BIOSResp<()> {
    let error = match http_status_code {
        500 => BIOSError::InternalError(msg),
        501 => BIOSError::NotImplemented(msg),
        503 => BIOSError::IOError(msg),
        400 => BIOSError::BadRequest(msg),
        401 => BIOSError::Unauthorized(msg),
        404 => BIOSError::NotFound(msg),
        406 => BIOSError::FormatError(msg),
        408 => BIOSError::Timeout(msg),
        409 => BIOSError::Conflict(msg),
        c if c > 500 => BIOSError::InternalError(msg),
        _ => BIOSError::BadRequest(msg),
    };
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

impl ResponseError for BIOSError {
    fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponseBuilder::new(self.status_code()).body(self.to_string())
    }
}
