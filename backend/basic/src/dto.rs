//! Basic DTOs
use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tardis::{basic::result::TardisResult, serde_json::Value};

use crate::enumeration::BasicQueryOpKind;
#[cfg(feature = "default")]
use tardis::web::poem_openapi;

/// Basic query condition object
/// 基础的查询条件对象
#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object))]
pub struct BasicQueryCondInfo {
    /// Query field
    #[oai(validator(min_length = "1"))]
    pub field: String,
    /// Query operator
    pub op: BasicQueryOpKind,
    /// Query value
    pub value: Value,
}

impl BasicQueryCondInfo {
    /// Check if the ``check_vars`` passed in meet the conditions in ``conds``
    /// 检查传入的 ``check_vars`` 是否满足 ``conds`` 中的条件
    ///
    ///  The outer level is the `OR` relationship, the inner level is the `AND` relationship
    pub fn check_or_and_conds(conds: &Vec<Vec<BasicQueryCondInfo>>, check_vars: &HashMap<String, Value>) -> TardisResult<bool> {
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
                    BasicQueryOpKind::Like => {
                        check_val.as_str().map(|check_val_str| cond.value.as_str().map(|cond_val_str| check_val_str.contains(cond_val_str)).unwrap_or(false)).unwrap_or(false)
                    }
                    BasicQueryOpKind::NotLike => {
                        check_val.as_str().map(|check_val_str| cond.value.as_str().map(|cond_val_str| check_val_str.contains(cond_val_str)).unwrap_or(false)).unwrap_or(false)
                    }
                    BasicQueryOpKind::In => check_val.as_array().map(|check_val_arr| check_val_arr.contains(&cond.value)).unwrap_or(false),
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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use tardis::{basic::result::TardisResult, serde_json::json};

    use crate::{dto::BasicQueryCondInfo, enumeration::BasicQueryOpKind};

    #[test]
    fn test_check_or_and_conds() -> TardisResult<()> {
        assert!(BasicQueryCondInfo::check_or_and_conds(&vec![vec![]], &HashMap::new())?);
        assert!(
            !(BasicQueryCondInfo::check_or_and_conds(
                &vec![vec![BasicQueryCondInfo {
                    field: "name".to_string(),
                    op: BasicQueryOpKind::Eq,
                    value: json!("gdxr")
                }]],
                &HashMap::new()
            )?)
        );
        // eq
        assert!(BasicQueryCondInfo::check_or_and_conds(
            &vec![vec![BasicQueryCondInfo {
                field: "name".to_string(),
                op: BasicQueryOpKind::Eq,
                value: json!("gdxr")
            }]],
            &HashMap::from([("name".to_string(), json!("gdxr"))])
        )?);
        // gt
        assert!(BasicQueryCondInfo::check_or_and_conds(
            &vec![vec![BasicQueryCondInfo {
                field: "gt".to_string(),
                op: BasicQueryOpKind::Gt,
                value: json!(0)
            }]],
            &HashMap::from([("gt".to_string(), json!(1))])
        )?);
        assert!(
            !(BasicQueryCondInfo::check_or_and_conds(
                &vec![vec![BasicQueryCondInfo {
                    field: "gt".to_string(),
                    op: BasicQueryOpKind::Gt,
                    value: json!(0)
                }]],
                &HashMap::from([("gt".to_string(), json!(-1))])
            )?)
        );
        // ge
        assert!(BasicQueryCondInfo::check_or_and_conds(
            &vec![vec![BasicQueryCondInfo {
                field: "ge".to_string(),
                op: BasicQueryOpKind::Ge,
                value: json!(0)
            }]],
            &HashMap::from([("ge".to_string(), json!(0))])
        )?);
        assert!(
            !(BasicQueryCondInfo::check_or_and_conds(
                &vec![vec![BasicQueryCondInfo {
                    field: "ge".to_string(),
                    op: BasicQueryOpKind::Ge,
                    value: json!(0)
                }]],
                &HashMap::from([("ge".to_string(), json!(-1))])
            )?)
        );
        // lt
        assert!(BasicQueryCondInfo::check_or_and_conds(
            &vec![vec![BasicQueryCondInfo {
                field: "lt".to_string(),
                op: BasicQueryOpKind::Lt,
                value: json!(0)
            }]],
            &HashMap::from([("lt".to_string(), json!(-1))])
        )?);
        assert!(
            !(BasicQueryCondInfo::check_or_and_conds(
                &vec![vec![BasicQueryCondInfo {
                    field: "lt".to_string(),
                    op: BasicQueryOpKind::Lt,
                    value: json!(0)
                }]],
                &HashMap::from([("lt".to_string(), json!(1))])
            )?)
        );
        // le
        assert!(BasicQueryCondInfo::check_or_and_conds(
            &vec![vec![BasicQueryCondInfo {
                field: "le".to_string(),
                op: BasicQueryOpKind::Le,
                value: json!(0)
            }]],
            &HashMap::from([("le".to_string(), json!(0))])
        )?);
        assert!(
            !(BasicQueryCondInfo::check_or_and_conds(
                &vec![vec![BasicQueryCondInfo {
                    field: "le".to_string(),
                    op: BasicQueryOpKind::Le,
                    value: json!(0)
                }]],
                &HashMap::from([("le".to_string(), json!(1))])
            )?)
        );
        assert!(
            !(BasicQueryCondInfo::check_or_and_conds(
                &vec![vec![BasicQueryCondInfo {
                    field: "le".to_string(),
                    op: BasicQueryOpKind::Le,
                    value: json!("ssss".to_string())
                }]],
                &HashMap::from([("le".to_string(), json!(1))])
            )?)
        );
        // like
        assert!(BasicQueryCondInfo::check_or_and_conds(
            &vec![vec![BasicQueryCondInfo {
                field: "like".to_string(),
                op: BasicQueryOpKind::Like,
                value: json!("dx")
            }]],
            &HashMap::from([("like".to_string(), json!("gdxr"))])
        )?);
        assert!(
            !(BasicQueryCondInfo::check_or_and_conds(
                &vec![vec![BasicQueryCondInfo {
                    field: "like".to_string(),
                    op: BasicQueryOpKind::Like,
                    value: json!("ddd")
                }]],
                &HashMap::from([("like".to_string(), json!("gdxr"))])
            )?)
        );
        assert!(
            !(BasicQueryCondInfo::check_or_and_conds(
                &vec![vec![BasicQueryCondInfo {
                    field: "like".to_string(),
                    op: BasicQueryOpKind::Like,
                    value: json!(111)
                }]],
                &HashMap::from([("like".to_string(), json!("dx"))])
            )?)
        );
        assert!(
            !(BasicQueryCondInfo::check_or_and_conds(
                &vec![vec![BasicQueryCondInfo {
                    field: "like".to_string(),
                    op: BasicQueryOpKind::Like,
                    value: json!("gdxr")
                }]],
                &HashMap::from([("like".to_string(), json!(1))])
            )?)
        );
        // In
        assert!(BasicQueryCondInfo::check_or_and_conds(
            &vec![vec![BasicQueryCondInfo {
                field: "in".to_string(),
                op: BasicQueryOpKind::In,
                value: json!("gdxr")
            }]],
            &HashMap::from([("in".to_string(), json!(["gdxr", "ddd"]))])
        )?);
        assert!(
            !(BasicQueryCondInfo::check_or_and_conds(
                &vec![vec![BasicQueryCondInfo {
                    field: "in".to_string(),
                    op: BasicQueryOpKind::In,
                    value: json!("gdxr")
                }]],
                &HashMap::from([("in".to_string(), json!("gdxr"))])
            )?)
        );
        assert!(
            !(BasicQueryCondInfo::check_or_and_conds(
                &vec![vec![BasicQueryCondInfo {
                    field: "in".to_string(),
                    op: BasicQueryOpKind::In,
                    value: json!(["gdxr"])
                }]],
                &HashMap::from([("in".to_string(), json!("gdxr"))])
            )?)
        );
        // and
        assert!(BasicQueryCondInfo::check_or_and_conds(
            &vec![vec![
                BasicQueryCondInfo {
                    field: "in".to_string(),
                    op: BasicQueryOpKind::In,
                    value: json!("gdxr")
                },
                BasicQueryCondInfo {
                    field: "like".to_string(),
                    op: BasicQueryOpKind::Like,
                    value: json!("dx")
                }
            ]],
            &HashMap::from([("in".to_string(), json!(["gdxr"])), ("like".to_string(), json!("gdxr"))])
        )?);
        assert!(
            !(BasicQueryCondInfo::check_or_and_conds(
                &vec![vec![
                    BasicQueryCondInfo {
                        field: "in".to_string(),
                        op: BasicQueryOpKind::In,
                        value: json!("gdxr")
                    },
                    BasicQueryCondInfo {
                        field: "like".to_string(),
                        op: BasicQueryOpKind::Like,
                        value: json!("dx")
                    }
                ]],
                &HashMap::from([("in".to_string(), json!(["gdxr"]))])
            )?)
        );
        assert!(
            !(BasicQueryCondInfo::check_or_and_conds(
                &vec![vec![
                    BasicQueryCondInfo {
                        field: "in".to_string(),
                        op: BasicQueryOpKind::In,
                        value: json!("gdxr")
                    },
                    BasicQueryCondInfo {
                        field: "like".to_string(),
                        op: BasicQueryOpKind::Like,
                        value: json!("dx11")
                    }
                ]],
                &HashMap::from([("in".to_string(), json!(["gdxr"])), ("like".to_string(), json!("gdxr"))])
            )?)
        );
        // or
        assert!(BasicQueryCondInfo::check_or_and_conds(
            &vec![
                vec![BasicQueryCondInfo {
                    field: "in".to_string(),
                    op: BasicQueryOpKind::In,
                    value: json!("gdxr")
                }],
                vec![BasicQueryCondInfo {
                    field: "like".to_string(),
                    op: BasicQueryOpKind::Like,
                    value: json!("dx")
                }]
            ],
            &HashMap::from([("in".to_string(), json!(["gdxr"])), ("like".to_string(), json!("gdxr"))])
        )?);
        assert!(BasicQueryCondInfo::check_or_and_conds(
            &vec![
                vec![BasicQueryCondInfo {
                    field: "in".to_string(),
                    op: BasicQueryOpKind::In,
                    value: json!("gdxr")
                }],
                vec![BasicQueryCondInfo {
                    field: "like".to_string(),
                    op: BasicQueryOpKind::Like,
                    value: json!("dx")
                }]
            ],
            &HashMap::from([("in".to_string(), json!(["gdxr"]))])
        )?);
        assert!(BasicQueryCondInfo::check_or_and_conds(
            &vec![
                vec![BasicQueryCondInfo {
                    field: "in".to_string(),
                    op: BasicQueryOpKind::In,
                    value: json!(["gdxr"])
                }],
                vec![BasicQueryCondInfo {
                    field: "like".to_string(),
                    op: BasicQueryOpKind::Like,
                    value: json!("dx")
                }]
            ],
            &HashMap::from([("in".to_string(), json!(["gdxr"])), ("like".to_string(), json!("gdxr"))])
        )?);
        assert!(
            !(BasicQueryCondInfo::check_or_and_conds(
                &vec![
                    vec![BasicQueryCondInfo {
                        field: "in".to_string(),
                        op: BasicQueryOpKind::In,
                        value: json!("gdxr1")
                    }],
                    vec![BasicQueryCondInfo {
                        field: "like".to_string(),
                        op: BasicQueryOpKind::Like,
                        value: json!("dx")
                    }]
                ],
                &HashMap::from([("in".to_string(), json!(["gdxr"]))])
            )?)
        );
        Ok(())
    }
}
