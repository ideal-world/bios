use serde::{Deserialize, Serialize};
use tardis::web::poem_openapi::Object;

use crate::rbum::dto::rbum_item_dto::{RbumItemAddReq, RbumItemDetailResp, RbumItemModifyReq, RbumItemSummaryResp};

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct IamCsTenantAddReq {
    pub basic: RbumItemAddReq,
}

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct IamCsTenantModifyReq {
    pub basic: RbumItemModifyReq,
}

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct IamCsTenantSummaryResp {
    pub basic: RbumItemSummaryResp,
}

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct IamCsTenantDetailResp {
    pub basic: RbumItemDetailResp,
}
