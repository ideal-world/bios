use serde::{Deserialize, Serialize};
use tardis::{basic::field::TrimString, serde_json, web::poem_openapi};

use super::conf_config_nacos_dto::{NacosCreateNamespaceRequest, NacosDeleteNamespaceRequest, NacosEditNamespaceRequest, PublishConfigForm};

#[derive(Debug, Serialize, Deserialize, poem_openapi::Object)]
pub struct RegisterResponse {
    pub username: String,
    pub password: String,
}

impl RegisterResponse {
    pub fn new(username: impl Into<String>, password: impl Into<String>) -> Self {
        Self {
            username: username.into(),
            password: password.into(),
        }
    }
}

pub struct NacosAuth<'a> {
    pub username: &'a str,
    pub password: &'a str,
}

macro_rules! derive_into_nacos_auth {
    (
        $($T:ty),*$(,)?
    ) => {
        $(
            impl<'a> From<&'a $T> for Option<NacosAuth<'a>> {
                fn from(value: &'a $T) -> Self {
                    let username = value.username.as_deref()?;
                    let password = value.password.as_deref()?;
                    Some(NacosAuth {
                        username,
                        password
                    })
                }
            }
        )*
    };
}

derive_into_nacos_auth! {
    NacosCreateNamespaceRequest,
    NacosEditNamespaceRequest,
    NacosDeleteNamespaceRequest,
    PublishConfigForm
}

#[derive(Debug, Serialize, Deserialize, poem_openapi::Object, Default)]
pub struct RegisterRequest {
    #[oai(validator(pattern = r"^[a-zA-Z\d_]{5,16}$"))]
    pub username: Option<TrimString>,
    #[oai(validator(pattern = r"^[a-zA-Z\d~!@#$%^&*\(\)_+]{8,16}$"))]
    pub password: Option<TrimString>,
}

#[derive(Debug, Serialize, Deserialize, poem_openapi::Object, Default)]
pub struct RegisterBundleRequest {
    pub backend_service: Option<serde_json::Value>,
    pub app_tenant_id: Option<String>,
    #[oai(flatten)]
    #[serde(flatten)]
    pub register_request: RegisterRequest,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum BackendServiceSource {
    Id(String),
    #[default]
    Default,
    New {
        name: Option<String>,
        // conn_uri: String,
        //
        // kind_code: Option<String>,
    },
}

#[derive(Debug, Serialize, Deserialize, poem_openapi::Object, Default)]
pub struct ChangePasswordRequest {
    #[oai(validator(pattern = r"^[a-zA-Z\d_]{5,16}$"))]
    pub username: TrimString,
    #[oai(validator(pattern = r"^[a-zA-Z\d~!@#$%^&*\(\)_+]{8,16}$"))]
    pub old_password: TrimString,
    #[oai(validator(pattern = r"^[a-zA-Z\d~!@#$%^&*\(\)_+]{8,16}$"))]
    pub password: Option<TrimString>,
}

impl RegisterRequest {
    #[inline]
    pub fn ak(&self) -> Option<&str> {
        self.username.as_deref()
    }
    #[inline]
    pub fn old_sk(&self) -> Option<&str> {
        self.password.as_deref()
    }
    #[inline]
    pub fn sk(&self) -> Option<&str> {
        self.password.as_deref()
    }
}

impl ChangePasswordRequest {
    #[inline]
    pub fn ak(&self) -> &str {
        self.username.as_str()
    }
    #[inline]
    pub fn sk(&self) -> Option<&str> {
        self.password.as_deref()
    }
}
