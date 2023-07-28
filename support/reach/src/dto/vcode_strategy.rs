use bios_basic::rbum::dto::{rbum_filer_dto::RbumItemBasicFilterReq, rbum_item_dto::RbumItemAddReq};
use serde::Serialize;
use tardis::{
    chrono::{DateTime, Utc},
    db::sea_orm,
    web::poem_openapi,
};


#[derive(Debug, poem_openapi::Object)]
pub struct ReachVCodeStrategyAddReq {
    #[oai(flatten)]
    pub rbum_item_add_req: RbumItemAddReq,
    pub max_error_times: i32,
    pub expire_sec: i32,
    pub length: i32,
    pub rel_reach_set_id: String,
}

#[derive(Debug, poem_openapi::Object)]
pub struct ReachVCodeStrategyModifyReq {
    pub max_error_times: i32,
    pub expire_sec: i32,
    pub length: i32,
}

#[derive(Debug, poem_openapi::Object, Default)]
pub struct ReachVCodeStrategyFilterReq {
    #[oai(flatten)]
    pub base_filter: RbumItemBasicFilterReq,
    pub rel_reach_set_id: Option<String>,
}

#[derive(Debug, poem_openapi::Object, Serialize, sea_orm::FromQueryResult)]
pub struct ReachVCodeStrategySummaryResp {
    pub id: String,
    pub own_paths: String,
    pub owner: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
    pub max_error_times: i32,
    pub expire_sec: i32,
    pub length: i32,
    pub rel_reach_set_id: String,
}

#[derive(Debug, poem_openapi::Object, Serialize, sea_orm::FromQueryResult)]

pub struct ReachVCodeStrategyDetailResp {
    pub id: String,
    pub own_paths: String,
    pub owner: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
    pub owner_name: String,
    pub max_error_times: i32,
    pub expire_sec: i32,
    pub length: i32,
    pub rel_reach_set_id: String,
}
