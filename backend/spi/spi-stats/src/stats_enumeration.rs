use std::str::FromStr;

use bios_basic::{enumeration::BasicQueryOpKind, helper::db_helper};
use serde::{Deserialize, Serialize};
use strum::Display;
use tardis::{
    basic::{error::TardisError, result::TardisResult},
    chrono::{DateTime, Local, NaiveDate, Utc},
    db::sea_orm::{self, prelude::DateTimeWithTimeZone, sea_query::ArrayType, DbErr, QueryResult, TryGetError, TryGetable},
    serde_json,
    web::poem_openapi,
};

#[derive(Display, Clone, Debug, PartialEq, Eq, Deserialize, Serialize, poem_openapi::Enum, strum::EnumString)]
pub enum StatsDataTypeKind {
    #[oai(rename = "string")]
    String,
    #[oai(rename = "int")]
    Int,
    #[oai(rename = "float")]
    Float,
    #[oai(rename = "double")]
    Double,
    #[oai(rename = "bool")]
    Boolean,
    #[oai(rename = "date")]
    Date,
    #[oai(rename = "datetime")]
    DateTime,
}

impl StatsDataTypeKind {
    fn err_json_value_type(&self) -> TardisError {
        TardisError::internal_error(
            &format!("Encounter an error at json_to_sea_orm_value, expect {type_kind} array", type_kind = self),
            "500-spi-stats-internal-error",
        )
    }

    pub fn to_pg_data_type(&self) -> &str {
        match self {
            StatsDataTypeKind::String => "character varying",
            StatsDataTypeKind::Int => "integer",
            StatsDataTypeKind::Float => "real",
            StatsDataTypeKind::Double => "double precision",
            StatsDataTypeKind::Boolean => "boolean",
            StatsDataTypeKind::Date => "date",
            StatsDataTypeKind::DateTime => "timestamp with time zone",
        }
    }

    pub fn json_to_sea_orm_value(&self, json_value: &serde_json::Value, like_by_str: bool) -> TardisResult<sea_orm::Value> {
        let err_parse_time = || {
            TardisError::internal_error(
                &format!("Encounter an error at json_to_sea_orm_value when parse time, value: {json_value}"),
                "500-spi-stats-internal-error",
            )
        };
        Ok(match self {
            StatsDataTypeKind::String => {
                if like_by_str {
                    sea_orm::Value::from(format!("{}%", json_value.as_str().ok_or_else(|| self.err_json_value_type())?))
                } else {
                    sea_orm::Value::from(json_value.as_str().ok_or_else(|| self.err_json_value_type())?.to_string())
                }
            }
            StatsDataTypeKind::Int => sea_orm::Value::from(json_value.as_f64().ok_or_else(|| self.err_json_value_type())? as i32),
            StatsDataTypeKind::Float => sea_orm::Value::from(json_value.as_f64().ok_or_else(|| self.err_json_value_type())? as f32),
            StatsDataTypeKind::Double => sea_orm::Value::from(json_value.as_f64().ok_or_else(|| self.err_json_value_type())?),
            StatsDataTypeKind::Boolean => sea_orm::Value::from(json_value.as_bool().ok_or_else(|| self.err_json_value_type())?),
            StatsDataTypeKind::Date => {
                sea_orm::Value::from(NaiveDate::parse_from_str(json_value.as_str().ok_or_else(|| self.err_json_value_type())?, "%Y-%m-%d").map_err(|_| err_parse_time())?)
            }
            StatsDataTypeKind::DateTime => sea_orm::Value::from(
                DateTime::parse_from_rfc3339(json_value.as_str().ok_or_else(|| self.err_json_value_type())?).map_err(|_| err_parse_time())?.with_timezone(&Utc),
            ),
        })
    }

    pub fn json_to_sea_orm_value_array(&self, json_value: &serde_json::Value, like_by_str: bool) -> TardisResult<sea_orm::Value> {
        let sea_orm_data_type = match self {
            StatsDataTypeKind::String => sea_orm::sea_query::ArrayType::String,
            StatsDataTypeKind::Int => sea_orm::sea_query::ArrayType::Int,
            StatsDataTypeKind::Float => sea_orm::sea_query::ArrayType::Float,
            StatsDataTypeKind::Double => sea_orm::sea_query::ArrayType::Double,
            StatsDataTypeKind::Boolean => sea_orm::sea_query::ArrayType::Bool,
            StatsDataTypeKind::Date => sea_orm::sea_query::ArrayType::TimeDate,
            StatsDataTypeKind::DateTime => sea_orm::sea_query::ArrayType::TimeDateTimeWithTimeZone,
        };
        let values =
            json_value.as_array().ok_or_else(|| self.err_json_value_type())?.iter().map(|json| self.json_to_sea_orm_value(json, like_by_str)).collect::<TardisResult<Vec<_>>>()?;
        Ok(sea_orm::Value::Array(sea_orm_data_type, Some(Box::new(values))))
    }

    pub fn result_to_sea_orm_value(&self, query_result: &QueryResult, key: &str) -> TardisResult<sea_orm::Value> {
        Ok(match self {
            StatsDataTypeKind::String => sea_orm::Value::from(query_result.try_get::<String>("", key)?),
            StatsDataTypeKind::Int => sea_orm::Value::from(query_result.try_get::<i32>("", key)?),
            StatsDataTypeKind::Float => sea_orm::Value::from(query_result.try_get::<f32>("", key)?),
            StatsDataTypeKind::Double => sea_orm::Value::from(query_result.try_get::<f64>("", key)?),
            StatsDataTypeKind::Boolean => sea_orm::Value::from(query_result.try_get::<bool>("", key)?),
            StatsDataTypeKind::Date => sea_orm::Value::from(query_result.try_get::<NaiveDate>("", key)?),
            StatsDataTypeKind::DateTime => sea_orm::Value::from(query_result.try_get::<DateTimeWithTimeZone>("", key)?),
        })
    }

    pub fn result_to_sea_orm_value_array(&self, query_result: &QueryResult, key: &str) -> TardisResult<sea_orm::Value> {
        Ok(match self {
            StatsDataTypeKind::String => sea_orm::Value::from(query_result.try_get::<Vec<String>>("", key)?),
            StatsDataTypeKind::Int => sea_orm::Value::from(query_result.try_get::<Vec<i32>>("", key)?),
            StatsDataTypeKind::Float => sea_orm::Value::from(query_result.try_get::<Vec<f32>>("", key)?),
            StatsDataTypeKind::Double => sea_orm::Value::from(query_result.try_get::<Vec<f64>>("", key)?),
            StatsDataTypeKind::Boolean => sea_orm::Value::from(query_result.try_get::<Vec<bool>>("", key)?),
            StatsDataTypeKind::Date => sea_orm::Value::from(query_result.try_get::<Vec<NaiveDate>>("", key)?),
            StatsDataTypeKind::DateTime => sea_orm::Value::from(query_result.try_get::<Vec<DateTimeWithTimeZone>>("", key)?),
        })
    }

    pub fn result_to_sea_orm_value_default(&self) -> TardisResult<sea_orm::Value> {
        Ok(match self {
            StatsDataTypeKind::String => sea_orm::Value::String(Some(Box::new("".to_string()))),
            StatsDataTypeKind::Int => sea_orm::Value::Int(Some(*Box::new(0))),
            StatsDataTypeKind::Float => sea_orm::Value::Float(Some(*Box::new(0.0))),
            StatsDataTypeKind::Double => sea_orm::Value::Double(Some(*Box::new(0.0))),
            StatsDataTypeKind::Boolean => sea_orm::Value::Bool(Some(*Box::new(false))),
            StatsDataTypeKind::Date => sea_orm::Value::ChronoDate(Some(Box::new(Local::now().with_timezone(&Utc).date_naive()))),
            StatsDataTypeKind::DateTime => sea_orm::Value::ChronoDateTimeWithTimeZone(Some(Box::new(Local::now().with_timezone(&Utc).fixed_offset()))),
        })
    }

    pub fn result_to_sea_orm_value_array_default(&self) -> TardisResult<sea_orm::Value> {
        Ok(match self {
            StatsDataTypeKind::String => sea_orm::Value::Array(ArrayType::String, Some(Box::new(vec!["".to_string().into()]))),
            StatsDataTypeKind::Int => sea_orm::Value::Array(ArrayType::Int, Some(Box::new(vec![sea_orm::Value::Int(Some(*Box::new(0)))]))),
            StatsDataTypeKind::Float => sea_orm::Value::Array(ArrayType::Float, Some(Box::new(vec![sea_orm::Value::Float(Some(*Box::new(0.0)))]))),
            StatsDataTypeKind::Double => sea_orm::Value::Array(ArrayType::Double, Some(Box::new(vec![sea_orm::Value::Double(Some(*Box::new(0.0)))]))),
            StatsDataTypeKind::Boolean => sea_orm::Value::Array(ArrayType::Bool, Some(Box::new(vec![sea_orm::Value::Bool(Some(*Box::new(false)))]))),
            StatsDataTypeKind::Date => sea_orm::Value::Array(
                ArrayType::ChronoDate,
                Some(Box::new(vec![sea_orm::Value::ChronoDate(Some(Box::new(Local::now().with_timezone(&Utc).date_naive())))])),
            ),
            StatsDataTypeKind::DateTime => sea_orm::Value::Array(
                ArrayType::ChronoDateTimeWithTimeZone,
                Some(Box::new(vec![sea_orm::Value::ChronoDateTimeWithTimeZone(Some(Box::new(
                    Local::now().with_timezone(&Utc).fixed_offset(),
                )))])),
            ),
        })
    }

    pub(crate) fn to_pg_where(
        &self,
        multi_values: bool,
        column_name: &str,
        op: &BasicQueryOpKind,
        param_idx: usize,
        value: &serde_json::Value,
        time_window_fun: &Option<StatsQueryTimeWindowKind>,
    ) -> TardisResult<Option<(String, Vec<sea_orm::Value>)>> {
        if value.is_null() {
            return Ok(None);
        }
        let value = if (self == &StatsDataTypeKind::DateTime || self != &StatsDataTypeKind::Date) && value.is_string() {
            if time_window_fun.is_some() {
                Some(vec![sea_orm::Value::from(value.as_str().ok_or_else(|| self.err_json_value_type())?.to_string())])
            } else {
                let value = self.json_to_sea_orm_value(value, op == &BasicQueryOpKind::Like)?;
                Some(vec![value])
            }
        } else {
            db_helper::json_to_sea_orm_value(value, op)
        };
        let Some(mut value) = value else {
            return Err(TardisError::internal_error("json_to_sea_orm_value result is empty", "spi-stats-inaternal-error"));
        };
        Ok(
            if multi_values && (time_window_fun.is_some() || op != &BasicQueryOpKind::In)
                || self == &StatsDataTypeKind::Int && (op == &BasicQueryOpKind::In || op == &BasicQueryOpKind::Like)
                || self == &StatsDataTypeKind::Float && (op == &BasicQueryOpKind::In || op == &BasicQueryOpKind::Like)
                || self == &StatsDataTypeKind::Double && (op == &BasicQueryOpKind::In || op == &BasicQueryOpKind::Like)
                || self == &StatsDataTypeKind::Boolean && (op != &BasicQueryOpKind::Eq && op != &BasicQueryOpKind::Ne)
                || self == &StatsDataTypeKind::Date && (op == &BasicQueryOpKind::In || op == &BasicQueryOpKind::Like)
                || self == &StatsDataTypeKind::DateTime && (op == &BasicQueryOpKind::In || op == &BasicQueryOpKind::Like)
                || (self != &StatsDataTypeKind::Date && self != &StatsDataTypeKind::DateTime) && time_window_fun.is_some()
            {
                None
            } else if multi_values {
                let mut index = 0;
                let param_sql = value
                    .iter()
                    .map(|_| {
                        let param_idx = param_idx + index;
                        index += 1;
                        format!("${} = any({column_name})", param_idx)
                    })
                    .collect::<Vec<_>>();

                Some((format!("({})", param_sql.join(" or ")), value))
            } else if let Some(time_window_fun) = time_window_fun {
                value.pop().map(|value| {
                    (
                        format!("{} {} ${param_idx}", time_window_fun.to_sql(column_name, self == &StatsDataTypeKind::DateTime), op.to_sql()),
                        vec![value],
                    )
                })
            } else if op == &BasicQueryOpKind::In {
                let mut index = 0;
                let param_sql = value
                    .iter()
                    .map(|_| {
                        let param_idx = param_idx + index;
                        index += 1;
                        format!("${}", param_idx)
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                Some((format!("{column_name} {} ({})", op.to_sql(), param_sql), value))
            } else if self == &StatsDataTypeKind::DateTime || self == &StatsDataTypeKind::Date {
                value.pop().map(|value| {
                    (
                        format!(
                            "coalesce({column_name},'1970-01-01 00:00:00 +00:00')::timestamp with time zone {} ${param_idx}",
                            op.to_sql()
                        ),
                        vec![value],
                    )
                })
            } else {
                value.pop().map(|value| (format!("{column_name} {} ${param_idx}", op.to_sql()), vec![value]))
            },
        )
    }

    pub(crate) fn to_pg_having(
        &self,
        multi_values: bool,
        column_name: &str,
        op: &BasicQueryOpKind,
        param_idx: usize,
        value: &serde_json::Value,
        fun: Option<&StatsQueryAggFunKind>,
    ) -> TardisResult<Option<(String, Vec<sea_orm::Value>)>> {
        let value = if (self == &StatsDataTypeKind::DateTime || self != &StatsDataTypeKind::Date) && value.is_string() {
            let value = self.json_to_sea_orm_value(value, op == &BasicQueryOpKind::Like)?;
            Some(vec![value])
        } else {
            db_helper::json_to_sea_orm_value(value, op)
        };

        let Some(mut value) = value else {
            return Err(TardisError::internal_error(
                "to_pg_having: json_to_sea_orm_value result is empty",
                "spi-stats-inaternal-error",
            ));
        };
        Ok(
            if multi_values && (fun.is_some() || op != &BasicQueryOpKind::In)
                || self == &StatsDataTypeKind::Int && (op == &BasicQueryOpKind::In || op == &BasicQueryOpKind::Like)
                || self == &StatsDataTypeKind::Float && (op == &BasicQueryOpKind::In || op == &BasicQueryOpKind::Like)
                || self == &StatsDataTypeKind::Double && (op == &BasicQueryOpKind::In || op == &BasicQueryOpKind::Like)
                || self == &StatsDataTypeKind::Boolean && (op != &BasicQueryOpKind::Eq && op != &BasicQueryOpKind::Ne)
                || self == &StatsDataTypeKind::Date && (op == &BasicQueryOpKind::In || op == &BasicQueryOpKind::Like)
                || self == &StatsDataTypeKind::DateTime && (op == &BasicQueryOpKind::In || op == &BasicQueryOpKind::Like)
            {
                None
            } else if multi_values {
                let mut index = 0;
                let param_sql = value
                    .iter()
                    .map(|_| {
                        let param_idx = param_idx + index;
                        index += 1;
                        format!("${} = any({column_name})", param_idx)
                    })
                    .collect::<Vec<_>>();
                Some((format!("({})", param_sql.join(" or ")), value))
            } else if let Some(fun) = fun {
                value.pop().map(|value| (format!("{} {} ${param_idx}", fun.to_sql(column_name), op.to_sql()), vec![value]))
            } else if op == &BasicQueryOpKind::In {
                let mut index = 0;
                let param_sql = value
                    .iter()
                    .map(|_| {
                        let param_idx = param_idx + index;
                        index += 1;
                        format!("${}", param_idx)
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                Some((format!("{column_name} {} ({})", op.to_sql(), param_sql), value))
            } else {
                value.pop().map(|value| (format!("{column_name} {} ${param_idx}", op.to_sql()), vec![value]))
            },
        )
    }

    pub(crate) fn to_pg_group(&self, column_name: &str, multi_values: bool, time_window_fun: &Option<StatsQueryTimeWindowKind>) -> Option<String> {
        if multi_values {
            Some(format!(
                "unnest(case when array_length({column_name},1) is null then array_append({column_name},'') else {column_name} end)"
            ))
        } else if let Some(time_window_fun) = time_window_fun {
            if self != &StatsDataTypeKind::Date && self != &StatsDataTypeKind::DateTime {
                return None;
            }
            Some(time_window_fun.to_sql(column_name, self == &StatsDataTypeKind::DateTime))
        } else {
            match self {
                StatsDataTypeKind::String => Some(format!("COALESCE({},'')", column_name)),
                StatsDataTypeKind::Int => Some(format!("COALESCE({}::decimal,0)", column_name)),
                StatsDataTypeKind::Float => Some(format!("COALESCE({}::decimal,0)", column_name)),
                StatsDataTypeKind::Double => Some(format!("COALESCE({}::decimal,0)", column_name)),
                StatsDataTypeKind::Boolean => Some(format!("COALESCE({}::bool,false)", column_name)),
                StatsDataTypeKind::Date => Some(column_name.to_string()),
                StatsDataTypeKind::DateTime => Some(column_name.to_string()),
            }
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

#[derive(Display, Clone, Debug, PartialEq, Eq, Deserialize, Serialize, poem_openapi::Enum, strum::EnumString)]
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

#[derive(Display, Clone, Debug, PartialEq, Eq, Deserialize, Serialize, poem_openapi::Enum, strum::EnumString)]
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
            StatsQueryAggFunKind::Sum => format!("sum(COALESCE({column_name}::decimal,0))"),
            StatsQueryAggFunKind::Avg => format!("avg(COALESCE({column_name}::decimal,0))"),
            StatsQueryAggFunKind::Max => format!("max(COALESCE({column_name}::decimal,0))"),
            StatsQueryAggFunKind::Min => format!("min(COALESCE({column_name}::decimal,0))"),
            StatsQueryAggFunKind::Count => format!("count({column_name})"),
        }
    }
}

impl TryGetable for StatsQueryAggFunKind {
    //TODO not_used
    fn try_get(res: &QueryResult, pre: &str, col: &str) -> Result<Self, TryGetError> {
        let s = String::try_get(res, pre, col)?;
        StatsQueryAggFunKind::from_str(&s).map_err(|_| TryGetError::DbErr(DbErr::RecordNotFound(format!("{pre}:{col}"))))
    }

    fn try_get_by<I: sea_orm::ColIdx>(_res: &QueryResult, _index: I) -> Result<Self, TryGetError> {
        //TODO
        unimplemented!()
    }
}

#[derive(Display, Clone, Debug, PartialEq, Eq, Deserialize, Serialize, poem_openapi::Enum, strum::EnumString)]
pub enum StatsQueryTimeWindowKind {
    #[oai(rename = "date")]
    Date,
    #[oai(rename = "hour")]
    Hour,
    #[oai(rename = "week")]
    Week,
    #[oai(rename = "day")]
    Day,
    #[oai(rename = "month")]
    Month,
    #[oai(rename = "year")]
    Year,
}

impl StatsQueryTimeWindowKind {
    // todo 支撑指定时区
    pub fn to_sql(&self, column_name: &str, is_date_time: bool) -> String {
        if is_date_time {
            match self {
                StatsQueryTimeWindowKind::Date => format!("date({})", Self::is_null_empty_column(column_name, None)),
                // StatsQueryTimeWindowKind::Hour => format!("date_part('hour',timezone('UTC', {column_name}))"),
                StatsQueryTimeWindowKind::Hour => format!(
                    "CONCAT(date_part('year', {}), '-',
                LPAD(date_part('month', {})::text, 2, '0'), '-',
                LPAD(date_part('day', {})::text, 2, '0'), ' ',
                LPAD(date_part('hour', {})::text, 2, '0'))",
                    Self::is_null_empty_column(column_name, None),
                    Self::is_null_empty_column(column_name, None),
                    Self::is_null_empty_column(column_name, None),
                    Self::is_null_empty_column(column_name, None)
                ),
                // StatsQueryTimeWindowKind::Day => format!("date_part('day',timezone('UTC', {column_name}))"),
                StatsQueryTimeWindowKind::Day => format!(
                    "CONCAT(date_part('year', {}), '-',
                LPAD(date_part('month', {})::text, 2, '0'), '-',
                LPAD(date_part('day', {})::text, 2, '0'))",
                    Self::is_null_empty_column(column_name, None),
                    Self::is_null_empty_column(column_name, None),
                    Self::is_null_empty_column(column_name, None)
                ),
                StatsQueryTimeWindowKind::Week => format!(
                    "CONCAT(date_part('year', {}), ' ',
                    date_part('week', {}))",
                    Self::is_null_empty_column(column_name, None),
                    Self::is_null_empty_column(column_name, None)
                ),
                // StatsQueryTimeWindowKind::Month => format!("date_part('month',timezone('UTC', {column_name}))"),
                StatsQueryTimeWindowKind::Month => {
                    format!(
                        "CONCAT(date_part('year', {}), '-',LPAD(date_part('month', {})::text, 2, '0'))",
                        Self::is_null_empty_column(column_name, None),
                        Self::is_null_empty_column(column_name, None)
                    )
                }
                StatsQueryTimeWindowKind::Year => {
                    format!("CONCAT(date_part('year',{}),'')", Self::is_null_empty_column(column_name, None))
                }
            }
        } else {
            match self {
                StatsQueryTimeWindowKind::Date => column_name.to_string(),
                // StatsQueryTimeWindowKind::Hour => format!("date_part('hour', {column_name})"),
                // StatsQueryTimeWindowKind::Day => format!("date_part('day', {column_name})"),
                // StatsQueryTimeWindowKind::Month => format!("date_part('month', {column_name})"),
                // StatsQueryTimeWindowKind::Year => format!("date_part('year', {column_name})"),
                StatsQueryTimeWindowKind::Hour => format!(
                    "CONCAT(date_part('year', {}), '-',
                LPAD(date_part('month', {})::text, 2, '0'), '-',
                LPAD(date_part('day', {})::text, 2, '0'), ' ',
                LPAD(date_part('hour', {})::text, 2, '0'))",
                    Self::is_null_empty_column(column_name, None),
                    Self::is_null_empty_column(column_name, None),
                    Self::is_null_empty_column(column_name, None),
                    Self::is_null_empty_column(column_name, None)
                ),
                StatsQueryTimeWindowKind::Day => format!(
                    "CONCAT(date_part('year', {}), '-',
                LPAD(date_part('month', {})::text, 2, '0'), '-',
                LPAD(date_part('day', {})::text, 2, '0'))",
                    Self::is_null_empty_column(column_name, None),
                    Self::is_null_empty_column(column_name, None),
                    Self::is_null_empty_column(column_name, None)
                ),
                StatsQueryTimeWindowKind::Week => format!(
                    "CONCAT(date_part('year', {}), ' ',
                    date_part('week', {}))",
                    Self::is_null_empty_column(column_name, None),
                    Self::is_null_empty_column(column_name, None)
                ),
                StatsQueryTimeWindowKind::Month => {
                    format!(
                        "CONCAT(date_part('year', {}), '-',LPAD(date_part('month', {})::text, 2, '0'))",
                        Self::is_null_empty_column(column_name, None),
                        Self::is_null_empty_column(column_name, None)
                    )
                }
                StatsQueryTimeWindowKind::Year => {
                    format!("CONCAT(date_part('year',{}),'')", Self::is_null_empty_column(column_name, None))
                }
            }
        }
    }

    pub fn is_null_empty_column(column_name: &str, time_zone: Option<String>) -> String {
        format!(
            "timezone('{}', coalesce({column_name},'1970-01-01 00:00:00 +00:00')::timestamp with time zone)",
            time_zone.unwrap_or("Asia/Shanghai".to_string())
        )
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
