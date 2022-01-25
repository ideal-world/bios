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

use std::collections::HashMap;
use std::time::Duration;

use log::info;
use reqwest::{Client, Method, Response};
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::basic::error::BIOSError;
use crate::basic::result::BIOSResult;
use crate::FrameworkConfig;

pub struct BIOSWebClient {
    client: Client,
}

impl BIOSWebClient {
    pub fn init_by_conf(conf: &FrameworkConfig) -> BIOSResult<BIOSWebClient> {
        BIOSWebClient::init(conf.web_client.connect_timeout_sec)
    }

    pub fn init(connect_timeout_sec: u64) -> BIOSResult<BIOSWebClient> {
        info!("[BIOS.Framework.WebClient] Initializing",);
        let client = reqwest::Client::builder().connect_timeout(Duration::from_secs(connect_timeout_sec)).https_only(false).build().unwrap();
        info!("[BIOS.Framework.WebClient] Initialized",);
        BIOSResult::Ok(BIOSWebClient { client })
    }

    pub async fn get_to_str(&self, url: &str, headers: Option<Vec<(String, String)>>) -> BIOSResult<BIOSHttpResponse<String>> {
        let (code, headers, response) = self.request::<()>(Method::GET, url, headers, None, None).await?;
        self.to_text(code, headers, response).await
    }

    pub async fn get<T: DeserializeOwned>(&self, url: &str, headers: Option<Vec<(String, String)>>) -> BIOSResult<BIOSHttpResponse<T>> {
        let (code, headers, response) = self.request::<()>(Method::GET, url, headers, None, None).await?;
        self.to_json::<T>(code, headers, response).await
    }

    pub async fn head(&self, url: &str, headers: Option<Vec<(String, String)>>) -> BIOSResult<BIOSHttpResponse<()>> {
        let (code, headers, _) = self.request::<()>(Method::HEAD, url, headers, None, None).await?;
        Ok(BIOSHttpResponse { code, headers, body: None })
    }

    pub async fn delete(&self, url: &str, headers: Option<Vec<(String, String)>>) -> BIOSResult<BIOSHttpResponse<()>> {
        let (code, headers, _) = self.request::<()>(Method::DELETE, url, headers, None, None).await?;
        Ok(BIOSHttpResponse { code, headers, body: None })
    }

    pub async fn post_str_to_str(&self, url: &str, body: &String, headers: Option<Vec<(String, String)>>) -> BIOSResult<BIOSHttpResponse<String>> {
        let (code, headers, response) = self.request::<()>(Method::POST, url, headers, None, Some(body)).await?;
        self.to_text(code, headers, response).await
    }

    pub async fn post_obj_to_str<B: Serialize>(&self, url: &str, body: &B, headers: Option<Vec<(String, String)>>) -> BIOSResult<BIOSHttpResponse<String>> {
        let (code, headers, response) = self.request::<B>(Method::POST, url, headers, Some(body), None).await?;
        self.to_text(code, headers, response).await
    }

    pub async fn post_to_obj<T: DeserializeOwned>(&self, url: &str, body: &String, headers: Option<Vec<(String, String)>>) -> BIOSResult<BIOSHttpResponse<T>> {
        let (code, headers, response) = self.request::<()>(Method::POST, url, headers, None, Some(body)).await?;
        self.to_json::<T>(code, headers, response).await
    }

    pub async fn post<B: Serialize, T: DeserializeOwned>(&self, url: &str, body: &B, headers: Option<Vec<(String, String)>>) -> BIOSResult<BIOSHttpResponse<T>> {
        let (code, headers, response) = self.request(Method::POST, url, headers, Some(body), None).await?;
        self.to_json::<T>(code, headers, response).await
    }

    pub async fn put_str_to_str(&self, url: &str, body: &String, headers: Option<Vec<(String, String)>>) -> BIOSResult<BIOSHttpResponse<String>> {
        let (code, headers, response) = self.request::<()>(Method::PUT, url, headers, None, Some(body)).await?;
        self.to_text(code, headers, response).await
    }

    pub async fn put_obj_to_str<B: Serialize>(&self, url: &str, body: &B, headers: Option<Vec<(String, String)>>) -> BIOSResult<BIOSHttpResponse<String>> {
        let (code, headers, response) = self.request::<B>(Method::PUT, url, headers, Some(body), None).await?;
        self.to_text(code, headers, response).await
    }

    pub async fn put_to_obj<T: DeserializeOwned>(&self, url: &str, body: &String, headers: Option<Vec<(String, String)>>) -> BIOSResult<BIOSHttpResponse<T>> {
        let (code, headers, response) = self.request::<()>(Method::PUT, url, headers, None, Some(body)).await?;
        self.to_json::<T>(code, headers, response).await
    }

    pub async fn put<B: Serialize, T: DeserializeOwned>(&self, url: &str, body: &B, headers: Option<Vec<(String, String)>>) -> BIOSResult<BIOSHttpResponse<T>> {
        let (code, headers, response) = self.request(Method::PUT, url, headers, Some(body), None).await?;
        self.to_json::<T>(code, headers, response).await
    }

    pub async fn patch_str_to_str(&self, url: &str, body: &String, headers: Option<Vec<(String, String)>>) -> BIOSResult<BIOSHttpResponse<String>> {
        let (code, headers, response) = self.request::<()>(Method::PATCH, url, headers, None, Some(body)).await?;
        self.to_text(code, headers, response).await
    }

    pub async fn patch_obj_to_str<B: Serialize>(&self, url: &str, body: &B, headers: Option<Vec<(String, String)>>) -> BIOSResult<BIOSHttpResponse<String>> {
        let (code, headers, response) = self.request::<B>(Method::PATCH, url, headers, Some(body), None).await?;
        self.to_text(code, headers, response).await
    }

    pub async fn patch_to_obj<T: DeserializeOwned>(&self, url: &str, body: &String, headers: Option<Vec<(String, String)>>) -> BIOSResult<BIOSHttpResponse<T>> {
        let (code, headers, response) = self.request::<()>(Method::PATCH, url, headers, None, Some(body)).await?;
        self.to_json::<T>(code, headers, response).await
    }

    pub async fn patch<B: Serialize, T: DeserializeOwned>(&self, url: &str, body: &B, headers: Option<Vec<(String, String)>>) -> BIOSResult<BIOSHttpResponse<T>> {
        let (code, headers, response) = self.request(Method::PATCH, url, headers, Some(body), None).await?;
        self.to_json::<T>(code, headers, response).await
    }

    async fn request<B: Serialize>(
        &self,
        method: Method,
        url: &str,
        headers: Option<Vec<(String, String)>>,
        body: Option<&B>,
        str_body: Option<&String>,
    ) -> BIOSResult<(u16, HashMap<String, String>, Response)> {
        let mut result = self.client.request(method, url);
        if let Some(headers) = headers {
            for (key, value) in headers {
                result = result.header(key, value);
            }
        }
        if let Some(body) = body {
            result = result.json(body);
        }
        if let Some(body) = str_body {
            result = result.body(body.to_string());
        }
        let response = result.send().await?;
        let code = response.status().as_u16();
        let headers = response.headers().iter().map(|(k, v)| (k.to_string(), v.to_str().unwrap().to_string())).collect();
        Ok((code, headers, response))
    }

    async fn to_text(&self, code: u16, headers: HashMap<String, String>, response: Response) -> BIOSResult<BIOSHttpResponse<String>> {
        match response.text().await {
            Ok(body) => Ok(BIOSHttpResponse { code, headers, body: Some(body) }),
            Err(err) => Err(BIOSError::Box(Box::new(err))),
        }
    }

    async fn to_json<T: DeserializeOwned>(&self, code: u16, headers: HashMap<String, String>, response: Response) -> BIOSResult<BIOSHttpResponse<T>> {
        match response.json().await {
            Ok(body) => Ok(BIOSHttpResponse { code, headers, body: Some(body) }),
            Err(err) => Err(BIOSError::Box(Box::new(err))),
        }
    }

    pub fn raw(&self) -> &Client {
        &self.client
    }
}

#[derive(Debug, Clone)]
pub struct BIOSHttpResponse<T> {
    pub code: u16,
    pub headers: HashMap<String, String>,
    pub body: Option<T>,
}

impl From<reqwest::Error> for BIOSError {
    fn from(error: reqwest::Error) -> Self {
        BIOSError::Box(Box::new(error))
    }
}
