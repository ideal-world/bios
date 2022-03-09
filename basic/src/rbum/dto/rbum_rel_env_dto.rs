use serde::{Deserialize, Serialize};
use tardis::chrono::{DateTime, Utc};
use tardis::db::sea_orm::*;
use tardis::web::poem_openapi::Object;

use crate::rbum::enumeration::{RbumRelEnvKind, RbumScopeKind};

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct RbumRelEnvAddReq {
    pub kind: RbumRelEnvKind,
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub value1: String,
    #[oai(validator(max_length = "2000"))]
    pub value2: String,
}

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct RbumRelEnvModifyReq {
    #[oai(validator(min_length = "1", max_length = "2000"))]
    pub value1: Option<String>,
    #[oai(validator(min_length = "1", max_length = "2000"))]
    pub value2: Option<String>,
}

#[derive(Object, FromQueryResult, Serialize, Deserialize, Debug)]
pub struct RbumRelEnvDetailResp {
    pub id: String,
    pub kind: String,
    pub value1: String,
    pub value2: String,
    pub rel_rbum_rel_id: String,
    pub rel_rbum_rel_name: String,

    pub rel_app_id: String,
    pub rel_app_name: String,
    pub rel_tenant_id: String,
    pub rel_tenant_name: String,
    pub updater_id: String,
    pub updater_name: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
}
