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

use poem_openapi::param::Query;
use poem_openapi::OpenApi;

use bios::basic::error::BIOSError;
use bios::web::web_resp::BIOSResp;

pub struct Api;

#[OpenApi]
impl Api {
    #[oai(path = "/hello", method = "get")]
    async fn index(&self, name: Query<Option<String>>) -> BIOSResp<String> {
        match name.0 {
            Some(name) => BIOSResp::ok(format!("hello, {}!", name)),
            None => BIOSResp::err(BIOSError::NotFound("name does not exist".to_string())),
        }
    }
}
