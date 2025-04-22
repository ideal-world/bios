use std::collections::HashMap;

use bios_basic::rbum::dto::rbum_cert_dto::RbumCertSummaryWithSkResp;
use bios_basic::rbum::dto::rbum_set_cate_dto::RbumSetCateDetailResp;
use bios_basic::rbum::dto::rbum_set_dto::{RbumSetDetailResp, RbumSetTreeResp};
use bios_basic::rbum::dto::rbum_set_item_dto::RbumSetItemDetailResp;
use bios_basic::rbum::rbum_enumeration::RbumScopeLevelKind;
use serde::{Deserialize, Serialize};

use tardis::basic::field::TrimString;
use tardis::chrono::{DateTime, Utc};
use tardis::db::sea_orm;
use tardis::web::poem_openapi;

use super::iam_account_dto::IamAccountDetailResp;
use super::iam_app_dto::IamAppDetailResp;
use super::iam_config_dto::IamConfigDetailResp;
use super::iam_role_dto::IamRoleDetailResp;

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object))]
pub struct IamSubDeployAddReq {
    pub name: Option<TrimString>,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub province: String,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub access_url: Option<String>,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub note: Option<String>,
    pub extend_sub_deploy_id: Option<String>,
    pub scope_level: Option<RbumScopeLevelKind>,
}

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object))]
pub struct IamSubDeployModifyReq {
    pub name: Option<TrimString>,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub province: Option<String>,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub access_url: Option<String>,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub note: Option<String>,
    pub scope_level: Option<RbumScopeLevelKind>,
    pub disabled: Option<bool>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object, sea_orm::FromQueryResult))]
pub struct IamSubDeploySummaryResp {
    pub id: String,
    pub code: String,
    pub name: String,
    pub province: String,
    pub access_url: String,
    pub note: String,
    pub disabled: bool,
    pub extend_sub_deploy_id: String,

    pub own_paths: String,
    pub owner: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
    pub create_by: String,
    pub update_by: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object, sea_orm::FromQueryResult))]
pub struct IamSubDeployDetailResp {
    pub id: String,
    pub code: String,
    pub name: String,
    pub province: String,
    pub access_url: String,
    pub note: String,
    pub disabled: bool,
    pub extend_sub_deploy_id: String,

    pub own_paths: String,
    pub owner: String,
    pub owner_name: Option<String>,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
    pub create_by: String,
    pub update_by: String,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamSubDeployOneExportAggResp {
    pub accounts: Option<Vec<IamAccountDetailResp>>,
    pub account_cert: Option<HashMap<String, Vec<RbumCertSummaryWithSkResp>>>,
    pub account_role: Option<HashMap<String, Vec<String>>>,
    pub account_org: Option<HashMap<String, Vec<String>>>,
    pub account_apps: Option<HashMap<String, Vec<String>>>,
    pub org_set: Option<RbumSetDetailResp>,
    pub org_set_cate: Option<Vec<RbumSetCateDetailResp>>,
    pub apps_set: Option<RbumSetDetailResp>,
    pub apps_set_cate: Option<Vec<RbumSetCateDetailResp>>,
    pub iam_config: Option<HashMap<String, Vec<IamConfigDetailResp>>>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamSubDeployOneImportReq {
    pub app: Option<Vec<IamAppDetailResp>>,
    pub app_apps: Option<HashMap<String, Vec<String>>>,
    pub app_role: Option<HashMap<String, Vec<IamRoleDetailResp>>>,
    pub app_account: Option<HashMap<String, Vec<String>>>,
    pub app_role_account: Option<HashMap<String, Vec<String>>>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamSubDeployTowExportAggResp {
    pub app: Option<Vec<IamAppDetailResp>>,
    pub app_apps: Option<HashMap<String, Vec<String>>>,
    pub app_role: Option<HashMap<String, Vec<IamRoleDetailResp>>>,
    pub app_account: Option<HashMap<String, Vec<String>>>,
    pub app_role_account: Option<HashMap<String, Vec<String>>>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamSubDeployTowImportReq {
    pub accounts: Option<Vec<IamAccountDetailResp>>,
    pub account_cert: Option<HashMap<String, Vec<RbumCertSummaryWithSkResp>>>,
    pub account_role: Option<HashMap<String, Vec<String>>>,
    pub account_org: Option<HashMap<String, Vec<String>>>,
    pub account_apps: Option<HashMap<String, Vec<String>>>,
    pub org_set: Option<RbumSetDetailResp>,
    pub org_set_cate: Option<Vec<RbumSetCateDetailResp>>,
    pub apps_set: Option<RbumSetDetailResp>,
    pub apps_set_cate: Option<Vec<RbumSetCateDetailResp>>,
    pub iam_config: Option<HashMap<String, Vec<IamConfigDetailResp>>>,
}
