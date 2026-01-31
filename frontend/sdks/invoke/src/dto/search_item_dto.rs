use serde::{Deserialize, Serialize};
use tardis::{
    basic::field::TrimString,
    chrono::{DateTime, Utc},
    serde_json::Value,
    web::poem_openapi,
};

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Clone)]
pub struct SearchItemAddReq {
    #[oai(validator(pattern = r"^[a-z0-9-_]+$"))]
    pub tag: String,
    #[oai(validator(min_length = "2"))]
    pub kind: String,
    #[oai(validator(min_length = "2"))]
    pub key: TrimString,
    #[oai(validator(min_length = "2"))]
    pub title: String,
    // #[oai(validator(min_length = "2"))]
    pub content: String,
    pub data_source: Option<String>,
    #[oai(validator(min_length = "2"))]
    pub owner: Option<String>,
    #[oai(validator(min_length = "2"))]
    pub own_paths: Option<String>,
    pub create_time: Option<DateTime<Utc>>,
    pub update_time: Option<DateTime<Utc>>,
    pub ext: Option<Value>,
    pub visit_keys: Option<SearchItemVisitKeysReq>,
    pub kv_disable: Option<bool>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Clone)]
pub struct SearchItemModifyReq {
    #[oai(validator(min_length = "2"))]
    pub kind: Option<String>,
    #[oai(validator(min_length = "2"))]
    pub title: Option<String>,
    #[oai(validator(min_length = "2"))]
    pub name: Option<String>,
    // #[oai(validator(min_length = "2"))]
    pub content: Option<String>,
    #[oai(validator(min_length = "2"))]
    pub owner: Option<String>,
    // #[oai(validator(min_length = "2"))]
    pub own_paths: Option<String>,
    pub create_time: Option<DateTime<Utc>>,
    pub update_time: Option<DateTime<Utc>>,
    pub ext: Option<Value>,
    // Overwrites the original content when it is true
    pub ext_override: Option<bool>,
    pub visit_keys: Option<SearchItemVisitKeysReq>,
    pub kv_disable: Option<bool>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Clone)]
pub struct SearchItemVisitKeysReq {
    pub accounts: Option<Vec<String>>,
    pub apps: Option<Vec<String>>,
    pub tenants: Option<Vec<String>>,
    pub roles: Option<Vec<String>>,
    pub groups: Option<Vec<String>>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct SearchItemSearchReq {
    #[oai(validator(pattern = r"^[a-z0-9-_]+$"))]
    pub tag: String,
    // Search context for record permission filtering
    pub ctx: SearchItemSearchCtxReq,
    // Search conditions
    pub query: SearchItemQueryReq,
    // Advanced search
    pub adv_by_or: Option<bool>,
    pub adv_query: Option<Vec<AdvSearchItemQueryReq>>,
    // Sort
    // When the record set is very large, it will seriously affect the performance, it is not recommended to use.
    pub sort: Option<Vec<SearchItemSearchSortReq>>,
    pub page: SearchItemSearchPageReq,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Default, Clone)]
pub struct SearchItemSearchCtxReq {
    pub accounts: Option<Vec<String>>,
    pub apps: Option<Vec<String>>,
    pub tenants: Option<Vec<String>>,
    pub roles: Option<Vec<String>>,
    pub groups: Option<Vec<String>>,
    pub cond_by_or: Option<bool>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Default, Clone)]
pub struct SearchItemQueryReq {
    pub in_q_content: Option<bool>,
    // Fuzzy search content
    pub q: Option<String>,
    // Fuzzy search scope
    pub q_scope: Option<SearchItemSearchQScopeKind>,
    pub kinds: Option<Vec<String>>,
    // Match keys, supports prefix match
    pub keys: Option<Vec<TrimString>>,
    #[oai(validator(min_length = "2"))]
    // Match owners, supports prefix match
    pub owners: Option<Vec<String>>,
    // Match own_path, supports prefix match
    pub own_paths: Option<Vec<String>>,
    pub rlike_own_paths: Option<Vec<String>>,
    pub create_time_start: Option<DateTime<Utc>>,
    pub create_time_end: Option<DateTime<Utc>>,
    pub update_time_start: Option<DateTime<Utc>>,
    pub update_time_end: Option<DateTime<Utc>>,
    // Extended filtering conditions
    pub ext: Option<Vec<BasicQueryCondInfo>>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Clone)]
pub struct SearchItemSearchPageReq {
    pub number: u32,
    pub size: u16,
    // Get the total number of matching records.
    // When the record set is very large, it will seriously affect the performance. It is not recommended to open it.
    pub fetch_total: bool,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Clone)]
pub struct SearchItemSearchSortReq {
    #[oai(validator(min_length = "2"))]
    pub field: String,
    pub order: SearchItemSearchSortKind,
}

#[derive(poem_openapi::Enum, Serialize, Deserialize, Debug, Clone)]
pub enum SearchItemSearchSortKind {
    #[serde(rename = "asc")]
    #[oai(rename = "asc")]
    Asc,
    #[serde(rename = "desc")]
    #[oai(rename = "desc")]
    Desc,
}

#[derive(poem_openapi::Enum, Serialize, Deserialize, Debug, Clone)]
pub enum SearchItemSearchQScopeKind {
    #[oai(rename = "title")]
    Title,
    #[oai(rename = "content")]
    Content,
    #[oai(rename = "title_content")]
    TitleContent,
}

/// Basic query condition object
///
/// 基础的查询条件对象
#[derive(Serialize, Deserialize, Debug, Clone, poem_openapi::Object)]
pub struct BasicQueryCondInfo {
    /// Query field
    #[oai(validator(min_length = "1"))]
    pub field: String,
    /// Query operator
    pub op: BasicQueryOpKind,
    /// Query value
    pub value: Value,
}

/// Basic query operator
///
/// 基础查询操作符
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, poem_openapi::Enum)]
#[serde(rename_all = "snake_case")]
pub enum BasicQueryOpKind {
    #[oai(rename = "=")]
    Eq,
    #[oai(rename = "!=")]
    Ne,
    #[oai(rename = ">")]
    Gt,
    #[oai(rename = ">=")]
    Ge,
    #[oai(rename = "<")]
    Lt,
    #[oai(rename = "<=")]
    Le,
    #[oai(rename = "like")]
    Like,
    #[oai(rename = "not_like")]
    NotLike,
    #[oai(rename = "l_like")]
    LLike,
    #[oai(rename = "not_l_like")]
    NotLLike,
    #[oai(rename = "r_like")]
    RLike,
    #[oai(rename = "not_r_like")]
    NotRLike,
    #[oai(rename = "in")]
    In,
    #[oai(rename = "not_in")]
    NotIn,
    #[oai(rename = "is_null")]
    IsNull,
    #[oai(rename = "is_not_null")]
    IsNotNull,
    #[oai(rename = "is_null_or_empty")]
    IsNullOrEmpty,
    #[oai(rename = "length")]
    Len,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Default, Clone)]
pub struct AdvSearchItemQueryReq {
    pub group_by_or: Option<bool>,
    pub ext_by_or: Option<bool>,
    // Extended filtering conditions
    pub ext: Option<Vec<BasicQueryCondInfo>>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct SearchItemSearchResp {
    #[oai(validator(min_length = "2"))]
    pub kind: String,
    #[oai(validator(min_length = "2"))]
    pub key: String,
    #[oai(validator(min_length = "2"))]
    pub title: String,
    #[oai(validator(min_length = "2"))]
    pub content: String,
    #[oai(validator(min_length = "2"))]
    pub owner: String,
    #[oai(validator(min_length = "2"))]
    pub own_paths: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
    pub ext: Value,
    pub rank_title: f32,
    pub rank_content: f32,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Clone)]
pub struct SearchSaveItemReq {
    #[oai(validator(min_length = "2"))]
    pub kind: Option<String>,
    #[oai(validator(min_length = "2"))]
    pub key: TrimString,
    #[oai(validator(min_length = "1"))]
    pub title: Option<String>,
    // #[oai(validator(min_length = "2"))]
    pub content: Option<String>,
    pub data_source: Option<String>,
    #[oai(validator(min_length = "2"))]
    pub owner: Option<String>,
    // #[oai(validator(min_length = "2"))]
    pub own_paths: Option<String>,
    pub create_time: Option<DateTime<Utc>>,
    pub update_time: Option<DateTime<Utc>>,
    pub ext: Option<Value>,
    pub visit_keys: Option<SearchItemVisitKeysReq>,
}
