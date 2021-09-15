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
    pub service_name: String,
    pub allow_tenant_register: bool,
    pub cache: IamCacheConfig,
    pub app: IamAppConfig,
    pub security: IamSecurityConfig,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct IamCacheConfig {
    pub token: String,
    pub token_rel: String,
    pub aksk: String,
    pub resources: String,
    pub change_resources: String,
    pub access_token: String,
    pub account_vcode_tmp_rel: String,
    pub account_vcode_error_times: String,
    pub change_resources_expire_sec: usize,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct IamSecurityConfig {
    pub system_admin_role_code: String,
    pub system_admin_role_name: String,
    pub tenant_admin_role_code: String,
    pub tenant_admin_role_name: String,
    pub app_admin_role_code: String,
    pub app_admin_role_name: String,
    pub default_valid_ak_rule_note: String,
    pub default_valid_ak_rule: String,
    pub default_valid_sk_rule_note: String,
    pub default_valid_sk_rule: String,
    pub default_valid_time_sec: i64,
    pub account_vcode_expire_sec: i64,
    pub account_vcode_max_error_times: i32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct IamAppConfig {
    pub tenant_name: String,
    pub app_name: String,
    pub admin_name: String,
    pub admin_password: String,
}

impl Default for IamConfig {
    fn default() -> Self {
        IamConfig {
            service_name: "iam".to_string(),
            allow_tenant_register: true,
            cache: Default::default(),
            app: Default::default(),
            security: Default::default(),
        }
    }
}

impl Default for IamCacheConfig {
    fn default() -> Self {
        IamCacheConfig {
            token: "bios:iam:token:info:".to_string(),
            token_rel: "bios:iam:token:rel:".to_string(),
            aksk: "bios:iam:app:aksk:".to_string(),
            resources: "bios:iam:resources".to_string(),
            change_resources: "bios:iam:change_resources:".to_string(),
            access_token: "bios:iam:oauth:access-token:".to_string(),
            account_vcode_tmp_rel: "bios:iam:account:vocde:tmprel:".to_string(),
            account_vcode_error_times: "bios:iam:account:vocde:errortimes:".to_string(),
            change_resources_expire_sec: 30,
        }
    }
}

impl Default for IamAppConfig {
    fn default() -> Self {
        IamAppConfig {
            tenant_name: "平台租户".to_string(),
            app_name: "用户权限应用".to_string(),
            admin_name: "bios_admin".to_string(),
            admin_password: bios::basic::field::uuid(),
        }
    }
}

impl Default for IamSecurityConfig {
    fn default() -> Self {
        IamSecurityConfig {
            system_admin_role_code: "SYSTEM_ADMIN".to_string(),
            system_admin_role_name: "系统管理员".to_string(),
            tenant_admin_role_code: "TENANT_ADMIN".to_string(),
            tenant_admin_role_name: "租户管理员".to_string(),
            app_admin_role_code: "APP_ADMIN".to_string(),
            app_admin_role_name: "应用管理员".to_string(),
            default_valid_ak_rule_note: "用户名校验规则".to_string(),
            default_valid_ak_rule: "^[a-zA-Z_\\d\\.]{3,20}$".to_string(),
            default_valid_sk_rule_note: "密码校验规则，8-40位字符".to_string(),
            default_valid_sk_rule: "^.{8,40}$".to_string(),
            default_valid_time_sec: 60 * 60 * 24 * 30,
            account_vcode_expire_sec: 60 * 5,
            account_vcode_max_error_times: 5,
        }
    }
}
