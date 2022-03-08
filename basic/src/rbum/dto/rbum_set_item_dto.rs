use serde::{Deserialize, Serialize};
use tardis::chrono::{DateTime, Utc};
use tardis::db::sea_orm::*;
use tardis::web::poem_openapi::Object;

use crate::rbum::enumeration::RbumScopeKind;

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct RbumSetItemAddReq {
}

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct RbumSetItemModifyReq {
}

#[derive(Object, FromQueryResult, Serialize, Deserialize, Debug)]
pub struct RbumSetItemSummaryResp {
}

#[derive(Object, FromQueryResult, Serialize, Deserialize, Debug)]
pub struct RbumSetItemDetailResp {
}
