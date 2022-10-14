use serde::{Deserialize, Serialize};
use tardis::chrono::{DateTime, Utc};
use tardis::web::poem_openapi;

use crate::chat_enumeration::ChatMessageKind;

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct ChatMessageAddReq {
    pub kind: ChatMessageKind,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub to_id: String,
    #[oai(validator(min_length = "2"))]
    pub content: String,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct ChatMessageInfoResp {
    pub id: String,
    pub kind: ChatMessageKind,
    pub content: String,
    pub to_id: String,
    pub owner: String,
    pub create_time: DateTime<Utc>,
}
