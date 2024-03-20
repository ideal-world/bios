use serde::{Deserialize, Serialize};
use tardis::web::poem_openapi::{self};
pub type NamespaceId = String;

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Default)]
pub struct NamespaceDescriptor {
    pub namespace_id: NamespaceId,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Default)]
#[serde(default)]
pub struct NamespaceAttribute {
    pub namespace: NamespaceId,
    pub namespace_show_name: String,
    pub namespace_desc: Option<String>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
#[serde(default)]
pub struct NamespaceItem {
    pub namespace: NamespaceId,
    pub namespace_show_name: String,
    pub namespace_desc: Option<String>,
    #[serde(rename = "type")]
    pub tp: u32,
    /// quota / 容量,
    /// refer to design of nacos,
    /// see: https://github.com/alibaba/nacos/issues/4558
    pub quota: u32,
    pub config_count: u32,
}

impl Default for NamespaceItem {
    fn default() -> Self {
        NamespaceItem {
            namespace: "public".to_string(),
            namespace_show_name: "".to_string(),
            namespace_desc: None,
            quota: 200,
            config_count: 0,
            tp: NamespaceType::default().as_u32(),
        }
    }
}

#[repr(u32)]
#[derive(Debug, Default, Serialize, Deserialize, Clone, Copy)]
pub enum NamespaceType {
    Global = 0,
    #[default]
    Private = 1,
    Custom = 2,
}

impl NamespaceType {
    pub fn from_u32(v: u32) -> Self {
        match v {
            0 => NamespaceType::Global,
            1 => NamespaceType::Private,
            2 => NamespaceType::Custom,
            _ => NamespaceType::Private,
        }
    }
    pub fn as_u32(&self) -> u32 {
        *self as u32
    }
}
