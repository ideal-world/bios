use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tardis::{
    basic::{error::TardisError, result::TardisResult},
    serde_json::Value,
};

use crate::basic_enumeration::BasicQueryOpKind;
#[cfg(feature = "default")]
use tardis::web::poem_openapi;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object))]
pub struct BasicQueryCondInfo {
    #[oai(validator(min_length = "1"))]
    pub field: String,
    pub op: BasicQueryOpKind,
    pub value: Value,
}

impl BasicQueryCondInfo {
    // The outer level is the `OR` relationship, the inner level is the `AND` relationship
    pub fn check_or_and_conds(conds: &Vec<Vec<BasicQueryCondInfo>>, check_vars: &HashMap<String, Value>) -> TardisResult<bool> {
        let is_match = conds.iter().any(|and_conds| {
            and_conds.iter().all(|cond| {
                let check_val = check_vars.get(&cond.field).unwrap();
                match &cond.op {
                    BasicQueryOpKind::Eq => &cond.value == check_val,
                    BasicQueryOpKind::Ne => &cond.value != check_val,
                    BasicQueryOpKind::Gt => {
                        if cond.value.is_f64() && check_val.is_f64() {
                            cond.value.as_f64().unwrap() > check_val.as_f64().unwrap()
                        } else if cond.value.is_i64() && check_val.is_i64() {
                            cond.value.as_i64().unwrap() > check_val.as_i64().unwrap()
                        } else if cond.value.is_u64() && check_val.is_u64() {
                            cond.value.as_u64().unwrap() > check_val.as_u64().unwrap()
                        } else {
                            false
                        }
                    }
                    BasicQueryOpKind::Ge => {
                        if cond.value.is_f64() && check_val.is_f64() {
                            cond.value.as_f64().unwrap() >= check_val.as_f64().unwrap()
                        } else if cond.value.is_i64() && check_val.is_i64() {
                            cond.value.as_i64().unwrap() >= check_val.as_i64().unwrap()
                        } else if cond.value.is_u64() && check_val.is_u64() {
                            cond.value.as_u64().unwrap() >= check_val.as_u64().unwrap()
                        } else {
                            false
                        }
                    }
                    BasicQueryOpKind::Lt => {
                        if cond.value.is_f64() && check_val.is_f64() {
                            cond.value.as_f64().unwrap() < check_val.as_f64().unwrap()
                        } else if cond.value.is_i64() && check_val.is_i64() {
                            cond.value.as_i64().unwrap() < check_val.as_i64().unwrap()
                        } else if cond.value.is_u64() && check_val.is_u64() {
                            cond.value.as_u64().unwrap() < check_val.as_u64().unwrap()
                        } else {
                            false
                        }
                    }
                    BasicQueryOpKind::Le => {
                        if cond.value.is_f64() && check_val.is_f64() {
                            cond.value.as_f64().unwrap() <= check_val.as_f64().unwrap()
                        } else if cond.value.is_i64() && check_val.is_i64() {
                            cond.value.as_i64().unwrap() <= check_val.as_i64().unwrap()
                        } else if cond.value.is_u64() && check_val.is_u64() {
                            cond.value.as_u64().unwrap() <= check_val.as_u64().unwrap()
                        } else {
                            false
                        }
                    }
                    BasicQueryOpKind::Like => check_val
                        .as_str()
                        .ok_or_else(|| TardisError::bad_request("Format error in conditional check", "400-basic-cond-check-format-err"))
                        .unwrap()
                        .contains(cond.value.as_str().ok_or_else(|| TardisError::bad_request("Format error in conditional check", "400-basic-cond-check-format-err")).unwrap()),
                    BasicQueryOpKind::In => check_val
                        .as_array()
                        .ok_or_else(|| TardisError::bad_request("Format error in conditional check", "400-basic-cond-check-format-err"))
                        .unwrap()
                        .contains(&cond.value),
                }
            })
        });
        Ok(is_match)
    }
}
