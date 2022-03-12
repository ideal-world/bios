use serde::{Deserialize, Serialize};
use tardis::chrono::{DateTime, Utc};
use tardis::db::sea_orm::*;
use tardis::web::poem_openapi::Object;

use crate::rbum::enumeration::RbumCertStatusKind;

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct RbumCertAddReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub name: String,
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub ak: Option<String>,
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub sk: Option<String>,
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub ext: Option<String>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub coexist_flag: Option<String>,
    pub status: RbumCertStatusKind,
}

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct RbumCertModifyReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub name: Option<String>,
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub ak: Option<String>,
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub sk: Option<String>,
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub ext: Option<String>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub coexist_flag: Option<String>,
    pub status: Option<RbumCertStatusKind>,
}

#[derive(Object, FromQueryResult, Serialize, Deserialize, Debug)]
pub struct RbumCertSummaryResp {
    pub id: String,
    pub name: String,

    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
}

#[derive(Object, FromQueryResult, Serialize, Deserialize, Debug)]
pub struct RbumCertDetailResp {
    pub id: String,
    pub name: String,
    pub ak: String,
    pub sk: String,
    pub ext: String,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub coexist_flag: String,
    pub status: RbumCertStatusKind,
    pub rel_rbum_cert_conf_id: String,
    pub rel_rbum_cert_conf_name: String,
    pub rel_rbum_domain_id: String,
    pub rel_rbum_domain_name: String,
    pub rel_rbum_item_id: String,
    pub rel_rbum_item_name: String,

    pub rel_app_id: String,
    pub rel_app_name: String,
    pub rel_tenant_id: String,
    pub rel_tenant_name: String,
    pub updater_id: String,
    pub updater_name: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
}
