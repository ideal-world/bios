//! Database operations helper
//!
//! 数据库操作辅助操作
use tardis::{
    chrono::{DateTime, ParseError, Utc},
    db::sea_orm,
    log::warn,
    serde_json,
};

use crate::enumeration::BasicQueryOpKind;

/// Convert JSON value to SeaORM value.
///
/// When the JSON value is a string, you can specify whether to add % on both sides of the string through the ``like_by_str`` parameter.
pub fn json_to_sea_orm_value(json_value: &serde_json::Value, like_kind: &BasicQueryOpKind) -> Option<Vec<sea_orm::Value>> {
    match json_value {
        serde_json::Value::Null => None,
        serde_json::Value::Bool(val) => Some(vec![sea_orm::Value::from(*val)]),
        serde_json::Value::Number(val) if val.is_i64() => Some(vec![sea_orm::Value::from(val.as_i64())]),
        serde_json::Value::Number(val) if val.is_u64() => Some(vec![sea_orm::Value::from(val.as_u64())]),
        serde_json::Value::Number(val) if val.is_f64() => Some(vec![sea_orm::Value::from(val.as_f64())]),
        serde_json::Value::Object(_) => Some(vec![sea_orm::Value::from(json_value.clone())]),
        serde_json::Value::String(val) => match str_to_datetime(val) {
            Ok(val) => Some(vec![sea_orm::Value::from(val)]),
            Err(_) => {
                if like_kind == &BasicQueryOpKind::Like || like_kind == &BasicQueryOpKind::NotLike {
                    Some(vec![sea_orm::Value::from(format!("%{val}%"))])
                } else if like_kind == &BasicQueryOpKind::LLike || like_kind == &BasicQueryOpKind::NotLLike {
                    Some(vec![sea_orm::Value::from(format!("%{val}"))])
                } else if like_kind == &BasicQueryOpKind::RLike || like_kind == &BasicQueryOpKind::NotRLike {
                    Some(vec![sea_orm::Value::from(format!("{val}%"))])
                } else {
                    Some(vec![sea_orm::Value::from(val)])
                }
            }
        },
        serde_json::Value::Array(val) => {
            // If the array is empty, return None.
            if val.is_empty() {
                return None;
            }
            // Convert each element in the array to SeaORM value.
            let vals = val.iter().map(|json| json_to_sea_orm_value(json, like_kind)).collect::<Vec<Option<Vec<sea_orm::Value>>>>();
            if vals.iter().any(|v| v.is_none()) {
                warn!("[Basic] json_to_sea_orm_value: json array conversion failed.");
                return None;
            }
            let vals = vals.into_iter().flat_map(|v| v.expect("ignore").into_iter()).collect::<Vec<sea_orm::Value>>();
            Some(vals)
        }
        _ => {
            warn!("[Basic] json_to_sea_orm_value: json conversion failed.");
            None
        }
    }
}

pub fn json_to_sea_orm_value_pure(json_value: serde_json::Value) -> Option<sea_orm::Value> {
    match json_value {
        serde_json::Value::Bool(b) => Some(sea_orm::Value::Bool(Some(b))),
        serde_json::Value::Number(n) => {
            // 尝试将数字转换为 i64 或 f64
            if let Some(i) = n.as_i64() {
                Some(sea_orm::Value::Int(Some(i as i32)))
            } else if let Some(f) = n.as_f64() {
                Some(sea_orm::Value::Float(Some(f as f32)))
            } else {
                None
            }
        }
        serde_json::Value::String(s) => Some(sea_orm::Value::String(Some(Box::new(s)))),
        serde_json::Value::Object(obj) => {
            // 将嵌套的 JSON 对象转换为 Json 类型
            Some(sea_orm::Value::Json(Some(Box::new(serde_json::Value::Object(obj)))))
        }
        serde_json::Value::Array(arr) => {
            // 将数组转换为 Json 类型
            Some(sea_orm::Value::Json(Some(Box::new(serde_json::Value::Array(arr)))))
        }
        serde_json::Value::Null => None,
    }
}

pub fn sea_orm_value_to_json(value: sea_orm::Value) -> Option<serde_json::Value> {
    match value {
        sea_orm::Value::Bool(Some(b)) => Some(serde_json::Value::Bool(b)),
        sea_orm::Value::BigInt(Some(i)) => Some(serde_json::Value::Number(i.into())),
        sea_orm::Value::String(Some(s)) => Some(serde_json::Value::String(s.to_string())),
        sea_orm::Value::Json(Some(json)) => Some(json.as_ref().clone()),
        sea_orm::Value::ChronoDate(Some(dt)) => Some(serde_json::Value::String(dt.to_string())),
        sea_orm::Value::ChronoDateTime(Some(dt)) => Some(serde_json::Value::String(dt.to_string())),
        sea_orm::Value::ChronoDateTimeUtc(Some(dt)) => Some(serde_json::Value::String(dt.to_string())),
        sea_orm::Value::Uuid(Some(u)) => Some(serde_json::Value::String(u.to_string())),
        sea_orm::Value::Decimal(Some(d)) => Some(serde_json::Value::String(d.to_string())),
        _ => None,
    }
}

/// Convert string to ``DateTime<Utc>``
pub fn str_to_datetime(input: &str) -> Result<DateTime<Utc>, ParseError> {
    DateTime::parse_from_rfc3339(input).map(|dt| dt.with_timezone(&Utc))
}
