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

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone)]
#[serde(default)]
pub struct BIOSContext {
    pub ident: IdentInfo,
    pub trace: Trace,
    pub lang: String,
}

impl Default for BIOSContext {
    fn default() -> Self {
        BIOSContext {
            ident: Default::default(),
            trace: Default::default(),
            lang: "en_US".to_string(),
        }
    }
}

#[derive(Deserialize, Serialize, Clone)]
#[serde(default)]
pub struct Trace {
    pub id: String,
    pub app: String,
    pub inst: String,
}

impl Default for Trace {
    fn default() -> Self {
        Trace {
            id: "".to_string(),
            app: "".to_string(),
            inst: "".to_string(),
        }
    }
}

#[derive(Deserialize, Serialize, Clone)]
#[serde(default)]
pub struct IdentInfo {
    pub app_id: String,
    pub tenant_id: String,
    pub ak: String,
    pub account_id: String,
    pub token: String,
    pub token_kind: String,
    pub roles: Vec<String>,
    pub groups: Vec<String>,
}

impl Default for IdentInfo {
    fn default() -> Self {
        IdentInfo {
            app_id: "".to_string(),
            tenant_id: "".to_string(),
            ak: "".to_string(),
            account_id: "".to_string(),
            token: "".to_string(),
            token_kind: "".to_string(),
            roles: vec![],
            groups: vec![],
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct BIOSResp<'c, T>
where
    T: Serialize,
{
    pub code: String,
    pub msg: String,
    pub body: Option<T>,
    pub trace_id: Option<String>,
    pub trace_app: Option<String>,
    pub trace_inst: Option<String>,
    #[serde(skip)]
    pub ctx: Option<&'c BIOSContext>,
}

impl<T> Default for BIOSResp<'_, T>
where
    T: Serialize,
{
    fn default() -> Self {
        BIOSResp {
            code: "".to_owned(),
            msg: "".to_owned(),
            body: None,
            trace_id: None,
            trace_app: None,
            trace_inst: None,
            ctx: None,
        }
    }
}
