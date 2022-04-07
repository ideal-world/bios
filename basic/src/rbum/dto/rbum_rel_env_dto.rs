use serde::{Deserialize, Serialize};
use tardis::chrono::{DateTime, Utc};
use tardis::web::poem_openapi::Object;

use crate::rbum::rbum_enumeration::RbumRelEnvKind;

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct RbumRelEnvAddReq {
    pub kind: RbumRelEnvKind,
    #[oai(validator(min_length = "1", max_length = "2000"))]
    pub value1: String,
    #[oai(validator(min_length = "1", max_length = "2000"))]
    pub value2: Option<String>,

    #[oai(validator(min_length = "2", max_length = "255"))]
    pub rel_rbum_rel_id: String,
}

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct RbumRelEnvModifyReq {
    #[oai(validator(min_length = "1", max_length = "2000"))]
    pub value1: Option<String>,
    #[oai(validator(min_length = "1", max_length = "2000"))]
    pub value2: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(tardis::web::poem_openapi::Object, tardis::db::sea_orm::FromQueryResult))]
pub struct RbumRelEnvDetailResp {
    pub id: String,
    pub kind: String,
    pub value1: String,
    pub value2: String,
    pub rel_rbum_rel_id: String,

    pub own_paths: String,
    pub owner: String,
    pub owner_name: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
}
