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
    pub app_id: String,
    pub tenant_id: String,
    pub ak: String,
    pub account_id: String,
    pub token: String,
    pub token_kind: String,
    pub roles: Vec<String>,
    pub groups: Vec<String>,
}

impl Default for BIOSContext {
    fn default() -> Self {
        BIOSContext {
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
