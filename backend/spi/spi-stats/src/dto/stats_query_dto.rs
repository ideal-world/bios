use std::collections::HashMap;

use bios_basic::enumeration::BasicQueryOpKind;
use serde::{Deserialize, Serialize};
use tardis::{
    chrono::{DateTime, Utc},
    serde_json::Value,
    web::{poem_openapi, web_resp::TardisPage},
};

use crate::stats_enumeration::{StatsQueryAggFunKind, StatsQueryTimeWindowKind};

/// Query Metrics Request
///
/// 查询指标请求
#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct StatsQueryMetricsReq {
    /// Fact code
    ///
    /// 事实编码
    pub from: String,
    /// Associated external dynamic id, used for ext extended fields
    ///
    /// 关联外部动态id,用于ext扩展字段
    pub rel_external_id: Option<String>,
    /// List of fields
    ///
    /// 字段列表
    pub select: Vec<StatsQueryMetricsSelectReq>,
    /// Ignore distinct key
    /// If true or null, the distinct key will not be counted
    /// If false, the distinct key will be counted
    ///
    /// 是否忽略distinct key
    /// 如果为true或null，则不计算distinct key
    /// 如果为false，则计算distinct key
    pub ignore_distinct: Option<bool>,
    /// List of grouped fields,
    /// the order is related to the returned hierarchy and is handled internally using ROLLUP
    ///
    /// 维度列表
    /// 顺序与返回的层级有关，内部使用ROLLUP处理
    pub group: Vec<StatsQueryDimensionGroupReq>,
    pub own_paths: Option<Vec<String>>,
    /// Ignore group rollup
    /// If true or null, the group rollup will not be counted
    /// If false, the group rollup will be counted
    ///
    /// 是否忽略group rollup
    /// 如果为true或null，则不计算group rollup
    /// 如果为false，则计算group rollup
    pub ignore_group_rollup: Option<bool>,
    /// Filter conditions, two-dimensional array, OR between groups, AND within groups
    ///
    /// 过滤条件, 二维数组, 组内AND, 组间OR
    #[oai(rename = "where")]
    pub _where: Option<Vec<Vec<StatsQueryMetricsWhereReq>>>,
    pub dimension_order: Option<Vec<StatsQueryDimensionOrderReq>>,
    /// Sort conditions
    /// The code and fun must exist in Select
    ///
    /// 排序条件
    /// code和fun必须存在于Select中
    pub metrics_order: Option<Vec<StatsQueryMetricsOrderReq>>,
    /// Sort conditions
    /// The code and fun must exist in Select
    ///
    /// 分组排序条件
    /// code和fun必须存在于Select中
    pub group_order: Option<Vec<StatsQueryDimensionGroupOrderReq>>,
    pub group_agg: Option<bool>,
    /// Filter conditions after group
    /// The code and fun must exist in Select
    ///
    /// 分组后的过滤条件
    /// code和fun必须存在于Select中
    pub having: Option<Vec<StatsQueryMetricsHavingReq>>,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub limit: Option<u32>,
}

// Query Metrics Record Request
///
/// 查询指标记录请求
#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct StatsQueryMetricsRecordReq {
    /// Fact code
    ///
    /// 事实编码
    pub from: String,
    /// Associated external dynamic id, used for ext extended fields
    ///
    /// 关联外部动态id,用于ext扩展字段
    pub rel_external_id: Option<String>,
    pub own_paths: Option<Vec<String>>,
    /// List of fields
    ///
    /// 字段列表
    pub select: Vec<StatsQueryMetricsSelectReq>,
    /// Ignore distinct key
    /// If true or null, the distinct key will not be counted
    /// If false, the distinct key will be counted
    ///
    /// 是否忽略distinct key
    /// 如果为true或null，则不计算distinct key
    /// 如果为false，则计算distinct key
    pub ignore_distinct: Option<bool>,
    /// List of grouped fields,
    /// the order is related to the returned hierarchy and is handled internally using ROLLUP
    ///
    /// 维度列表
    /// 顺序与返回的层级有关，内部使用ROLLUP处理
    pub group: Vec<StatsQueryDimensionGroupReq>,
    /// Filter conditions, two-dimensional array, OR between groups, AND within groups
    ///
    /// 过滤条件, 二维数组, 组内AND, 组间OR
    #[oai(rename = "where")]
    pub _where: Option<Vec<Vec<StatsQueryMetricsWhereReq>>>,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub page_size: u64,
    pub page_number: u64,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Clone)]
pub struct StatsQueryMetricsSelectReq {
    /// Associated external dynamic id, used for ext extended fields
    ///
    /// 关联外部动态id,用于ext扩展字段
    pub rel_external_id: Option<String>,
    /// Measure column key
    ///
    /// 度量字段编码
    pub code: String,
    /// Aggregate function
    ///
    /// 聚合函数
    pub fun: StatsQueryAggFunKind,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Clone)]
pub struct StatsQueryDimensionGroupReq {
    /// Associated external dynamic id, used for ext extended fields
    ///
    /// 关联外部动态id,用于ext扩展字段
    pub rel_external_id: Option<String>,
    /// Dimension column key
    ///
    /// 维度字段编码
    pub code: String,
    /// Time window function
    ///
    /// 时间窗口函数
    pub time_window: Option<StatsQueryTimeWindowKind>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Clone)]
pub struct StatsQueryMetricsWhereReq {
    /// Associated external dynamic id, used for ext extended fields
    ///
    /// 关联外部动态id,用于ext扩展字段
    pub rel_external_id: Option<String>,
    /// Dimension or measure column key
    ///
    /// 度量字段编码
    pub code: String,
    /// Operator
    ///
    /// 操作符
    pub op: BasicQueryOpKind,
    /// Value
    ///
    /// 值
    pub value: Value,
    /// Time window function
    ///
    /// 时间窗口函数
    pub time_window: Option<StatsQueryTimeWindowKind>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Clone)]
pub struct StatsQueryMetricsOrderReq {
    /// Associated external dynamic id, used for ext extended fields
    ///
    /// 关联外部动态id,用于ext扩展字段
    pub rel_external_id: Option<String>,
    /// Measure column key
    ///
    /// 度量字段编码
    pub code: String,
    pub fun: StatsQueryAggFunKind,
    /// Sort direction
    ///
    /// 排序方向
    pub asc: bool,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Clone)]
pub struct StatsQueryDimensionGroupOrderReq {
    /// Associated external dynamic id, used for ext extended fields
    ///
    /// 关联外部动态id,用于ext扩展字段
    pub rel_external_id: Option<String>,
    /// Dimension column key
    ///
    /// 维度字段编码
    pub code: String,
    /// Time window function
    ///
    /// 时间窗口函数
    pub time_window: Option<StatsQueryTimeWindowKind>,
    /// Sort direction
    ///
    /// 排序方向
    pub asc: bool,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Clone)]
pub struct StatsQueryDimensionOrderReq {
    /// Associated external dynamic id, used for ext extended fields
    ///
    /// 关联外部动态id,用于ext扩展字段
    pub rel_external_id: Option<String>,
    /// Dimension column key
    ///
    /// 维度字段编码
    pub code: String,
    /// Sort direction
    ///
    /// 排序方向
    pub asc: bool,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Clone)]
pub struct StatsQueryMetricsHavingReq {
    /// Associated external dynamic id, used for ext extended fields
    ///
    /// 关联外部动态id,用于ext扩展字段
    pub rel_external_id: Option<String>,
    /// Measure Column key
    ///
    /// 度量字段编码
    pub code: String,
    /// Aggregate function
    ///
    /// 聚合函数
    pub fun: StatsQueryAggFunKind,
    /// Operator
    ///
    /// 操作符
    pub op: BasicQueryOpKind,
    /// Value
    ///
    /// 值
    pub value: Value,
}

/// Query Metrics Response
///
/// 查询指标响应
#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct StatsQueryMetricsResp {
    /// Fact key
    ///
    /// 事实编码
    pub from: String,
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
pub struct StatsQueryStatementReq {
    /// List of fields
    ///
    /// 字段列表
    pub select: Vec<StatsQueryStatementSelectReq>,
    /// List of grouped fields,
    /// the order is related to the returned hierarchy and is handled internally using ROLLUP
    ///
    /// 维度列表
    /// 顺序与返回的层级有关，内部使用ROLLUP处理
    pub group: Vec<StatsQueryDimensionGroupReq>,
    pub own_paths: Option<Vec<String>>,
    /// Ignore group rollup
    /// If true or null, the group rollup will not be counted
    /// If false, the group rollup will be counted
    ///
    /// 是否忽略group rollup
    /// 如果为true或null，则不计算group rollup
    /// 如果为false，则计算group rollup
    pub ignore_group_rollup: Option<bool>,
    /// Filter conditions, two-dimensional array, OR between groups, AND within groups
    ///
    /// 过滤条件, 二维数组, 组内AND, 组间OR
    #[oai(rename = "where")]
    pub _where: Option<Vec<Vec<StatsQueryStatementWhereReq>>>,
    pub dimension_order: Option<Vec<StatsQueryDimensionOrderReq>>,
    /// Sort conditions
    /// The code and fun must exist in Select
    ///
    /// 排序条件
    /// code和fun必须存在于Select中
    pub metrics_order: Option<Vec<StatsQueryMetricsOrderReq>>,
    /// Sort conditions
    /// The code and fun must exist in Select
    ///
    /// 分组排序条件
    /// code和fun必须存在于Select中
    pub group_order: Option<Vec<StatsQueryDimensionGroupOrderReq>>,
    pub group_agg: Option<bool>,
    /// Filter conditions after group
    /// The code and fun must exist in Select
    ///
    /// 分组后的过滤条件
    /// code和fun必须存在于Select中
    pub having: Option<Vec<StatsQueryStatementHavingReq>>,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub limit: Option<u32>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct StatsQueryStatementSelectReq {
    /// Fact code
    ///
    /// 事实编码
    pub from: String,
    /// Measure column key
    ///
    /// 度量字段编码
    pub code: String,
    /// Aggregate function
    ///
    /// 聚合函数
    pub fun: StatsQueryAggFunKind,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct StatsQueryStatementWhereReq {
    /// Fact code
    ///
    /// 事实编码
    pub from: String,
    /// Dimension or measure column key
    ///
    /// 度量字段编码
    pub code: String,
    /// Operator
    ///
    /// 操作符
    pub op: BasicQueryOpKind,
    /// Value
    ///
    /// 值
    pub value: Value,
    /// Time window function
    ///
    /// 时间窗口函数
    pub time_window: Option<StatsQueryTimeWindowKind>,
}
#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct StatsQueryStatementHavingReq {
    /// Fact code
    ///
    /// 事实编码
    pub from: String,
    /// Measure Column key
    ///
    /// 度量字段编码
    pub code: String,
    /// Aggregate function
    ///
    /// 聚合函数
    pub fun: StatsQueryAggFunKind,
    /// Operator
    ///
    /// 操作符
    pub op: BasicQueryOpKind,
    /// Value
    ///
    /// 值
    pub value: Value,
}

/// Query Metrics Response
///
/// 查询指标响应
#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct StatsQueryStatementResp {
    pub group: Value,
}
/// Query Metrics Record Detail Response
///
/// 查询指标记录明细响应
#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct StatsQueryRecordDetailResp {
    pub columns: Vec<StatsQueryRecordDetailColumnResp>,

    pub data: TardisPage<HashMap<String, Value>>,
}

/// Query Metrics Record Detail Column Response
///
/// 查询指标记录明细字段响应
#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct StatsQueryRecordDetailColumnResp {
    pub key: String,

    pub show_names: String,
}
