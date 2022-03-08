use serde::{Deserialize, Serialize};
use tardis::chrono::{DateTime, Utc};
use tardis::db::sea_orm::*;
use tardis::web::poem_openapi::Object;

use crate::rbum::enumeration::RbumScopeKind;

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct RbumCertAddReq {
}

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct RbumCertModifyReq {
}

#[derive(Object, FromQueryResult, Serialize, Deserialize, Debug)]
pub struct RbumCertSummaryResp {
}

#[derive(Object, FromQueryResult, Serialize, Deserialize, Debug)]
pub struct RbumCertDetailResp {
}
