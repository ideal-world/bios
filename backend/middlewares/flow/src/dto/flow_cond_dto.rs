//! Basic DTOs
//!
//! 基础的DTOs
use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use strum::Display;
use tardis::{basic::result::TardisResult, serde_json::Value};

use tardis::web::poem_openapi;

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
    pub op_text: Option<String>,
    /// Query value
    pub value: Value,
}

impl BasicQueryCondInfo {
    /// Check if the ``check_vars`` passed in meet the conditions in ``conds``
    ///
    /// 检查传入的 ``check_vars`` 是否满足 ``conds`` 中的条件
    ///
    ///  The outer level is the `OR` relationship, the inner level is the `AND` relationship
    pub fn check_or_and_conds(conds: &[Vec<BasicQueryCondInfo>], check_vars: &HashMap<String, Value>) -> TardisResult<bool> {
        let is_match = conds.iter().any(|and_conds| {
            and_conds.iter().all(|cond| match check_vars.get(&cond.field) {
                Some(check_val) => match &cond.op {
                    BasicQueryOpKind::Eq => &cond.value == check_val,
                    BasicQueryOpKind::Ne => &cond.value != check_val,
                    BasicQueryOpKind::Gt => {
                        if cond.value.is_f64() && check_val.is_f64() {
                            cond.value.as_f64().unwrap_or(0.0) < check_val.as_f64().unwrap_or(0.0)
                        } else if cond.value.is_i64() && check_val.is_i64() {
                            cond.value.as_i64().unwrap_or(0) < check_val.as_i64().unwrap_or(0)
                        } else if cond.value.is_u64() && check_val.is_u64() {
                            cond.value.as_u64().unwrap_or(0) < check_val.as_u64().unwrap_or(0)
                        } else {
                            false
                        }
                    }
                    BasicQueryOpKind::Ge => {
                        if cond.value.is_f64() && check_val.is_f64() {
                            cond.value.as_f64().unwrap_or(0.0) <= check_val.as_f64().unwrap_or(0.0)
                        } else if cond.value.is_i64() && check_val.is_i64() {
                            cond.value.as_i64().unwrap_or(0) <= check_val.as_i64().unwrap_or(0)
                        } else if cond.value.is_u64() && check_val.is_u64() {
                            cond.value.as_u64().unwrap_or(0) <= check_val.as_u64().unwrap_or(0)
                        } else {
                            false
                        }
                    }
                    BasicQueryOpKind::Lt => {
                        if cond.value.is_f64() && check_val.is_f64() {
                            cond.value.as_f64().unwrap_or(0.0) > check_val.as_f64().unwrap_or(0.0)
                        } else if cond.value.is_i64() && check_val.is_i64() {
                            cond.value.as_i64().unwrap_or(0) > check_val.as_i64().unwrap_or(0)
                        } else if cond.value.is_u64() && check_val.is_u64() {
                            cond.value.as_u64().unwrap_or(0) > check_val.as_u64().unwrap_or(0)
                        } else {
                            false
                        }
                    }
                    BasicQueryOpKind::Le => {
                        if cond.value.is_f64() && check_val.is_f64() {
                            cond.value.as_f64().unwrap_or(0.0) >= check_val.as_f64().unwrap_or(0.0)
                        } else if cond.value.is_i64() && check_val.is_i64() {
                            cond.value.as_i64().unwrap_or(0) >= check_val.as_i64().unwrap_or(0)
                        } else if cond.value.is_u64() && check_val.is_u64() {
                            cond.value.as_u64().unwrap_or(0) >= check_val.as_u64().unwrap_or(0)
                        } else {
                            false
                        }
                    }
                    BasicQueryOpKind::Like
                    | BasicQueryOpKind::LLike
                    | BasicQueryOpKind::RLike => {
                        check_val.as_str().map(|check_val_str| cond.value.as_str().map(|cond_val_str| check_val_str.contains(cond_val_str)).unwrap_or(false)).unwrap_or(false)
                    }
                    BasicQueryOpKind::NotLike
                    | BasicQueryOpKind::NotLLike
                    | BasicQueryOpKind::NotRLike => {
                        check_val.as_str().map(|check_val_str| cond.value.as_str().map(|cond_val_str| !check_val_str.contains(cond_val_str)).unwrap_or(false)).unwrap_or(false)
                    }
                    BasicQueryOpKind::In => check_val
                        .as_array()
                        .map(|check_val_arr| {
                            if cond.value.is_array() {
                                cond.value.as_array().unwrap_or(&vec![]).iter().any(|item| check_val_arr.contains(item))
                            } else {
                                check_val_arr.contains(&cond.value)
                            }
                        })
                        .unwrap_or({
                            if cond.value.is_array() {
                                cond.value.as_array().unwrap_or(&vec![]).contains(check_val)
                            } else {
                                cond.value == *check_val
                            }
                        }),
                    BasicQueryOpKind::NotIn => check_val.as_array().map(|check_val_arr| check_val_arr.contains(&cond.value)).unwrap_or(false),
                    BasicQueryOpKind::IsNull => false,
                    BasicQueryOpKind::IsNotNull => false,
                    BasicQueryOpKind::IsNullOrEmpty => false,
                },
                None => false,
            })
        });
        Ok(is_match)
    }
}

/// Basic query operator
///
/// 基础查询操作符
#[derive(Display, Clone, Debug, PartialEq, Eq, Deserialize, Serialize, poem_openapi::Enum)]
pub enum BasicQueryOpKind {
    #[serde(rename = "=")]
    #[oai(rename = "=")]
    Eq,
    #[serde(rename = "!=")]
    #[oai(rename = "!=")]
    Ne,
    #[serde(rename = ">")]
    #[oai(rename = ">")]
    Gt,
    #[serde(rename = ">=")]
    #[oai(rename = ">=")]
    Ge,
    #[serde(rename = "<")]
    #[oai(rename = "<")]
    Lt,
    #[serde(rename = "<=")]
    #[oai(rename = "<=")]
    Le,
    #[serde(rename = "like")]
    #[oai(rename = "like")]
    Like,
    #[serde(rename = "not_like")]
    #[oai(rename = "not_like")]
    NotLike,
    #[serde(rename = "l_like")]
    #[oai(rename = "l_like")]
    LLike,
    #[serde(rename = "not_l_like")]
    #[oai(rename = "not_l_like")]
    NotLLike,
    #[serde(rename = "r_like")]
    #[oai(rename = "r_like")]
    RLike,
    #[serde(rename = "not_r_like")]
    #[oai(rename = "not_r_like")]
    NotRLike,
    #[serde(rename = "in")]
    #[oai(rename = "in")]
    In,
    #[serde(rename = "not_in")]
    #[oai(rename = "not_in")]
    NotIn,
    #[serde(rename = "is_null")]
    #[oai(rename = "is_null")]
    IsNull,
    #[serde(rename = "is_not_null")]
    #[oai(rename = "is_not_null")]
    IsNotNull,
    #[serde(rename = "is_null_or_empty")]
    #[oai(rename = "is_null_or_empty")]
    IsNullOrEmpty,
}
