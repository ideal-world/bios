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

use serde::Deserialize;

#[derive(Deserialize)]
pub struct IdentInfo {
    pub app_id: Option<String>,
    pub tenant_id: Option<String>,
    pub account_id: Option<String>,
    pub token: Option<String>,
    pub token_kind: Option<String>,
    pub ak: Option<String>,
    pub roles: Option<Vec<String>>,
    pub groups: Option<Vec<String>>,
}

#[derive(Deserialize)]
pub struct IdentPubInfo {}

#[derive(Deserialize)]
pub struct IdentAppInfo {
    pub app_id: String,
    pub tenant_id: String,
    pub ak: String,
}

#[derive(Deserialize)]
pub struct IdentAccountInfo {
    pub app_id: String,
    pub tenant_id: String,
    pub ak: String,
    pub account_id: String,
    pub token: String,
    pub token_kind: String,
    pub roles: Vec<String>,
    pub groups: Vec<String>,
}
