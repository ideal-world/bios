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

use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use serde::{Deserialize, Serialize};

use crate::basic::error::{BIOSError, BIOSResult};

pub type BIOSResp = BIOSResult<HttpResponse>;

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct BIOSRespHelper<T>
where
    T: Serialize,
{
    pub code: String,
    pub msg: String,
    pub body: Option<T>,
}

impl<T> Default for BIOSRespHelper<T>
where
    T: Serialize,
{
    fn default() -> Self {
        BIOSRespHelper {
            code: "".to_owned(),
            msg: "".to_owned(),
            body: None,
        }
    }
}

impl<T: Serialize> BIOSRespHelper<T> {
    pub fn ok(body: T) -> BIOSResp {
        Ok(HttpResponse::Ok().json(BIOSRespHelper {
            code: "200".to_owned(),
            msg: "".to_owned(),
            body: Some(body),
        }))
    }

    pub fn resp(http_code: u16, resp: &BIOSRespHelper<T>) -> BIOSResult<BIOSResp> {
        match StatusCode::from_u16(http_code) {
            Ok(code) => Ok(Ok(HttpResponse::build(code).json(resp))),
            Err(e) => Err(BIOSError::Box(Box::new(e))),
        }
    }
}

impl BIOSRespHelper<()> {
    pub fn bus_err(bus_code: &str, message: &str) -> BIOSResp {
        Ok(HttpResponse::Ok().json(BIOSRespHelper::<()> {
            code: bus_code.to_owned(),
            msg: message.to_owned(),
            body: None,
        }))
    }

    pub fn bus_error(error: BIOSError) -> BIOSResp {
        Ok(HttpResponse::Ok().json(BIOSRespHelper::<()> {
            code: format!("{}", error.status_code().as_u16()),
            msg: error.to_string(),
            body: None,
        }))
    }

    pub fn err(error: BIOSError) -> BIOSResp {
        Err(error)
    }
}
