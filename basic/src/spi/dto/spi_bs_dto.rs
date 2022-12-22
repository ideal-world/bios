use serde::{Deserialize, Serialize};
use tardis::basic::field::TrimString;
use tardis::chrono::{DateTime, Utc};
use tardis::db::sea_orm;
use tardis::web::poem_openapi;

use crate::rbum::dto::rbum_filer_dto::{RbumBasicFilterReq, RbumItemFilterFetcher, RbumItemRelFilterReq};

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object))]
pub struct SpiBsAddReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub name: TrimString,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub kind_id: TrimString,
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub conn_uri: String,
    #[oai(validator(min_length = "2"))]
    pub ak: TrimString,
    #[oai(validator(min_length = "2"))]
    pub sk: TrimString,
    pub ext: String,
    pub private: bool,

    pub disabled: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object))]
pub struct SpiBsModifyReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub name: Option<TrimString>,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub kind_id: Option<TrimString>,
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub conn_uri: Option<String>,
    #[oai(validator(min_length = "2"))]
    pub ak: Option<TrimString>,
    #[oai(validator(min_length = "2"))]
    pub sk: Option<TrimString>,
    pub ext: Option<String>,
    pub private: Option<bool>,

    pub disabled: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object, sea_orm::FromQueryResult))]
pub struct SpiBsSummaryResp {
    pub id: String,
    pub name: String,
    pub kind_id: String,
    pub kind_code: String,
    pub kind_name: String,
    pub conn_uri: String,
    pub ak: String,
    pub sk: String,
    pub ext: String,
    pub private: bool,
    pub disabled: bool,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object, sea_orm::FromQueryResult))]
pub struct SpiBsDetailResp {
    pub id: String,
    pub name: String,
    pub kind_id: String,
    pub kind_code: String,
    pub kind_name: String,
    pub conn_uri: String,
    pub ak: String,
    pub sk: String,
    pub ext: String,
    pub private: bool,
    pub disabled: bool,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
    pub rel_app_tenant_ids: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SpiBsCertResp {
    pub kind_code: String,
    pub conn_uri: String,
    pub ak: String,
    pub sk: String,
    pub ext: String,
    pub private: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object))]
#[serde(default)]
pub struct SpiBsFilterReq {
    pub basic: RbumBasicFilterReq,
    pub rel: Option<RbumItemRelFilterReq>,
    pub rel2: Option<RbumItemRelFilterReq>,
    pub private: Option<bool>,
    pub domain_code: Option<String>,
}

impl RbumItemFilterFetcher for SpiBsFilterReq {
    fn basic(&self) -> &RbumBasicFilterReq {
        &self.basic
    }
    fn rel(&self) -> &Option<RbumItemRelFilterReq> {
        &self.rel
    }
    fn rel2(&self) -> &Option<RbumItemRelFilterReq> {
        &self.rel2
    }
}
