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

use std::time::Duration;

use actix_http::client::SendRequestError;
use actix_http::encoding::Decoder;
use actix_http::error::PayloadError;
use actix_http::{Payload, PayloadStream};
use awc::Connector;
use awc::{Client, ClientResponse};
use log::info;

use crate::basic::error::{BIOSError, BIOSResult};

pub struct BIOSWebClient;

impl BIOSWebClient {
    pub fn init(connect_timeout_sec: u64, request_timeout_sec: u64) -> Client {
        info!(
            "[BIOS.Framework.WebClient] Initializing, connect_timeout_sec:{}, request_timeout_sec:{}",
            connect_timeout_sec,
            request_timeout_sec
        );
        let client = Client::builder()
            .connector(
                Connector::new()
                    .timeout(Duration::from_secs(connect_timeout_sec))
                    .finish(),
            )
            .timeout(Duration::from_secs(request_timeout_sec))
            .finish();
        info!(
            "[BIOS.Framework.WebClient] Initialized, connect_timeout_sec:{}, request_timeout_sec:{}",
            connect_timeout_sec,
            request_timeout_sec
        );
        client
    }

    pub async fn body_as_str(
        response: &mut ClientResponse<Decoder<Payload<PayloadStream>>>,
    ) -> BIOSResult<String> {
        Ok(String::from_utf8(response.body().await?.to_vec())?)
    }
}

impl From<PayloadError> for BIOSError {
    fn from(error: PayloadError) -> Self {
        BIOSError::Box(Box::new(error))
    }
}

impl From<SendRequestError> for BIOSError {
    fn from(error: SendRequestError) -> Self {
        match error {
            SendRequestError::Url(e) => BIOSError::Box(Box::new(e)),
            SendRequestError::Connect(e) => BIOSError::Box(Box::new(e)),
            SendRequestError::Send(e) => BIOSError::Box(Box::new(e)),
            SendRequestError::Response(e) => BIOSError::FormatError(e.to_string()),
            SendRequestError::Http(e) => BIOSError::Box(Box::new(e)),
            SendRequestError::H2(e) => BIOSError::Box(Box::new(e)),
            SendRequestError::Timeout => BIOSError::Timeout(error.to_string()),
            SendRequestError::TunnelNotSupported => BIOSError::Timeout(error.to_string()),
            SendRequestError::Body(e) => BIOSError::IOError(e.to_string()),
        }
    }
}
