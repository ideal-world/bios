use std::default;

use serde::{Deserialize, Serialize};
use tardis::{
    basic::field::TrimString,
    chrono::{DateTime, Utc},
    web::poem_openapi,
};
pub type NamespaceId = String;

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Default)]
pub struct NamespaceDescriptor {
    namespace_id: NamespaceId,
}


#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Default)]
pub struct NamespaceAttribute {
    pub namespace: NamespaceId,
    pub namespace_show_name: String,
    pub namespace_desc: Option<String>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Default)]
#[serde(default)]
pub struct NamespaceItem {
    pub namespace: NamespaceId,
    pub namespace_show_name: String,
    pub namespace_desc: Option<String>,
    pub quota: u32,
    pub config_count: u32,
    #[serde(rename="type")]
    pub tp: u32,
}

#[repr(u32)]
#[derive(Debug, Default, Serialize, Deserialize, Clone, Copy)]
pub enum NamesapceType {
    Global = 0,
    #[default]
    Private = 1,
    Custom = 2
}

impl NamesapceType {
    pub fn from_u32(v: u32) -> Self {
        match v {
            0 => NamesapceType::Global,
            1 => NamesapceType::Private,
            2 => NamesapceType::Custom,
            _ => NamesapceType::Private
        }
    }
    pub fn as_u32(&self) -> u32 {
        *self as u32
    }
}