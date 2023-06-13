use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tardis::{
    basic::{dto::TardisContext, error::TardisError},
    web::poem_openapi,
    TardisFuns,
};

use crate::auth_config::AuthConfig;

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Clone)]
pub struct AuthReq {
    pub scheme: String,
    pub path: String,
    pub query: HashMap<String, String>,
    pub method: String,
    pub host: String,
    pub port: u16,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct AuthResp {
    pub allow: bool,
    pub status_code: u16,
    pub reason: Option<String>,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
}

impl AuthResp {
    fn init_common_header(config: &AuthConfig) -> HashMap<String, String> {
        HashMap::from([
            ("Access-Control-Allow-Origin".to_string(), config.cors_allow_origin.clone()),
            ("Access-Control-Allow-Methods".to_string(), config.cors_allow_methods.clone()),
            ("Access-Control-Allow-Headers".to_string(), config.cors_allow_headers.clone()),
            ("Access-Control-Max-Age".to_string(), "3600000".to_string()),
            ("Access-Control-Allow-Credentials".to_string(), "true".to_string()),
            ("Access-Control-Allow-Credentials".to_string(), "true".to_string()),
            ("Content-Type".to_string(), "application/json".to_string()),
        ])
    }

    pub(crate) fn ok(ctx: Option<&AuthContext>, resp_body: Option<String>, resp_headers: Option<HashMap<String, String>>, config: &AuthConfig) -> Self {
        let mut headers = Self::init_common_header(config);
        if let Some(resp_headers) = resp_headers {
            headers.extend(resp_headers);
        }
        headers.insert(
            config.head_key_context.to_string(),
            if let Some(ctx) = ctx {
                let ctx = TardisContext {
                    own_paths: ctx.own_paths.as_deref().unwrap_or_default().to_string(),
                    ak: ctx.ak.as_deref().unwrap_or_default().to_string(),
                    owner: ctx.account_id.as_deref().unwrap_or_default().to_string(),
                    roles: if let Some(roles) = &ctx.roles { roles.clone() } else { vec![] },
                    groups: if let Some(groups) = &ctx.groups { groups.clone() } else { vec![] },
                    ext: Default::default(),
                    sync_task_fns: Default::default(),
                    async_task_fns: Default::default(),
                };
                TardisFuns::crypto.base64.encode(&TardisFuns::json.obj_to_string(&ctx).unwrap_or_default())
            } else {
                "".to_string()
            },
        );
        Self {
            allow: true,
            status_code: 200,
            reason: None,
            headers,
            body: resp_body,
        }
    }

    pub(crate) fn err(e: TardisError, config: &AuthConfig) -> Self {
        Self {
            allow: false,
            status_code: e.code.parse().unwrap_or(500),
            reason: Some(e.message),
            headers: AuthResp::init_common_header(config),
            body: None,
        }
    }
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct MixAuthResp {
    pub url: String,
    pub method: String,
    pub allow: bool,
    pub status_code: u16,
    pub reason: Option<String>,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
}

pub struct AuthContext {
    pub rbum_uri: String,
    pub rbum_action: String,
    pub app_id: Option<String>,
    pub tenant_id: Option<String>,
    pub account_id: Option<String>,
    pub roles: Option<Vec<String>>,
    pub groups: Option<Vec<String>>,
    pub own_paths: Option<String>,
    pub ak: Option<String>,
}

#[derive(Serialize, Deserialize, Default)]
pub struct ResContainerNode {
    children: Option<HashMap<String, ResContainerNode>>,
    leaf_info: Option<ResContainerLeafInfo>,
}

impl ResContainerNode {
    pub fn new() -> Self {
        ResContainerNode {
            children: Some(HashMap::new()),
            leaf_info: None,
        }
    }
    pub fn has_child(&self, key: &str) -> bool {
        self.children.as_ref().map(|n| n.contains_key(key)).unwrap_or(false)
    }

    pub fn child_len(&self) -> usize {
        self.children.as_ref().map(|n| n.len()).unwrap_or(0)
    }

    pub fn insert_child(&mut self, key: &str) {
        self.children.as_mut().expect("[Auth.kernel] children get none").insert(key.to_string(), ResContainerNode::new());
    }

    pub fn get_child(&self, key: &str) -> &ResContainerNode {
        self.children.as_ref().expect("[Auth.kernel] children get none").get(key).unwrap_or_else(|| panic!("[Auth.kernel] children get key:{key} none"))
    }

    pub fn get_child_mut(&mut self, key: &str) -> &mut ResContainerNode {
        self.children.as_mut().expect("[Auth.kernel] children get none").get_mut(key).unwrap_or_else(|| panic!("[Auth.kernel] children get key:{key} none"))
    }

    pub fn get_child_opt(&self, key: &str) -> Option<&ResContainerNode> {
        self.children.as_ref().expect("[Auth.kernel] children get none").get(key)
    }

    pub fn remove_child(&mut self, key: &str) {
        self.children.as_mut().expect("[Auth.kernel] children get none").remove(key);
    }

    pub fn insert_leaf(
        &mut self,
        key: &str,
        res_action: &str,
        res_uri: &str,
        auth_info: Option<ResAuthInfo>,
        need_crypto_req: bool,
        need_crypto_resp: bool,
        need_double_auth: bool,
    ) {
        self.children.as_mut().expect("[Auth.kernel] children get none").insert(
            key.to_string(),
            ResContainerNode {
                children: None,
                leaf_info: Some(ResContainerLeafInfo {
                    action: res_action.to_string(),
                    uri: res_uri.to_string(),
                    auth: auth_info,
                    need_crypto_req,
                    need_crypto_resp,
                    need_double_auth,
                }),
            },
        );
    }

    pub fn get_leaf_info(&self) -> ResContainerLeafInfo {
        self.leaf_info.as_ref().expect("[Auth.kernel] leaf_info get none").clone()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ResContainerLeafInfo {
    pub action: String,
    pub uri: String,
    pub auth: Option<ResAuthInfo>,
    pub need_crypto_req: bool,
    pub need_crypto_resp: bool,
    pub need_double_auth: bool,
}

#[derive(Serialize, Deserialize, Default)]
pub(crate) struct ServConfig {
    pub strict_security_mode: bool,
    pub pub_key: String,
    pub double_auth_exp_sec: u32,
    pub apis: Vec<Api>,
    pub login_req_method: String,
    pub login_req_paths: Vec<String>,
    pub logout_req_method: String,
    pub logout_req_path: String,
    pub double_auth_req_method: String,
    pub double_auth_req_path: String,
}
#[derive(Serialize, Deserialize, Default, Clone)]
pub(crate) struct Api {
    pub action: String,
    pub uri: String,
    pub need_crypto_req: bool,
    pub need_crypto_resp: bool,
    pub need_double_auth: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ResAuthInfo {
    pub accounts: Option<String>,
    pub roles: Option<String>,
    pub groups: Option<String>,
    pub apps: Option<String>,
    pub tenants: Option<String>,
    pub st: Option<i64>,
    pub et: Option<i64>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct MixRequest {
    pub body: String,
    pub headers: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MixRequestBody {
    pub method: String,
    pub uri: String,
    pub body: String,
    pub headers: HashMap<String, String>,
    pub ts: f64,
}
