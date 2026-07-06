use std::collections::HashMap;

use bios_basic::rbum::rbum_enumeration::RbumScopeLevelKind;
use serde::{Deserialize, Serialize};
use tardis::basic::field::TrimString;
use tardis::chrono::{DateTime, Utc};
use tardis::db::sea_orm::{DbErr, FromQueryResult, QueryResult};
use tardis::web::poem_openapi;

/// 发布系统新增请求
#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamPublishSystemAddReq {
    #[oai(skip = true)]
    pub id: Option<TrimString>,
    /// 系统名称
    #[oai(validator(min_length = "1", max_length = "50"))]
    pub name: TrimString,
    /// 系统标识
    #[oai(validator(max_length = "50"))]
    pub sys_ident: Option<String>,
    /// 所属分公司（租户 ID 列表）
    pub rel_tenant_ids: Vec<TrimString>,
    pub scope_level: Option<RbumScopeLevelKind>,
    /// 描述
    #[oai(validator(max_length = "500"))]
    pub description: Option<String>,
}

/// 发布系统修改请求
#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamPublishSystemModifyReq {
    /// 系统名称
    #[oai(validator(min_length = "1", max_length = "50"))]
    pub name: Option<TrimString>,
    /// 系统标识
    #[oai(validator(max_length = "50"))]
    pub sys_ident: Option<String>,
    /// 所属分公司（租户 ID 列表）
    pub rel_tenant_ids: Option<Vec<TrimString>>,
    pub scope_level: Option<RbumScopeLevelKind>,
    /// 描述
    #[oai(validator(max_length = "500"))]
    pub description: Option<String>,
}

/// 发布系统概要响应
#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamPublishSystemSummaryResp {
    pub id: String,
    pub name: String,
    pub sys_ident: Option<String>,
    pub rel_tenant_ids: Vec<String>,
    pub scope_level: RbumScopeLevelKind,
    pub description: Option<String>,
    pub own_paths: String,
    pub owner: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
}

impl FromQueryResult for IamPublishSystemSummaryResp {
    fn from_query_result(res: &QueryResult, pre: &str) -> Result<Self, DbErr> {
        Ok(Self {
            id: res.try_get(pre, "id")?,
            name: res.try_get(pre, "name")?,
            sys_ident: res.try_get(pre, "sys_ident").ok(),
            rel_tenant_ids: vec![],
            scope_level: res.try_get(pre, "scope_level")?,
            description: res.try_get(pre, "description").ok(),
            own_paths: res.try_get(pre, "own_paths")?,
            owner: res.try_get(pre, "owner")?,
            create_time: res.try_get(pre, "create_time")?,
            update_time: res.try_get(pre, "update_time")?,
        })
    }
}

/// 发布系统详情响应
#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Clone)]
pub struct IamPublishSystemDetailResp {
    pub id: String,
    pub name: String,
    pub sys_ident: Option<String>,
    pub rel_tenant_ids: Vec<String>,
    pub scope_level: RbumScopeLevelKind,
    pub description: Option<String>,
    pub own_paths: String,
    pub owner: String,
    pub owner_name: Option<String>,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
}

impl FromQueryResult for IamPublishSystemDetailResp {
    fn from_query_result(res: &QueryResult, pre: &str) -> Result<Self, DbErr> {
        Ok(Self {
            id: res.try_get(pre, "id")?,
            name: res.try_get(pre, "name")?,
            sys_ident: res.try_get(pre, "sys_ident").ok(),
            rel_tenant_ids: vec![],
            scope_level: res.try_get(pre, "scope_level")?,
            description: res.try_get(pre, "description").ok(),
            own_paths: res.try_get(pre, "own_paths")?,
            owner: res.try_get(pre, "owner")?,
            owner_name: res.try_get(pre, "owner_name").ok(),
            create_time: res.try_get(pre, "create_time")?,
            update_time: res.try_get(pre, "update_time")?,
        })
    }
}

/// 发布系统删除响应
#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamPublishSystemDeleteResp {
    /// 是否已删除
    pub deleted: bool,
    /// 关联产品 ID，key 为租户 ID，value 为该租户下的产品 ID 列表（存在关联产品时无法删除）
    pub rel_app_ids: HashMap<String, Vec<String>>,
}
