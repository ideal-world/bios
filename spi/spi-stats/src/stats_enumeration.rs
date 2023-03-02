use std::str::FromStr;

use bios_basic::{helper::db_helper, spi::spi_enumeration::SpiQueryOpKind};
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
    #[oai(rename = "string")]
    String,
    #[oai(rename = "int")]
    Int,
    #[oai(rename = "float")]
    Float,
    #[oai(rename = "bool")]
    Boolean,
    #[oai(rename = "date")]
    Date,
    #[oai(rename = "datetime")]
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

    pub fn json_to_sea_orm_value(&self, json_value: &serde_json::Value, like_by_str: bool) -> sea_orm::Value {
        match self {
            StatsDataTypeKind::String => {
                if like_by_str {
                    sea_orm::Value::from(format!("{}%", json_value.as_str().unwrap()))
                } else {
                    sea_orm::Value::from(json_value.as_str().unwrap().to_string())
                }
            }
            StatsDataTypeKind::Int => sea_orm::Value::from(json_value.as_i64().unwrap() as i32),
            StatsDataTypeKind::Float => sea_orm::Value::from(json_value.as_f64().unwrap() as f32),
            StatsDataTypeKind::Boolean => sea_orm::Value::from(json_value.as_bool().unwrap()),
            StatsDataTypeKind::Date => sea_orm::Value::from(json_value.as_str().unwrap().to_string()),
            StatsDataTypeKind::DateTime => sea_orm::Value::from(json_value.as_str().unwrap().to_string()),
        }
    }

    pub fn json_to_sea_orm_value_array(&self, json_value: &serde_json::Value, like_by_str: bool) -> sea_orm::Value {
        let sea_orm_data_type = match self {
            StatsDataTypeKind::String => sea_orm::sea_query::ArrayType::String,
            StatsDataTypeKind::Int => sea_orm::sea_query::ArrayType::Int,
            StatsDataTypeKind::Float => sea_orm::sea_query::ArrayType::Float,
            StatsDataTypeKind::Boolean => sea_orm::sea_query::ArrayType::Bool,
            StatsDataTypeKind::Date => sea_orm::sea_query::ArrayType::TimeDate,
            StatsDataTypeKind::DateTime => sea_orm::sea_query::ArrayType::TimeDateTimeWithTimeZone,
        };
        let values = json_value.as_array().unwrap().iter().map(|json| self.json_to_sea_orm_value(json, like_by_str)).collect();
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

    pub(crate) fn to_pg_where(
        &self,
        multi_values: bool,
        column_name: &str,
        op: &SpiQueryOpKind,
        param_idx: usize,
        value: &serde_json::Value,
        time_window_fun: &Option<StatsQueryTimeWindowKind>,
    ) -> Option<(String, sea_orm::Value)> {
        let value = db_helper::json_to_sea_orm_value(value, op == &SpiQueryOpKind::Like);
        value.as_ref()?;
        let value = value.unwrap();
        if multi_values && (time_window_fun.is_some() || op != &SpiQueryOpKind::In)
            || self == &StatsDataTypeKind::Int && (op == &SpiQueryOpKind::In || op == &SpiQueryOpKind::Like)
            || self == &StatsDataTypeKind::Float && (op == &SpiQueryOpKind::In || op == &SpiQueryOpKind::Like)
            || self == &StatsDataTypeKind::Boolean && (op != &SpiQueryOpKind::Eq && op != &SpiQueryOpKind::Ne)
            || self == &StatsDataTypeKind::Date && (op == &SpiQueryOpKind::In || op == &SpiQueryOpKind::Like)
            || self == &StatsDataTypeKind::DateTime && (op == &SpiQueryOpKind::In || op == &SpiQueryOpKind::Like)
            || (self != &StatsDataTypeKind::Date && self != &StatsDataTypeKind::DateTime) && time_window_fun.is_some()
        {
            None
        } else if multi_values {
            Some((format!("{column_name} @> array[${param_idx}::varchar]"), value))
        } else if let Some(time_window_fun) = time_window_fun {
            Some((
                format!("{} {} ${param_idx}", time_window_fun.to_sql(column_name, self == &StatsDataTypeKind::DateTime), op.to_sql()),
                value,
            ))
        } else {
            Some((format!("{column_name} {} ${param_idx}", op.to_sql()), value))
        }
    }

    pub(crate) fn to_pg_having(
        &self,
        multi_values: bool,
        column_name: &str,
        op: &SpiQueryOpKind,
        param_idx: usize,
        value: &serde_json::Value,
        fun: Option<&StatsQueryAggFunKind>,
    ) -> Option<(String, sea_orm::Value)> {
        let value = db_helper::json_to_sea_orm_value(value, op == &SpiQueryOpKind::Like);
        value.as_ref()?;
        let value = value.unwrap();
        if multi_values && (fun.is_some() || op != &SpiQueryOpKind::In)
            || self == &StatsDataTypeKind::Int && (op == &SpiQueryOpKind::In || op == &SpiQueryOpKind::Like)
            || self == &StatsDataTypeKind::Float && (op == &SpiQueryOpKind::In || op == &SpiQueryOpKind::Like)
            || self == &StatsDataTypeKind::Boolean && (op != &SpiQueryOpKind::Eq && op != &SpiQueryOpKind::Ne)
            || self == &StatsDataTypeKind::Date && (op == &SpiQueryOpKind::In || op == &SpiQueryOpKind::Like)
            || self == &StatsDataTypeKind::DateTime && (op == &SpiQueryOpKind::In || op == &SpiQueryOpKind::Like)
        {
            None
        } else if multi_values {
            Some((format!("{column_name} @> array[${param_idx}::varchar]"), value))
        } else if let Some(fun) = fun {
            Some((format!("{} {} ${param_idx}", fun.to_sql(column_name), op.to_sql()), value))
        } else {
            Some((format!("{column_name} {} ${param_idx}", op.to_sql()), value))
        }
    }

    pub(crate) fn to_pg_group(&self, column_name: &str, time_window_fun: &Option<StatsQueryTimeWindowKind>) -> Option<String> {
        if let Some(time_window_fun) = time_window_fun {
            if self != &StatsDataTypeKind::Date && self != &StatsDataTypeKind::DateTime {
                return None;
            }
            Some(time_window_fun.to_sql(column_name, self == &StatsDataTypeKind::DateTime))
        } else {
            Some(column_name.to_string())
        }
    }

    pub(crate) fn to_pg_select(&self, column_name: &str, fun: &StatsQueryAggFunKind) -> String {
        fun.to_sql(column_name)
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
    #[oai(rename = "dimension")]
    Dimension,
    #[oai(rename = "measure")]
    Measure,
    #[oai(rename = "ext")]
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
    #[oai(rename = "sum")]
    Sum,
    #[oai(rename = "avg")]
    Avg,
    #[oai(rename = "max")]
    Max,
    #[oai(rename = "min")]
    Min,
    #[oai(rename = "count")]
    Count,
}

impl StatsQueryAggFunKind {
    pub(crate) fn to_sql(&self, column_name: &str) -> String {
        match self {
            StatsQueryAggFunKind::Sum => format!("sum({})", column_name),
            StatsQueryAggFunKind::Avg => format!("avg({})", column_name),
            StatsQueryAggFunKind::Max => format!("max({})", column_name),
            StatsQueryAggFunKind::Min => format!("min({})", column_name),
            StatsQueryAggFunKind::Count => format!("count({})", column_name),
        }
    }
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
pub enum StatsQueryTimeWindowKind {
    #[oai(rename = "date")]
    Date,
    #[oai(rename = "hour")]
    Hour,
    #[oai(rename = "day")]
    Day,
    #[oai(rename = "month")]
    Month,
    #[oai(rename = "year")]
    Year,
}

impl StatsQueryTimeWindowKind {
    pub fn to_sql(&self, column_name: &str, is_date_time: bool) -> String {
        if is_date_time {
            match self {
                StatsQueryTimeWindowKind::Date => format!("date(timezone('UTC', {column_name}))"),
                StatsQueryTimeWindowKind::Hour => format!("date_part('hour',timezone('UTC', {column_name}))"),
                StatsQueryTimeWindowKind::Day => format!("date_part('day',timezone('UTC', {column_name}))"),
                StatsQueryTimeWindowKind::Month => format!("date_part('month',timezone('UTC', {column_name}))"),
                StatsQueryTimeWindowKind::Year => format!("date_part('year',timezone('UTC', {column_name}))"),
            }
        } else {
            match self {
                StatsQueryTimeWindowKind::Date => column_name.to_string(),
                StatsQueryTimeWindowKind::Hour => format!("date_part('hour',timezone('UTC', {column_name}))"),
                StatsQueryTimeWindowKind::Day => format!("date_part('day',timezone('UTC', {column_name}))"),
                StatsQueryTimeWindowKind::Month => format!("date_part('month',timezone('UTC', {column_name}))"),
                StatsQueryTimeWindowKind::Year => format!("date_part('year',timezone('UTC', {column_name}))"),
            }
        }
    }
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
