use std::str::FromStr;

use serde::{Deserialize, Serialize};
use tardis::{
    chrono::NaiveDate,
    db::sea_orm::{self, prelude::DateTimeWithTimeZone, DbErr, QueryResult, TryGetError, TryGetable},
    derive_more::Display,
    serde_json,
    web::poem_openapi,
};

#[derive(Display, Clone, Debug, PartialEq, Eq, Deserialize, Serialize, poem_openapi::Enum, sea_orm::strum::EnumString)]
pub enum StatsDataTypeKind {
    String,
    Int,
    Float,
    Boolean,
    Date,
    DateTime,
}

impl StatsDataTypeKind {
    pub fn to_pg_data_type(&self) -> &str {
        match self {
            StatsDataTypeKind::String => "character varying",
            StatsDataTypeKind::Int => "integer",
            StatsDataTypeKind::Float => "real",
            StatsDataTypeKind::Boolean => "boolean",
            StatsDataTypeKind::Date => "date",
            StatsDataTypeKind::DateTime => "timestamp with time zone",
        }
    }

    pub fn json_to_sea_orm_value(&self, json_value: &serde_json::Value) -> sea_orm::Value {
        match self {
            StatsDataTypeKind::String => sea_orm::Value::from(json_value.as_str().unwrap()),
            StatsDataTypeKind::Int => sea_orm::Value::from(json_value.as_i64().unwrap() as i32),
            StatsDataTypeKind::Float => sea_orm::Value::from(json_value.as_f64().unwrap() as f32),
            StatsDataTypeKind::Boolean => sea_orm::Value::from(json_value.as_bool().unwrap()),
            StatsDataTypeKind::Date => sea_orm::Value::from(json_value.as_str().unwrap().to_string()),
            StatsDataTypeKind::DateTime => sea_orm::Value::from(json_value.as_str().unwrap().to_string()),
        }
    }

    pub fn json_to_sea_orm_value_array(&self, json_value: &serde_json::Value) -> sea_orm::Value {
        let sea_orm_data_type = match self {
            StatsDataTypeKind::String => sea_orm::sea_query::ArrayType::String,
            StatsDataTypeKind::Int => sea_orm::sea_query::ArrayType::Int,
            StatsDataTypeKind::Float => sea_orm::sea_query::ArrayType::Float,
            StatsDataTypeKind::Boolean => sea_orm::sea_query::ArrayType::Bool,
            StatsDataTypeKind::Date => sea_orm::sea_query::ArrayType::TimeDate,
            StatsDataTypeKind::DateTime => sea_orm::sea_query::ArrayType::TimeDateTimeWithTimeZone,
        };
        let values = json_value.as_array().unwrap().iter().map(|json| self.json_to_sea_orm_value(json)).collect();
        sea_orm::Value::Array(sea_orm_data_type, Some(Box::new(values)))
    }

    pub fn result_to_sea_orm_value(&self, query_result: &QueryResult, key: &str) -> sea_orm::Value {
        match self {
            StatsDataTypeKind::String => sea_orm::Value::from(query_result.try_get::<String>("", key).unwrap()),
            StatsDataTypeKind::Int => sea_orm::Value::from(query_result.try_get::<i32>("", key).unwrap()),
            StatsDataTypeKind::Float => sea_orm::Value::from(query_result.try_get::<f32>("", key).unwrap()),
            StatsDataTypeKind::Boolean => sea_orm::Value::from(query_result.try_get::<bool>("", key).unwrap()),
            StatsDataTypeKind::Date => sea_orm::Value::from(query_result.try_get::<NaiveDate>("", key).unwrap()),
            StatsDataTypeKind::DateTime => sea_orm::Value::from(query_result.try_get::<DateTimeWithTimeZone>("", key).unwrap()),
        }
    }

    pub fn result_to_sea_orm_value_array(&self, query_result: &QueryResult, key: &str) -> sea_orm::Value {
        match self {
            StatsDataTypeKind::String => sea_orm::Value::from(query_result.try_get::<Vec<String>>("", key).unwrap()),
            StatsDataTypeKind::Int => sea_orm::Value::from(query_result.try_get::<Vec<i32>>("", key).unwrap()),
            StatsDataTypeKind::Float => sea_orm::Value::from(query_result.try_get::<Vec<f32>>("", key).unwrap()),
            StatsDataTypeKind::Boolean => sea_orm::Value::from(query_result.try_get::<Vec<bool>>("", key).unwrap()),
            StatsDataTypeKind::Date => sea_orm::Value::from(query_result.try_get::<Vec<NaiveDate>>("", key).unwrap()),
            StatsDataTypeKind::DateTime => sea_orm::Value::from(query_result.try_get::<Vec<DateTimeWithTimeZone>>("", key).unwrap()),
        }
    }
}

impl TryGetable for StatsDataTypeKind {
    fn try_get(res: &QueryResult, pre: &str, col: &str) -> Result<Self, TryGetError> {
        let s = String::try_get(res, pre, col)?;
        StatsDataTypeKind::from_str(&s).map_err(|_| TryGetError::DbErr(DbErr::RecordNotFound(format!("{pre}:{col}"))))
    }

    fn try_get_by<I: sea_orm::ColIdx>(_res: &QueryResult, _index: I) -> Result<Self, TryGetError> {
        panic!("not implemented")
    }
}

#[derive(Display, Clone, Debug, PartialEq, Eq, Deserialize, Serialize, poem_openapi::Enum, sea_orm::strum::EnumString)]
pub enum StatsFactColKind {
    Dimension,
    Measure,
    Ext,
}

impl TryGetable for StatsFactColKind {
    fn try_get(res: &QueryResult, pre: &str, col: &str) -> Result<Self, TryGetError> {
        let s = String::try_get(res, pre, col)?;
        StatsFactColKind::from_str(&s).map_err(|_| TryGetError::DbErr(DbErr::RecordNotFound(format!("{pre}:{col}"))))
    }

    fn try_get_by<I: sea_orm::ColIdx>(_res: &QueryResult, _index: I) -> Result<Self, TryGetError> {
        panic!("not implemented")
    }
}

#[derive(Display, Clone, Debug, PartialEq, Eq, Deserialize, Serialize, poem_openapi::Enum, sea_orm::strum::EnumString)]
pub enum StatsQueryAggFunKind {
    Sum,
    Avg,
    Max,
    Min,
    Count,
}

impl TryGetable for StatsQueryAggFunKind {
    fn try_get(res: &QueryResult, pre: &str, col: &str) -> Result<Self, TryGetError> {
        let s = String::try_get(res, pre, col)?;
        StatsQueryAggFunKind::from_str(&s).map_err(|_| TryGetError::DbErr(DbErr::RecordNotFound(format!("{pre}:{col}"))))
    }

    fn try_get_by<I: sea_orm::ColIdx>(_res: &QueryResult, _index: I) -> Result<Self, TryGetError> {
        panic!("not implemented")
    }
}

#[derive(Display, Clone, Debug, PartialEq, Eq, Deserialize, Serialize, poem_openapi::Enum, sea_orm::strum::EnumString)]
pub enum StatsQueryFunKind {
    Sum,
    Avg,
    Max,
    Min,
    Count,
}

impl TryGetable for StatsQueryFunKind {
    fn try_get(res: &QueryResult, pre: &str, col: &str) -> Result<Self, TryGetError> {
        let s = String::try_get(res, pre, col)?;
        StatsQueryFunKind::from_str(&s).map_err(|_| TryGetError::DbErr(DbErr::RecordNotFound(format!("{pre}:{col}"))))
    }

    fn try_get_by<I: sea_orm::ColIdx>(_res: &QueryResult, _index: I) -> Result<Self, TryGetError> {
        panic!("not implemented")
    }
}

#[derive(Display, Clone, Debug, PartialEq, Eq, Deserialize, Serialize, poem_openapi::Enum, sea_orm::strum::EnumString)]
pub enum StatsQueryTimeWindowKind {
    Date,
    Hour,
    Day,
    Month,
    Year,
}

impl TryGetable for StatsQueryTimeWindowKind {
    fn try_get(res: &QueryResult, pre: &str, col: &str) -> Result<Self, TryGetError> {
        let s = String::try_get(res, pre, col)?;
        StatsQueryTimeWindowKind::from_str(&s).map_err(|_| TryGetError::DbErr(DbErr::RecordNotFound(format!("{pre}:{col}"))))
    }

    fn try_get_by<I: sea_orm::ColIdx>(_res: &QueryResult, _index: I) -> Result<Self, TryGetError> {
        panic!("not implemented")
    }
}
