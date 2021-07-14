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

use crate::basic::error::BIOSError;
use crate::web::resp_handler::BIOSRespHelper;

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
            let bus_code = http_code.to_string();
            if http_code >= 400 {
                res.headers_mut().insert(
                    http::header::CONTENT_TYPE,
                    HeaderValue::from_static("application/json"),
                );
                let msg = match res.response().error() {
                    Some(e) => e.to_string(),
                    None => match http_code {
                        404 => format!(
                            "method:{}, url:{}",
                            res.request().method().to_string(),
                            res.request().uri().to_string()
                        ),
                        _ => "unknown error".to_string(),
                    },
                };
                if http_code>= 500 {
                    warn!("[BIOS.Framework.WebServer] process error,request method:{}, url:{}, response code:{}, message:{}",
                          res.request().method().to_string(),
                          res.request().uri().to_string(),
                          bus_code,
                          msg
                    )
                }else{
                    trace!("[BIOS.Framework.WebServer] process error,request method:{}, url:{}, response code:{}, message:{}",
                          res.request().method().to_string(),
                          res.request().uri().to_string(),
                          bus_code,
                          msg
                    )
                }
                let res = res.map_body(|_, _| {
                    AnyBody::Bytes(Bytes::from(serde_json::json!(BIOSRespHelper::<()> {
                            code: bus_code,
                            msg: msg,
                            body: None,
                        }).to_string())
                    )
                });
                Ok(res)
            } else {
                Ok(res)
            }
        }
        .boxed_local()
    }
}

impl ResponseError for BIOSError {
    fn status_code(&self) -> StatusCode {
        match *self {
            BIOSError::Custom(_, _) => StatusCode::BAD_REQUEST,
            BIOSError::BadRequest(_) => StatusCode::BAD_REQUEST,
            BIOSError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            BIOSError::NotFound(_) => StatusCode::NOT_FOUND,
            BIOSError::Conflict(_) => StatusCode::CONFLICT,
            BIOSError::ValidationError(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponseBuilder::new(self.status_code()).json(BIOSRespHelper::<()> {
            code: self.status_code().to_string(),
            msg: self.to_string(),
            body: None,
        })
    }
}
