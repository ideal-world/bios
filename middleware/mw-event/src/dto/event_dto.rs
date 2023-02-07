use bios_basic::rbum::dto::rbum_filer_dto::{RbumBasicFilterReq, RbumItemFilterFetcher};
use serde::{Deserialize, Serialize};
use tardis::{
    basic::field::TrimString,
    db::sea_orm::{self},
    web::poem_openapi,
};

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct EventDefAddOrModifyReq {
    #[oai(validator(pattern = r"^[a-z0-9]+$"))]
    pub code: TrimString,
    pub name: TrimString,
    pub save_message: bool,
    pub need_mgr: bool,
    pub queue_size: u16,
    pub use_sk: Option<String>,
    pub mgr_sk: Option<String>,
}

#[derive(poem_openapi::Object, sea_orm::FromQueryResult, Serialize, Deserialize, Debug)]
pub struct EventDefInfoResp {
    #[oai(validator(pattern = r"^[a-z0-9]+$"))]
    pub code: String,
    pub name: String,
    pub save_message: bool,
    pub need_mgr: bool,
    pub queue_size: u16,
    pub use_sk: String,
    pub mgr_sk: String,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct EventDefFilterReq {
    pub basic: RbumBasicFilterReq,
}

impl RbumItemFilterFetcher for EventDefFilterReq {
    fn basic(&self) -> &RbumBasicFilterReq {
        &self.basic
    }

    fn rel(&self) -> &Option<bios_basic::rbum::dto::rbum_filer_dto::RbumItemRelFilterReq> {
        &None
    }

    fn rel2(&self) -> &Option<bios_basic::rbum::dto::rbum_filer_dto::RbumItemRelFilterReq> {
        &None
    }
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct EventListenerRegisterReq {
    #[oai(validator(pattern = r"^[a-z0-9]+$"))]
    pub event_code: TrimString,
    pub event_sk: Option<String>,
    #[oai(validator(pattern = r"^[a-z0-9]+$"))]
    pub listener_code: TrimString,
    pub channel: EventChannelKind,
    pub subscribe_mode: bool,
    pub callback_url: Option<String>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct EventListenerRegisterResp {
    pub ws_addr: Option<String>,
    pub http_addr: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EventListenerInfo {
    pub channel: EventChannelKind,
    pub subscribe_mode: bool,
    pub mgr: bool,
    pub token:String,
}

#[derive(poem_openapi::Enum, Serialize, Deserialize, Debug)]
pub enum EventChannelKind {
    #[oai(rename = "ws")]
    Ws,
    #[oai(rename = "http")]
    Http,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EventMsgReq {
    pub msg: String,
    pub to_sessions: Vec<String>,
}

