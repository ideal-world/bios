use serde::{Deserialize, Serialize};
use tardis::basic::field::TrimString;
use tardis::chrono::{DateTime, Utc};
#[cfg(feature = "default")]
use tardis::db::sea_orm;
#[cfg(feature = "default")]
use tardis::web::poem_openapi;

use crate::rbum::rbum_enumeration::{RbumCertRelKind, RbumCertStatusKind};

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object))]
pub struct RbumCertAddReq {
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "2000")))]
    pub ak: TrimString,
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "2000")))]
    pub sk: Option<TrimString>,
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "2000")))]
    pub vcode: Option<TrimString>,
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "2000")))]
    pub ext: Option<String>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "2000")))]
    pub conn_uri: Option<String>,
    pub status: RbumCertStatusKind,

    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "255")))]
    pub rel_rbum_cert_conf_id: Option<String>,
    pub rel_rbum_kind: RbumCertRelKind,
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "255")))]
    pub rel_rbum_id: String,
    pub is_outside: bool,
}

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object))]
pub struct RbumCertModifyReq {
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "2000")))]
    pub ext: Option<String>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "2000")))]
    pub conn_uri: Option<String>,
    pub status: Option<RbumCertStatusKind>,
}

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object, sea_orm::FromQueryResult))]
pub struct RbumCertSummaryResp {
    pub id: String,
    pub ak: String,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub status: RbumCertStatusKind,

    pub rel_rbum_cert_conf_id: Option<String>,
    pub rel_rbum_cert_conf_name: Option<String>,
    pub rel_rbum_cert_conf_code: Option<String>,
    pub rel_rbum_kind: RbumCertRelKind,
    pub rel_rbum_id: String,

    pub own_paths: String,
    pub owner: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object, sea_orm::FromQueryResult))]
pub struct RbumCertSummaryWithSkResp {
    pub id: String,
    pub ak: String,
    pub sk: String,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub status: RbumCertStatusKind,

    pub rel_rbum_cert_conf_id: Option<String>,
    pub rel_rbum_cert_conf_name: Option<String>,
    pub rel_rbum_cert_conf_code: Option<String>,
    pub rel_rbum_kind: RbumCertRelKind,
    pub rel_rbum_id: String,

    pub own_paths: String,
    pub owner: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object, sea_orm::FromQueryResult))]
pub struct RbumCertDetailResp {
    pub id: String,
    pub ak: String,
    pub ext: String,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub conn_uri: String,
    pub status: RbumCertStatusKind,
    pub rel_rbum_cert_conf_id: Option<String>,
    pub rel_rbum_cert_conf_name: Option<String>,
    pub rel_rbum_cert_conf_code: Option<String>,
    pub rel_rbum_kind: RbumCertRelKind,
    pub rel_rbum_id: String,

    pub own_paths: String,
    pub owner: String,
    pub owner_name: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
}
