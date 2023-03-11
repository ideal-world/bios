use bios_basic::rbum::dto::rbum_rel_agg_dto::{RbumRelAggResp, RbumRelAttrAggAddReq};
use serde::{Deserialize, Serialize};
use tardis::web::poem_openapi;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object))]
pub struct PluginBsAddReq {
    pub attrs: Option<Vec<RbumRelAttrAggAddReq>>,
}

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object))]
pub struct PluginBsInfoResp {
    pub id: String,
    pub name: String,
    pub kind_id: String,
    pub kind_code: String,
    pub kind_name: String,
    pub rel: Option<RbumRelAggResp>,
}

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object))]
pub struct PluginBsCertInfoResp {
    pub id: String,
    pub name: String,
    pub conn_uri: String,
    pub ak: String,
    pub sk: String,
    pub ext: String,
    pub private: bool,
    pub rel: Option<RbumRelAggResp>,
}
