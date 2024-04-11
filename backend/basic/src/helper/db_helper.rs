//! Database operations helper
//! 
//! 数据库操作辅助操作
use tardis::{
    chrono::{DateTime, ParseError, Utc},
    db::sea_orm,
    log::warn,
    serde_json,
};

/// Convert JSON value to SeaORM value.
///
/// When the JSON value is a string, you can specify whether to add % on both sides of the string through the ``like_by_str`` parameter.
pub fn json_to_sea_orm_value(json_value: &serde_json::Value, like_by_str: bool) -> Option<Vec<sea_orm::Value>> {
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
                if like_by_str {
                    Some(vec![sea_orm::Value::from(format!("%{val}%"))])
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
            let vals = val.iter().map(|json| json_to_sea_orm_value(json, like_by_str)).collect::<Vec<Option<Vec<sea_orm::Value>>>>();
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

/// Convert string to ``DateTime<Utc>``
pub fn str_to_datetime(input: &str) -> Result<DateTime<Utc>, ParseError> {
    DateTime::parse_from_rfc3339(input).map(|dt| dt.with_timezone(&Utc))
}
