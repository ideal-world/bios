use std::collections::HashMap;

use crate::search_enumeration::{SearchDataTypeKind, SearchQueryAggFunKind, SearchQueryExpandFunKind, SearchQueryTimeWindowKind};
use bios_basic::{dto::BasicQueryCondInfo, enumeration::BasicQueryOpKind};
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
    #[oai(validator(min_length = "1"))]
    pub title: String,
    // #[oai(validator(min_length = "2"))]
    pub content: String,
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

impl From<bios_sdk_invoke::dto::search_item_dto::SearchItemAddReq> for SearchItemAddReq {
    fn from(value: bios_sdk_invoke::dto::search_item_dto::SearchItemAddReq) -> Self {
        Self {
            tag: value.tag,
            kind: value.kind,
            key: value.key,
            title: value.title,
            content: value.content,
            data_source: value.data_source,
            owner: value.owner,
            own_paths: value.own_paths,
            create_time: value.create_time,
            update_time: value.update_time,
            ext: value.ext,
            visit_keys: value.visit_keys.map(Into::into),
        }
    }
}
#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Default)]
pub struct SearchItemModifyReq {
    #[oai(validator(min_length = "2"))]
    pub kind: Option<String>,
    #[oai(validator(min_length = "1"))]
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
impl From<bios_sdk_invoke::dto::search_item_dto::SearchItemModifyReq> for SearchItemModifyReq {
    fn from(value: bios_sdk_invoke::dto::search_item_dto::SearchItemModifyReq) -> Self {
        Self {
            kind: value.kind,
            title: value.title,
            content: value.content,
            owner: value.owner,
            own_paths: value.own_paths,
            create_time: value.create_time,
            update_time: value.update_time,
            ext: value.ext,
            ext_override: value.ext_override,
            visit_keys: value.visit_keys.map(Into::into),
        }
    }
}
#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Clone)]
pub struct SearchItemVisitKeysReq {
    pub accounts: Option<Vec<String>>,
    pub apps: Option<Vec<String>>,
    pub tenants: Option<Vec<String>>,
    pub roles: Option<Vec<String>>,
    pub groups: Option<Vec<String>>,
}
impl From<bios_sdk_invoke::dto::search_item_dto::SearchItemVisitKeysReq> for SearchItemVisitKeysReq {
    fn from(value: bios_sdk_invoke::dto::search_item_dto::SearchItemVisitKeysReq) -> Self {
        Self {
            accounts: value.accounts,
            apps: value.apps,
            tenants: value.tenants,
            roles: value.roles,
            groups: value.groups,
        }
    }
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
    // Advanced search
    pub adv_by_or: Option<bool>,
    pub adv_query: Option<Vec<AdvSearchItemQueryReq>>,
    // Sort
    // When the record set is very large, it will seriously affect the performance, it is not recommended to use.
    pub sort: Option<Vec<SearchItemSearchSortReq>>,
    pub page: SearchItemSearchPageReq,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct GroupSearchItemSearchReq {
    pub group_column: String,
    #[oai(validator(pattern = r"^[a-z0-9-_]+$"))]
    pub tag: String,
    // Search context for record permission filtering
    pub ctx: SearchItemSearchCtxReq,
    // Search conditions
    pub query: SearchItemQueryReq,
    // Advanced search
    pub adv_by_or: Option<bool>,
    pub adv_query: Option<Vec<AdvSearchItemQueryReq>>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct MultipleSearchItemSearchReq {
    #[oai(validator(pattern = r"^[a-z0-9-_]+$"))]
    pub tag: String,
    // Search context for record permission filtering
    pub ctx: SearchItemSearchCtxReq,
    // Search conditions
    pub query: SearchItemQueryReq,
    // Advanced search
    pub adv_query: Option<Vec<AdvSearchItemQueryReq>>,
    // Local cross join conditions
    pub local_cross_joins: Option<Vec<MultipleLocalCrossJoinColumnSearchItemSearchReq>>,
    /// Join conditions
    pub joins: Vec<MultipleJoinSearchItemSearchReq>,
    // Sort
    // When the record set is very large, it will seriously affect the performance, it is not recommended to use.
    pub sort: Option<Vec<SearchItemSearchSortReq>>,
    pub page: SearchItemSearchPageReq,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct MultipleJoinSearchItemSearchReq {
    #[oai(validator(pattern = r"^[a-z0-9-_]+$"))]
    pub tag: String,
    // Whether it is a inner join
    pub inner: Option<bool>,
    // Join columns
    pub join_columns: Vec<MultipleJoinColumnSearchItemSearchReq>,
    // Return columns
    pub return_columns: Option<Vec<MultipleJoinReturnColumnSearchItemSearchReq>>,
    // Advanced search
    pub adv_query: Option<Vec<AdvSearchItemQueryReq>>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct MultipleJoinColumnSearchItemSearchReq {
    pub is_cross_join: Option<bool>,
    pub on_local_field: String,
    pub on_foreign_field: String,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct MultipleLocalCrossJoinColumnSearchItemSearchReq {
    /// Expansion function
    pub fun: SearchQueryExpandFunKind,
    pub column: String,
    pub column_alias_name: Option<String>,
    pub params: Option<Vec<Value>>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct MultipleJoinReturnColumnSearchItemSearchReq {
    pub column: String,
    pub column_alias_name: Option<String>,
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

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Default, Clone)]
pub struct AdvSearchItemQueryReq {
    pub group_by_or: Option<bool>,
    pub ext_by_or: Option<bool>,
    // Extended filtering conditions
    pub ext: Option<Vec<BasicQueryCondInfo>>,
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

#[derive(poem_openapi::Enum, Serialize, Deserialize, Debug, Clone)]
pub enum SearchItemSearchQScopeKind {
    #[oai(rename = "title")]
    Title,
    #[oai(rename = "content")]
    Content,
    #[oai(rename = "title_content")]
    TitleContent,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Clone)]
pub struct SearchItemSearchSortReq {
    #[oai(validator(min_length = "2"))]
    pub field: String,
    pub order: SearchItemSearchSortKind,
}

#[derive(poem_openapi::Enum, Serialize, Deserialize, Debug, Clone)]
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

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Clone)]
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
    pub content: String,
    pub data_source: String,
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

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct GroupSearchItemSearchResp {
    pub group_column: Option<String>,
    pub count: i64,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct SearchQueryMetricsReq {
    pub tag: String,
    pub select: Vec<SearchQueryMetricsSelectReq>,
    pub ignore_distinct: Option<bool>,
    pub group: Vec<SearchQueryDimensionGroupReq>,
    pub ignore_group_rollup: Option<bool>,
    pub _where: Option<Vec<Vec<SearchQueryMetricsWhereReq>>>,
    pub dimension_order: Option<Vec<SearchQueryDimensionOrderReq>>,
    pub metrics_order: Option<Vec<SearchQueryMetricsOrderReq>>,
    pub group_order: Option<Vec<SearchQueryDimensionGroupOrderReq>>,
    pub group_agg: Option<bool>,
    pub having: Option<Vec<SearchQueryMetricsHavingReq>>,
    // Search context for record permission filtering
    pub ctx: SearchItemSearchCtxReq,
    // Search conditions
    pub query: SearchItemQueryReq,
    // Advanced search
    pub adv_query: Option<Vec<AdvSearchItemQueryReq>>,
    pub conf_limit: Option<u32>,
    pub limit: Option<u32>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct SearchQueryMetricsSelectReq {
    pub in_ext: Option<bool>,
    pub multi_values: Option<bool>,
    pub data_type: SearchDataTypeKind,
    /// Measure column key
    pub code: String,
    /// Aggregate function
    pub fun: SearchQueryAggFunKind,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct SearchQueryDimensionGroupReq {
    pub in_ext: Option<bool>,
    pub multi_values: Option<bool>,
    pub data_type: SearchDataTypeKind,
    /// Dimension column key
    pub code: String,
    /// Time window function
    pub time_window: Option<SearchQueryTimeWindowKind>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct SearchQueryMetricsWhereReq {
    pub in_ext: Option<bool>,
    pub multi_values: Option<bool>,
    pub data_type: SearchDataTypeKind,
    /// Dimension or measure column key
    pub code: String,
    /// Operator
    pub op: BasicQueryOpKind,
    /// Value
    pub value: Value,
    /// Time window function
    pub time_window: Option<SearchQueryTimeWindowKind>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct SearchQueryDimensionOrderReq {
    pub in_ext: Option<bool>,
    /// Dimension column key
    pub code: String,
    /// Sort direction
    pub asc: bool,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct SearchQueryMetricsOrderReq {
    pub in_ext: Option<bool>,
    /// Measure column key
    pub code: String,
    pub fun: SearchQueryAggFunKind,
    /// Sort direction
    pub asc: bool,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct SearchQueryDimensionGroupOrderReq {
    pub in_ext: Option<bool>,
    /// Dimension column key
    pub code: String,
    /// Time window function
    pub time_window: Option<SearchQueryTimeWindowKind>,
    /// Sort direction
    pub asc: bool,
}
#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct SearchQueryMetricsHavingReq {
    pub in_ext: Option<bool>,
    pub multi_values: Option<bool>,
    pub data_type: SearchDataTypeKind,
    /// Measure Column key
    pub code: String,
    /// Aggregate function
    pub fun: SearchQueryAggFunKind,
    /// Operator
    pub op: BasicQueryOpKind,
    /// Value
    pub value: Value,
}

/// Query Metrics Response
#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct SearchQueryMetricsResp {
    /// Fact key
    pub tag: String,
    /// Show names
    ///
    /// key = alias name, value = show name
    ///
    /// The format of the alias: `field name__<function name>`
    pub show_names: HashMap<String, String>,
    /// Group
    ///
    /// Format with only one level (single dimension):
    /// map
    /// ```
    /// {
    ///     "":{  // The root group
    ///         "alias name1":value,
    ///         "alias name...":value,
    ///     },
    ///     "<group name1>" {
    ///         "alias name1":value,
    ///         "alias name...":value,
    ///     }
    ///     "<group name...>" {
    ///         "alias name1":value,
    ///         "alias name...":value,
    ///     }
    /// }
    /// ```
    /// array
    /// ```
    /// [   
    ///     {
    ///         "name": "<group name1>",
    ///         "value": value
    ///     },
    ///     
    /// ]
    /// ```
    ///
    /// Format with multiple levels (multiple dimensions):
    /// ```
    /// {
    ///     "":{  // The root group
    ///         "": {
    ///             "alias name1":value,
    ///             "alias name...":value,
    ///         }
    ///     },
    ///     "<group name1>" {
    ///         "": {
    ///             "alias name1":value,
    ///             "alias name...":value,
    ///         },
    ///         "<sub group name...>": {
    ///             "alias name1":value,
    ///             "alias name...":value,
    ///         }
    ///     }
    ///     "<group name...>" {
    ///         "": {
    ///             "alias name1":value,
    ///             "alias name...":value,
    ///         },
    ///         "<sub group name...>": {
    ///             "alias name1":value,
    ///             "alias name...":value,
    ///         }
    ///     }
    /// }
    /// ```
    ///
    /// # Example
    /// ```
    /// {
    ///     "from": "req",
    ///     "show_names": {
    ///         "ct__date": "创建时间",
    ///         "act_hours__sum": "实例工时",
    ///         "status__": "状态",
    ///         "plan_hours__sum": "计划工时"
    ///     },
    ///     "group": {
    ///         "": {
    ///             "": {
    ///                 "act_hours__sum": 180,
    ///                 "plan_hours__sum": 330
    ///             }
    ///         },
    ///         "2023-01-01": {
    ///             "": {
    ///                 "act_hours__sum": 120,
    ///                 "plan_hours__sum": 240
    ///             },
    ///             "open": {
    ///                 "act_hours__sum": 80,
    ///                 "plan_hours__sum": 160
    ///             },
    ///             "close": {
    ///                 "act_hours__sum": 40,
    ///                 "plan_hours__sum": 80
    ///             }
    ///         }
    ///         "2023-01-02": {
    ///             "": {
    ///                 "act_hours__sum": 60,
    ///                 "plan_hours__sum": 90
    ///             },
    ///             "open": {
    ///                 "act_hours__sum": 40,
    ///                 "plan_hours__sum": 60
    ///             },
    ///             "progress": {
    ///                 "act_hours__sum": 20,
    ///                 "plan_hours__sum": 30
    ///             }
    ///         }
    ///     }
    /// }
    /// ```
    pub group: Value,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct SearchExportDataReq {
    pub tags: Vec<String>,
    pub tag_kind: Option<HashMap<String, Vec<String>>>,
    pub tag_key: Option<HashMap<String, Vec<String>>>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct SearchExportDataResp {
    pub tag_data: HashMap<String, Vec<SearchExportAggResp>>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct SearchImportDataReq {
    pub data_source: String,
    pub tag_data: HashMap<String, Vec<SearchImportAggReq>>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct SearchExportAggResp {
    pub tag: String,
    pub kind: String,
    pub key: String,
    pub title: String,
    pub content: String,
    pub data_source: String,
    pub owner: String,
    pub own_paths: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
    pub ext: Value,
    pub visit_keys: Option<Value>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct SearchImportAggReq {
    pub tag: String,
    pub kind: String,
    pub key: String,
    pub title: String,
    pub content: String,
    pub data_source: String,
    pub owner: String,
    pub own_paths: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
    pub ext: Value,
    pub visit_keys: Option<SearchItemVisitKeysReq>,
}

// 分词规则类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SearchWordCombinationsRuleWay {
    Number,
    SpecLength(usize),
    SpecSymbols(Vec<String>),
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct SearchBatchOperateReq {
    pub add_or_modify_reqs: Vec<SearchBatchAddOrModifyItemReq>,
    pub delete_reqs: Vec<SearchBatchDeleteItemReq>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Clone)]
pub struct SearchBatchAddOrModifyItemReq {
    #[oai(validator(pattern = r"^[a-z0-9-_]+$"))]
    pub tag: String,
    #[oai(validator(min_length = "2"))]
    pub kind: String,
    #[oai(validator(min_length = "2"))]
    pub key: TrimString,
    #[oai(validator(min_length = "1"))]
    pub title: String,
    // #[oai(validator(min_length = "2"))]
    pub content: String,
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

impl From<SearchBatchAddOrModifyItemReq> for SearchItemAddReq {
    fn from(value: SearchBatchAddOrModifyItemReq) -> Self {
        Self {
            tag: value.tag,
            kind: value.kind,
            key: value.key,
            title: value.title,
            content: value.content,
            data_source: value.data_source,
            owner: value.owner,
            own_paths: value.own_paths,
            create_time: value.create_time,
            update_time: value.update_time,
            ext: value.ext,
            visit_keys: value.visit_keys,
        }
    }
}

impl From<SearchBatchAddOrModifyItemReq> for SearchItemModifyReq {
    fn from(value: SearchBatchAddOrModifyItemReq) -> Self {
        Self {
            kind: Some(value.kind),
            title: Some(value.title),
            content: Some(value.content),
            owner: value.owner,
            own_paths: value.own_paths,
            create_time: value.create_time,
            update_time: value.update_time,
            ext: value.ext,
            ext_override: None,
            visit_keys: value.visit_keys,
        }
    }
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct SearchBatchDeleteItemReq {
    pub tag: String,
    pub key: String,
}