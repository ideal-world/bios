use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tardis::{
    basic::{dto::TardisContext, error::TardisError},
    web::poem_openapi,
    TardisFuns,
};

use crate::auth_config::AuthConfig;

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct AuthReq {
    pub scheme: String,
    pub path: String,
    pub query: String,
    pub method: String,
    pub host: String,
    pub port: u16,
    pub headers: HashMap<String, String>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct AuthResp {
    pub allow: bool,
    pub status_code: u16,
    pub reason: Option<String>,
    pub headers: HashMap<String, String>,
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

    pub(crate) fn ok(ctx: Option<&AuthContext>, config: &AuthConfig) -> Self {
        let mut headers = Self::init_common_header(config);
        headers.insert(
            config.head_key_context.to_string(),
            if let Some(ctx) = ctx {
                let ctx = TardisContext {
                    own_paths: ctx.own_paths.as_deref().unwrap_or_default().to_string(),
                    ak: ctx.ak.as_deref().unwrap_or_default().to_string(),
                    owner: ctx.iam_account_id.as_deref().unwrap_or_default().to_string(),
                    roles: if let Some(roles) = &ctx.iam_roles { roles.clone() } else { vec![] },
                    groups: if let Some(groups) = &ctx.iam_groups { groups.clone() } else { vec![] },
                    ext: Default::default(),
                };
                TardisFuns::crypto.base64.encode(&TardisFuns::json.obj_to_string(&ctx).unwrap())
            } else {
                "".to_string()
            },
        );
        Self {
            allow: true,
            status_code: 200,
            reason: None,
            headers,
        }
    }

    pub(crate) fn err(e: TardisError, config: &AuthConfig) -> Self {
        Self {
            allow: false,
            status_code: e.code.parse().unwrap_or(500),
            reason: Some(e.message),
            headers: AuthResp::init_common_header(config),
        }
    }
}

pub struct AuthContext {
    pub rbum_uri: String,
    pub rbum_action: String,
    pub iam_app_id: Option<String>,
    pub iam_tenant_id: Option<String>,
    pub iam_account_id: Option<String>,
    pub iam_roles: Option<Vec<String>>,
    pub iam_groups: Option<Vec<String>>,
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
        self.children.as_mut().unwrap().insert(key.to_string(), ResContainerNode::new());
    }

    pub fn get_child(&self, key: &str) -> &ResContainerNode {
        self.children.as_ref().unwrap().get(key).unwrap()
    }

    pub fn get_child_mut(&mut self, key: &str) -> &mut ResContainerNode {
        self.children.as_mut().unwrap().get_mut(key).unwrap()
    }

    pub fn get_child_opt(&self, key: &str) -> Option<&ResContainerNode> {
        self.children.as_ref().unwrap().get(key)
    }

    pub fn remove_child(&mut self, key: &str) {
        self.children.as_mut().unwrap().remove(key);
    }

    pub fn insert_leaf(&mut self, key: &str, res_action: &str, res_uri: &str, auth_info: &ResAuthInfo) {
        self.children.as_mut().unwrap().insert(
            key.to_string(),
            ResContainerNode {
                children: None,
                leaf_info: Some(ResContainerLeafInfo {
                    action: res_action.to_string(),
                    uri: res_uri.to_string(),
                    auth: auth_info.clone(),
                }),
            },
        );
    }

    pub fn get_leaf_info(&self) -> ResContainerLeafInfo {
        self.leaf_info.as_ref().unwrap().clone()
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ResContainerLeafInfo {
    pub action: String,
    pub uri: String,
    pub auth: ResAuthInfo,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ResAuthInfo {
    pub accounts: Option<String>,
    pub roles: Option<String>,
    pub groups: Option<String>,
    pub apps: Option<String>,
    pub tenants: Option<String>,
}
