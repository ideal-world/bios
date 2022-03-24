use serde::{Deserialize, Serialize};
use tardis::web::poem_openapi::Object;

use crate::rbum::dto::rbum_rel_attr_dto::RbumRelAttrDetailResp;
use crate::rbum::dto::rbum_rel_dto::{RbumRelAddReq, RbumRelDetailResp};
use crate::rbum::dto::rbum_rel_env_dto::RbumRelEnvDetailResp;
use crate::rbum::rbum_enumeration::RbumRelEnvKind;

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct RbumRelAggAddReq {
    pub rel: RbumRelAddReq,

    pub attrs: Vec<RbumRelAttrAggAddReq>,
    pub envs: Vec<RbumRelEnvAggAddReq>,
}

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct RbumRelAttrAggAddReq {
    pub is_from: bool,
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub value: String,

    #[oai(validator(min_length = "2", max_length = "255"))]
    pub rel_rbum_kind_attr_id: String,
}

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct RbumRelEnvAggAddReq {
    pub kind: RbumRelEnvKind,
    #[oai(validator(min_length = "1", max_length = "2000"))]
    pub value1: String,
    #[oai(validator(min_length = "1", max_length = "2000"))]
    pub value2: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(tardis::web::poem_openapi::Object))]
pub struct RbumRelAggResp {
    pub rel: RbumRelDetailResp,
    pub attrs: Vec<RbumRelAttrDetailResp>,
    pub envs: Vec<RbumRelEnvDetailResp>,
}
