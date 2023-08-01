use std::collections::HashMap;

use bios_basic::dto::BasicQueryCondInfo;
use serde::{Deserialize, Serialize};
use tardis::{
    basic::field::TrimString,
    chrono::{DateTime, Utc},
    serde_json::{self, Value},
    web::poem_openapi,
    TardisFuns,
};

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
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
    #[oai(validator(min_length = "2"))]
    pub owner: Option<String>,
    #[oai(validator(min_length = "2"))]
    pub own_paths: Option<String>,
    pub create_time: Option<DateTime<Utc>>,
    pub update_time: Option<DateTime<Utc>>,
    pub ext: Option<Value>,
    pub visit_keys: Option<SearchItemVisitKeysReq>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct SearchItemModifyReq {
    #[oai(validator(min_length = "2"))]
    pub kind: Option<String>,
    #[oai(validator(min_length = "2"))]
    pub title: Option<String>,
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
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Clone)]
pub struct SearchItemVisitKeysReq {
    pub accounts: Option<Vec<String>>,
    pub apps: Option<Vec<String>>,
    pub tenants: Option<Vec<String>>,
    pub roles: Option<Vec<String>>,
    pub groups: Option<Vec<String>>,
}

impl SearchItemVisitKeysReq {
    pub fn to_sql(&self) -> serde_json::Value {
        let mut sqls = HashMap::new();
        if let Some(accounts) = &self.accounts {
            sqls.insert("accounts".to_string(), accounts.clone());
        } else {
            sqls.insert("accounts".to_string(), vec![]);
        }
        if let Some(apps) = &self.apps {
            sqls.insert("apps".to_string(), apps.clone());
        } else {
            sqls.insert("apps".to_string(), vec![]);
        }
        if let Some(tenants) = &self.tenants {
            sqls.insert("tenants".to_string(), tenants.clone());
        } else {
            sqls.insert("tenants".to_string(), vec![]);
        }
        if let Some(roles) = &self.roles {
            sqls.insert("roles".to_string(), roles.clone());
        } else {
            sqls.insert("roles".to_string(), vec![]);
        }
        if let Some(groups) = &self.groups {
            sqls.insert("groups".to_string(), groups.clone());
        } else {
            sqls.insert("groups".to_string(), vec![]);
        }
        TardisFuns::json.obj_to_json(&sqls).expect("it's impossible to fail here, since sql has type HashMap<String, Vec<String>>")
    }
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct SearchItemSearchReq {
    #[oai(validator(pattern = r"^[a-z0-9-_]+$"))]
    pub tag: String,
    // Search context for record permission filtering
    pub ctx: SearchItemSearchCtxReq,
    // Search conditions
    pub query: SearchItemQueryReq,
    // Sort
    // When the record set is very large, it will seriously affect the performance, it is not recommended to use.
    pub sort: Option<Vec<SearchItemSearchSortReq>>,
    pub page: SearchItemSearchPageReq,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct SearchItemSearchCtxReq {
    pub accounts: Option<Vec<String>>,
    pub apps: Option<Vec<String>>,
    pub tenants: Option<Vec<String>>,
    pub roles: Option<Vec<String>>,
    pub groups: Option<Vec<String>>,
    pub cond_by_or: Option<bool>,
}

impl SearchItemSearchCtxReq {
    pub fn to_sql(&self) -> HashMap<&str, Vec<String>> {
        let mut sqls = HashMap::new();
        if let Some(accounts) = &self.accounts {
            sqls.insert("accounts", accounts.clone());
        }
        if let Some(apps) = &self.apps {
            sqls.insert("apps", apps.clone());
        }
        if let Some(tenants) = &self.tenants {
            sqls.insert("tenants", tenants.clone());
        }
        if let Some(roles) = &self.roles {
            sqls.insert("roles", roles.clone());
        }
        if let Some(groups) = &self.groups {
            sqls.insert("groups", groups.clone());
        }
        sqls
    }
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Default)]
pub struct SearchItemQueryReq {
    // Fuzzy search content
    pub q: Option<String>,
    // Fuzzy search scope
    pub q_scope: Option<SearchItemSearchQScopeKind>,
    pub kinds: Option<Vec<String>>,
    // Match keys, support prefix match
    pub keys: Option<Vec<TrimString>>,
    #[oai(validator(min_length = "2"))]
    // Match owners, support prefix match
    pub owners: Option<Vec<String>>,
    // Match own_path, support prefix match
    pub own_paths: Option<Vec<String>>,
    pub create_time_start: Option<DateTime<Utc>>,
    pub create_time_end: Option<DateTime<Utc>>,
    pub update_time_start: Option<DateTime<Utc>>,
    pub update_time_end: Option<DateTime<Utc>>,
    // Extended filtering conditions
    pub ext: Option<Vec<BasicQueryCondInfo>>,
}

#[derive(poem_openapi::Enum, Serialize, Deserialize, Debug)]
pub enum SearchItemSearchQScopeKind {
    #[oai(rename = "title")]
    Title,
    #[oai(rename = "content")]
    Content,
    #[oai(rename = "title_content")]
    TitleContent,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct SearchItemSearchSortReq {
    #[oai(validator(min_length = "2"))]
    pub field: String,
    pub order: SearchItemSearchSortKind,
}

#[derive(poem_openapi::Enum, Serialize, Deserialize, Debug)]
pub enum SearchItemSearchSortKind {
    #[oai(rename = "asc")]
    Asc,
    #[oai(rename = "desc")]
    Desc,
}

impl SearchItemSearchSortKind {
    pub fn to_sql(&self) -> String {
        match self {
            SearchItemSearchSortKind::Asc => "ASC".to_string(),
            SearchItemSearchSortKind::Desc => "DESC".to_string(),
        }
    }
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct SearchItemSearchPageReq {
    pub number: u32,
    pub size: u16,
    // Get the total number of matching records.
    // When the record set is very large, it will seriously affect the performance. It is not recommended to open it.
    pub fetch_total: bool,
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
    pub owner: String,
    #[oai(validator(min_length = "2"))]
    pub own_paths: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
    pub ext: Value,
    pub rank_title: f32,
    pub rank_content: f32,
}
