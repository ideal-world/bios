use bios_basic::rbum::dto::{rbum_kind_dto::RbumKindDetailResp, rbum_rel_dto::RbumRelBoneResp};
use serde::{Deserialize, Serialize};
use tardis::web::poem_openapi;

use super::plugin_bs_dto::{PluginBsAddReq, PluginBsInfoResp};

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct PluginKindAddAggReq {
    pub kind_id: String,
    pub app_tenant_id: String,
    pub bs_id: String,
    pub bs_rel: Option<PluginBsAddReq>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct PluginKindAggResp {
    pub kind: RbumKindDetailResp,
    pub rel_bind: Option<RbumRelBoneResp>,
    pub rel_bs: Option<PluginBsInfoResp>,
}
