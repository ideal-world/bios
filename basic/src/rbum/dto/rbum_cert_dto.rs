use serde::{Deserialize, Serialize};
use tardis::basic::field::TrimString;
use tardis::chrono::{DateTime, Utc};
use tardis::web::poem_openapi::Object;

use crate::rbum::enumeration::RbumCertStatusKind;

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct RbumCertAddReq {
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub ak: TrimString,
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub sk: Option<TrimString>,
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub ext: Option<String>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub coexist_flag: Option<String>,
    pub status: RbumCertStatusKind,

    #[oai(validator(min_length = "2", max_length = "255"))]
    pub rel_rbum_cert_conf_id: String,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub rel_rbum_item_id: Option<String>,
}

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct RbumCertModifyReq {
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub ext: Option<String>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub coexist_flag: Option<String>,
    pub status: Option<RbumCertStatusKind>,
}

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(tardis::web::poem_openapi::Object, tardis::db::sea_orm::FromQueryResult))]
pub struct RbumCertSummaryResp {
    pub id: String,
    pub ak: String,
    pub rel_rbum_cert_conf_id: String,
    pub rel_rbum_cert_conf_name: String,
    pub rel_rbum_item_id: Option<String>,
    pub rel_rbum_item_name: Option<String>,

    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(tardis::web::poem_openapi::Object, tardis::db::sea_orm::FromQueryResult))]
pub struct RbumCertDetailResp {
    pub id: String,
    pub ak: String,
    pub ext: String,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub coexist_flag: String,
    pub status: String,
    pub rel_rbum_cert_conf_id: String,
    pub rel_rbum_cert_conf_name: String,
    pub rel_rbum_item_id: Option<String>,
    pub rel_rbum_item_name: Option<String>,

    pub scope_ids: String,
    pub updater_id: String,
    pub updater_name: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
}
