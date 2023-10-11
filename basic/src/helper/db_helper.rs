use tardis::{
    chrono::{DateTime, ParseError, Utc},
    db::sea_orm,
    serde_json,
};

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
            if val.is_empty() {
                return None;
            }
            let _dt = match val.first().unwrap() {
                serde_json::Value::Bool(_) => sea_orm::sea_query::ArrayType::Bool,
                serde_json::Value::Number(n) if n.is_i64() => sea_orm::sea_query::ArrayType::BigInt,
                serde_json::Value::Number(n) if n.is_u64() => sea_orm::sea_query::ArrayType::BigInt,
                serde_json::Value::Number(n) if n.is_f64() => sea_orm::sea_query::ArrayType::Double,
                serde_json::Value::String(_) => sea_orm::sea_query::ArrayType::String,
                serde_json::Value::Object(_) => sea_orm::sea_query::ArrayType::Json,
                _ => return None,
            };
            let vals = val.iter().map(|json| json_to_sea_orm_value(json, like_by_str)).collect::<Vec<Option<Vec<sea_orm::Value>>>>();
            if vals.iter().any(|v| v.is_none()) {
                return None;
            }
            let vals = vals.into_iter().flat_map(|v| v.unwrap().into_iter()).collect::<Vec<sea_orm::Value>>();
            Some(vals)
        }
        _ => None,
    }
}

fn str_to_datetime(input: &str) -> Result<DateTime<Utc>, ParseError> {
    DateTime::parse_from_rfc3339(input).map(|dt| dt.with_timezone(&Utc))
}
