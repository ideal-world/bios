//! Basic DTOs
//!
//! 基础的DTOs
use std::collections::HashMap;

use itertools::Itertools;
use serde::{Deserialize, Serialize};
use strum::Display;
use tardis::chrono::DateTime;
use tardis::serde_json::json;
use tardis::TardisFuns;
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
            and_conds.iter().all(|cond| {
                let field = if cond.field.contains("custom_") {
                    cond.field[7..cond.field.len()].to_string()
                } else {
                    cond.field.clone()
                };
                match check_vars.get(&field) {
                    Some(check_val) => match &cond.op {
                        BasicQueryOpKind::Eq => {
                            if cond.value.is_array() {
                                cond.value.as_array().cloned().unwrap_or(vec![]).first().cloned().unwrap_or(json!("")) == *check_val
                            } else {
                                &cond.value == check_val
                            }
                        }
                        BasicQueryOpKind::Ne => {
                            if cond.value.is_array() {
                                cond.value.as_array().cloned().unwrap_or(vec![]).first().cloned().unwrap_or(json!("")) != *check_val
                            } else {
                                &cond.value != check_val
                            }
                        }
                        BasicQueryOpKind::Gt => {
                            if cond.value.is_f64() {
                                cond.value.as_f64().unwrap_or(0.0)
                                    < if check_val.is_string() {
                                        check_val.as_str().unwrap_or("").parse::<f64>().unwrap_or(0.0)
                                    } else if check_val.is_f64() {
                                        check_val.as_f64().unwrap_or(0.0)
                                    } else {
                                        0.0
                                    }
                            } else if cond.value.is_i64() {
                                cond.value.as_i64().unwrap_or(0)
                                    < if check_val.is_string() {
                                        check_val.as_str().unwrap_or("").parse::<i64>().unwrap_or(0)
                                    } else if check_val.is_i64() {
                                        check_val.as_i64().unwrap_or(0)
                                    } else {
                                        0
                                    }
                            } else if cond.value.is_u64() {
                                cond.value.as_u64().unwrap_or(0)
                                    < if check_val.is_string() {
                                        check_val.as_str().unwrap_or("").parse::<u64>().unwrap_or(0)
                                    } else if check_val.is_f64() {
                                        check_val.as_u64().unwrap_or(0)
                                    } else {
                                        0
                                    }
                            } else if cond.value.is_string() {
                                cond.value.as_str().unwrap_or("") < check_val.as_str().unwrap_or("")
                            } else {
                                false
                            }
                        }
                        BasicQueryOpKind::Ge => {
                            if cond.value.is_f64() {
                                cond.value.as_f64().unwrap_or(0.0)
                                    <= if check_val.is_string() {
                                        check_val.as_str().unwrap_or("").parse::<f64>().unwrap_or(0.0)
                                    } else if check_val.is_f64() {
                                        check_val.as_f64().unwrap_or(0.0)
                                    } else {
                                        0.0
                                    }
                            } else if cond.value.is_i64() {
                                cond.value.as_i64().unwrap_or(0)
                                    <= if check_val.is_string() {
                                        check_val.as_str().unwrap_or("").parse::<i64>().unwrap_or(0)
                                    } else if check_val.is_i64() {
                                        check_val.as_i64().unwrap_or(0)
                                    } else {
                                        0
                                    }
                            } else if cond.value.is_u64() {
                                cond.value.as_u64().unwrap_or(0)
                                    <= if check_val.is_string() {
                                        check_val.as_str().unwrap_or("").parse::<u64>().unwrap_or(0)
                                    } else if check_val.is_f64() {
                                        check_val.as_u64().unwrap_or(0)
                                    } else {
                                        0
                                    }
                            } else if cond.value.is_string() {
                                cond.value.as_str().unwrap_or("") <= check_val.as_str().unwrap_or("")
                            } else {
                                false
                            }
                        }
                        BasicQueryOpKind::Lt => {
                            if cond.value.is_f64() {
                                cond.value.as_f64().unwrap_or(0.0)
                                    > if check_val.is_string() {
                                        check_val.as_str().unwrap_or("").parse::<f64>().unwrap_or(0.0)
                                    } else if check_val.is_f64() {
                                        check_val.as_f64().unwrap_or(0.0)
                                    } else {
                                        0.0
                                    }
                            } else if cond.value.is_i64() {
                                cond.value.as_i64().unwrap_or(0)
                                    > if check_val.is_string() {
                                        check_val.as_str().unwrap_or("").parse::<i64>().unwrap_or(0)
                                    } else if check_val.is_i64() {
                                        check_val.as_i64().unwrap_or(0)
                                    } else {
                                        0
                                    }
                            } else if cond.value.is_u64() {
                                cond.value.as_u64().unwrap_or(0)
                                    > if check_val.is_string() {
                                        check_val.as_str().unwrap_or("").parse::<u64>().unwrap_or(0)
                                    } else if check_val.is_f64() {
                                        check_val.as_u64().unwrap_or(0)
                                    } else {
                                        0
                                    }
                            } else if cond.value.is_string() {
                                cond.value.as_str().unwrap_or("") > check_val.as_str().unwrap_or("")
                            } else {
                                false
                            }
                        }
                        BasicQueryOpKind::Le => {
                            if cond.value.is_f64() {
                                cond.value.as_f64().unwrap_or(0.0)
                                    >= if check_val.is_string() {
                                        check_val.as_str().unwrap_or("").parse::<f64>().unwrap_or(0.0)
                                    } else if check_val.is_f64() {
                                        check_val.as_f64().unwrap_or(0.0)
                                    } else {
                                        0.0
                                    }
                            } else if cond.value.is_i64() {
                                cond.value.as_i64().unwrap_or(0)
                                    >= if check_val.is_string() {
                                        check_val.as_str().unwrap_or("").parse::<i64>().unwrap_or(0)
                                    } else if check_val.is_i64() {
                                        check_val.as_i64().unwrap_or(0)
                                    } else {
                                        0
                                    }
                            } else if cond.value.is_u64() {
                                cond.value.as_u64().unwrap_or(0) >= check_val.as_str().unwrap_or("").parse::<u64>().unwrap_or(0)
                            } else if cond.value.is_string() {
                                cond.value.as_str().unwrap_or("") >= check_val.as_str().unwrap_or("")
                            } else {
                                false
                            }
                        }
                        BasicQueryOpKind::Like | BasicQueryOpKind::LLike | BasicQueryOpKind::RLike => {
                            check_val.as_str().map(|check_val_str| cond.value.as_str().map(|cond_val_str| check_val_str.contains(cond_val_str)).unwrap_or(false)).unwrap_or(false)
                        }
                        BasicQueryOpKind::NotLike | BasicQueryOpKind::NotLLike | BasicQueryOpKind::NotRLike => {
                            check_val.as_str().map(|check_val_str| cond.value.as_str().map(|cond_val_str| !check_val_str.contains(cond_val_str)).unwrap_or(false)).unwrap_or(false)
                        }
                        BasicQueryOpKind::In => {
                            let check_val_arr = if check_val.is_array() {
                                // 数组直接返回
                                check_val.as_array().cloned().unwrap_or(vec![])
                            } else if check_val.is_string() {
                                // 字符串需要判断
                                let check_val_s = check_val.as_str().map(|s| s.to_string()).unwrap_or_default();
                                if let Ok(o) = TardisFuns::json.str_to_json(&check_val_s) {
                                    if o.is_array() {
                                        // 如果字符串可以被序列化为数组，则处理后返回
                                        o.as_array().cloned().unwrap_or(vec![])
                                    } else {
                                        vec![check_val.clone()]
                                    }
                                } else {
                                    vec![check_val.clone()]
                                }
                            } else {
                                vec![check_val.clone()]
                            };
                            if cond.value.is_array() {
                                cond.value.as_array().unwrap_or(&vec![]).iter().any(|item| check_val_arr.contains(item))
                            } else {
                                check_val_arr.contains(&cond.value)
                            }
                        }
                        BasicQueryOpKind::NotIn => {
                            let check_val_arr = if check_val.is_array() {
                                // 数组直接返回
                                check_val.as_array().cloned().unwrap_or(vec![])
                            } else if check_val.is_string() {
                                // 字符串需要判断
                                let check_val_s = check_val.as_str().map(|s| s.to_string()).unwrap_or_default();
                                if let Ok(o) = TardisFuns::json.str_to_json(&check_val_s) {
                                    if o.is_array() {
                                        // 如果字符串可以被序列化为数组，则处理后返回
                                        o.as_array().cloned().unwrap_or(vec![])
                                    } else {
                                        vec![check_val.clone()]
                                    }
                                } else {
                                    vec![check_val.clone()]
                                }
                            } else {
                                vec![check_val.clone()]
                            };
                            if cond.value.is_array() {
                                !cond.value.as_array().unwrap_or(&vec![]).iter().any(|item| check_val_arr.contains(item))
                            } else {
                                !check_val_arr.contains(&cond.value)
                            }
                        }
                        BasicQueryOpKind::IsNull => false,
                        BasicQueryOpKind::IsNotNull => true,
                        BasicQueryOpKind::IsNullOrEmpty => false,
                    },
                    None => cond.op == BasicQueryOpKind::IsNullOrEmpty || cond.op == BasicQueryOpKind::IsNull,
                }
            })
        });
        Ok(is_match)
    }

    /// @TODO 将前端传入的字段格式处理为当前条件判断适配的格式
    pub fn transform(original_vars: HashMap<String, Value>) -> TardisResult<HashMap<String, Value>> {
        let mut result = HashMap::new();
        for (field, value) in original_vars {
            if value.is_string() {
                let s = value.as_str().map(|str| str.to_string()).unwrap_or_default();
                if let Ok(t) = DateTime::parse_from_rfc3339(&s) {
                    result.insert(field, json!(t.to_rfc3339_opts(tardis::chrono::SecondsFormat::Millis, false).to_string()));
                } else if let Ok(o) = TardisFuns::json.str_to_json(&s) {
                    if o.is_array() {
                        let list = o.as_array().cloned().unwrap_or(vec![]);
                        if list.iter().any(|item| item.get("itemId").is_some()) {
                            result.insert(field, json!(list.iter().map(|item| item.get("itemId").cloned().unwrap_or(json!(""))).collect_vec()));
                        } else {
                            result.insert(field, json!(list));
                        }
                    } else if o.is_object() {
                        if let Some(id) = o.get("itemId") {
                            result.insert(field, id.clone());
                        } else {
                            result.insert(field, value);
                        }
                    } else {
                        result.insert(field, value);
                    }
                } else {
                    result.insert(field, value);
                }
            } else {
                result.insert(field, value);
            }
        }
        Ok(result)
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
