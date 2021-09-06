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

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct WorkSpaceConfig {
    pub iam: IamConfig,
}

impl Default for WorkSpaceConfig {
    fn default() -> Self {
        WorkSpaceConfig { iam: IamConfig::default() }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct IamConfig {
    pub cache_token: String,
    pub cache_aksk: String,
    pub cache_resources: String,
    pub cache_change_resources: String,
}

impl Default for IamConfig {
    fn default() -> Self {
        IamConfig {
            cache_token: "bios:iam:token:info:".to_string(),
            cache_aksk: "bios:iam:app:aksk:".to_string(),
            cache_resources: "bios:iam:resources".to_string(),
            cache_change_resources: "bios:iam:change_resources".to_string(),
        }
    }
}
