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

use async_trait::async_trait;
use poem::error::{CorsError, MethodNotAllowedError, NotFoundError, ParsePathError};
use poem::http::StatusCode;
use poem::{Endpoint, IntoResponse, Middleware, Request, Response};
use poem_openapi::error::{AuthorizationError, ContentTypeError, ParseJsonError, ParseMultipartError, ParseParamError};
use poem_openapi::payload::Payload;
use poem_openapi::registry::{MetaMediaType, MetaResponse, MetaResponses, MetaSchemaRef, Registry};
use poem_openapi::{
    types::{ParseFromJSON, ToJSON},
    ApiResponse,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::{trace, warn};

use crate::basic::error::BIOSError;
use crate::basic::result::{parse, StatusCodeKind};
use crate::BIOSFuns;

#[derive(Deserialize, Serialize, Debug)]
#[serde(default)]
pub struct BIOSResp<T>
where
    T: ParseFromJSON + ToJSON + Serialize + Send + Sync,
{
    pub code: String,
    pub msg: String,
    pub data: Option<T>,
}

impl<T> Default for BIOSResp<T>
where
    T: ParseFromJSON + ToJSON + Serialize + Send + Sync,
{
    fn default() -> Self {
        BIOSResp {
            code: "".to_string(),
            msg: "".to_string(),
            data: None,
        }
    }
}

impl<T> BIOSResp<T>
where
    T: ParseFromJSON + ToJSON + Serialize + Send + Sync,
{
    pub fn ok(data: T) -> Self {
        Self {
            code: StatusCodeKind::Success.to_string(),
            msg: "".to_string(),
            data: Some(data),
        }
    }

    pub fn err(error: BIOSError) -> Self {
        let (code, msg) = parse(error);
        Self { code, msg, data: None }
    }
}

impl<T> IntoResponse for BIOSResp<T>
where
    T: ParseFromJSON + ToJSON + Serialize + Send + Sync,
{
    fn into_response(self) -> Response {
        Response::builder().status(StatusCode::OK).header("Content-Type", "application/json; charset=utf8").body(BIOSFuns::json.obj_to_string(&self).unwrap())
    }
}

impl<T> Payload for BIOSResp<T>
where
    T: ParseFromJSON + ToJSON + Serialize + Send + Sync,
{
    const CONTENT_TYPE: &'static str = "application/json";
    fn schema_ref() -> MetaSchemaRef {
        T::schema_ref()
    }

    #[allow(unused_variables)]
    fn register(registry: &mut Registry) {
        T::register(registry);
    }
}

impl<T> ApiResponse for BIOSResp<T>
where
    T: ParseFromJSON + ToJSON + Serialize + Send + Sync,
{
    fn meta() -> MetaResponses {
        MetaResponses {
            responses: vec![MetaResponse {
                description: "",
                status: Some(200),
                content: vec![MetaMediaType {
                    content_type: Self::CONTENT_TYPE,
                    schema: Self::schema_ref(),
                }],
                headers: vec![],
            }],
        }
    }

    fn register(registry: &mut Registry) {
        T::register(registry);
    }
}

pub struct UniformError;

impl<E: Endpoint> Middleware<E> for UniformError {
    type Output = UniformErrorImpl<E>;

    fn transform(&self, ep: E) -> Self::Output {
        UniformErrorImpl(ep)
    }
}

pub struct UniformErrorImpl<E>(E);

#[async_trait]
impl<E: Endpoint> Endpoint for UniformErrorImpl<E> {
    type Output = Response;

    async fn call(&self, req: Request) -> poem::Result<Self::Output> {
        let method = req.method().to_string();
        let url = req.uri().to_string();
        let resp = self.0.call(req).await;
        match resp {
            Ok(resp) => {
                let mut resp = resp.into_response();
                if resp.status() != StatusCode::OK {
                    let msg = resp.take_body().into_string().await.expect("Http request exception type conversion error");
                    let code = if resp.status().as_u16() >= 500 {
                        warn!(
                            "[BIOS.Framework.WebServer] process error,request method:{}, url:{}, response code:{}, message:{}",
                            method,
                            url,
                            resp.status().as_u16(),
                            msg
                        );
                        resp.status()
                    } else {
                        trace!(
                            "[BIOS.Framework.WebServer] process error,request method:{}, url:{}, response code:{}, message:{}",
                            method,
                            url,
                            resp.status().as_u16(),
                            msg
                        );
                        // Request fallback friendly
                        StatusCode::OK
                    };
                    resp.set_status(code);
                    resp.headers_mut().insert("Content-Type", "application/json; charset=utf8".parse().unwrap());
                    resp.set_body(
                        json!({
                            "code": mapping_code(code).to_string(),
                            "msg": msg,
                        })
                        .to_string(),
                    );
                }
                Ok(resp)
            }
            Err(err) => Ok(error_handler(err)),
        }
    }
}

fn mapping_code(http_code: StatusCode) -> StatusCodeKind {
    match http_code {
        StatusCode::OK => StatusCodeKind::Success,
        StatusCode::BAD_REQUEST => StatusCodeKind::BadRequest,
        StatusCode::UNAUTHORIZED => StatusCodeKind::Unauthorized,
        StatusCode::FORBIDDEN => StatusCodeKind::NotFound,
        StatusCode::NOT_FOUND => StatusCodeKind::NotFound,
        StatusCode::METHOD_NOT_ALLOWED => StatusCodeKind::NotFound,
        StatusCode::INTERNAL_SERVER_ERROR => StatusCodeKind::InternalError,
        StatusCode::SERVICE_UNAVAILABLE => StatusCodeKind::InternalError,
        _ => StatusCodeKind::UnKnown,
    }
}

fn error_handler(err: poem::Error) -> Response {
    let (code, msg) =
        if err.is::<ParseParamError>() || err.is::<ParseJsonError>() || err.is::<ParseMultipartError>() || err.is::<ParsePathError>() || err.is::<MethodNotAllowedError>() {
            (StatusCodeKind::BadRequest.to_string(), err.to_string())
        } else if err.is::<NotFoundError>() || err.is::<ContentTypeError>() {
            (StatusCodeKind::NotFound.to_string(), err.to_string())
        } else if err.is::<AuthorizationError>() || err.is::<CorsError>() {
            (StatusCodeKind::Unauthorized.to_string(), err.to_string())
        } else {
            warn!("[BIOS.Framework.WebServer] process error: {:?}", err);
            (StatusCodeKind::UnKnown.to_string(), err.to_string())
        };
    Response::builder().status(StatusCode::OK).header("Content-Type", "application/json; charset=utf8").body(
        json!({
            "code": code,
            "msg": msg,
        })
        .to_string(),
    )
}
