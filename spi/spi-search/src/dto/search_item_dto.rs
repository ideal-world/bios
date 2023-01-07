use bios_basic::spi::dto::spi_basic_dto::SpiQueryCondReq;
use serde::{Deserialize, Serialize};
use tardis::{
    chrono::{DateTime, Utc},
    serde_json::Value,
    web::poem_openapi, basic::field::TrimString,
};

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct SearchItemAddOrModifyReq {
    #[oai(validator(pattern = r"^[a-z0-9]+$"))]
    pub tag: String,
    #[oai(validator(min_length = "2"))]
    pub key: TrimString,
    #[oai(validator(min_length = "2"))]
    pub title: String,
    #[oai(validator(min_length = "2"))]
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
pub struct SearchItemVisitKeysReq {
    pub accounts: Option<Vec<String>>,
    pub apps: Option<Vec<String>>,
    pub tenants: Option<Vec<String>>,
    pub roles: Option<Vec<String>>,
    pub groups: Option<Vec<String>>,
}

impl SearchItemVisitKeysReq {
    pub fn to_sql(&self) -> Vec<String> {
        let mut sqls = Vec::new();
        if let Some(accounts) = &self.accounts {
            for account in accounts {
                sqls.push(format!("ac:{}", account));
            }
        }
        if let Some(apps) = &self.apps {
            for app in apps {
                sqls.push(format!("ap:{}", app));
            }
        }
        if let Some(tenants) = &self.tenants {
            for tenant in tenants {
                sqls.push(format!("te:{}", tenant));
            }
        }
        if let Some(roles) = &self.roles {
            for role in roles {
                sqls.push(format!("ro:{}", role));
            }
        }
        if let Some(groups) = &self.groups {
            for group in groups {
                sqls.push(format!("gr:{}", group));
            }
        }
        sqls
    }
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct SearchItemSearchReq {
    #[oai(validator(pattern = r"^[a-z0-9]+$"))]
    pub tag: String,
    pub ctx: SearchItemSearchCtxReq,
    pub query: SearchItemQueryReq,
    pub sort: Option<Vec<SearchItemSearchSortReq>>,
    pub page: SearchItemSearchPageReq,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct SearchItemSearchCtxReq {
    #[oai(validator(min_length = "2"))]
    pub account: Option<String>,
    #[oai(validator(min_length = "2"))]
    pub app: Option<String>,
    #[oai(validator(min_length = "2"))]
    pub tenant: Option<String>,
    pub roles: Option<Vec<String>>,
    pub groups: Option<Vec<String>>,
}

impl SearchItemSearchCtxReq {
    pub fn to_sql(&self) -> Vec<String> {
        let mut sqls = Vec::new();
        if let Some(account) = &self.account {
            sqls.push(format!("ac:{}", account));
        }
        if let Some(app) = &self.app {
            sqls.push(format!("ap:{}", app));
        }
        if let Some(tenant) = &self.tenant {
            sqls.push(format!("te:{}", tenant));
        }
        if let Some(roles) = &self.roles {
            for role in roles {
                sqls.push(format!("ro:{}", role));
            }
        }
        if let Some(groups) = &self.groups {
            for group in groups {
                sqls.push(format!("gr:{}", group));
            }
        }
        sqls
    }
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct SearchItemQueryReq {
    #[oai(validator(min_length = "2"))]
    pub q: Option<String>,
    #[oai(validator(min_length = "2"))]
    pub key: Option<TrimString>,
    #[oai(validator(min_length = "2"))]
    pub owner: Option<String>,
    #[oai(validator(min_length = "2"))]
    pub own_paths: Option<String>,
    pub create_time_start: Option<DateTime<Utc>>,
    pub create_time_end: Option<DateTime<Utc>>,
    pub update_time_start: Option<DateTime<Utc>>,
    pub update_time_end: Option<DateTime<Utc>>,
    pub ext: Option<Vec<SpiQueryCondReq>>,
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
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct SearchItemSearchResp {
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
