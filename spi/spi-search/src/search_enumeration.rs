use std::str::FromStr;

use bios_basic::{basic_enumeration::BasicQueryOpKind, helper::db_helper};
use serde::{Deserialize, Serialize};
use tardis::{
    basic::{error::TardisError, result::TardisResult},
    chrono::{DateTime, NaiveDate, Utc},
    db::sea_orm::{self, prelude::DateTimeWithTimeZone, DbErr, QueryResult, TryGetError, TryGetable},
    derive_more::Display,
    serde_json,
    web::poem_openapi,
};

#[derive(Display, Clone, Debug, PartialEq, Eq, Deserialize, Serialize, poem_openapi::Enum, strum::EnumString)]
pub enum SearchDataTypeKind {
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

impl SearchDataTypeKind {
    fn err_json_value_type(&self) -> TardisError {
        TardisError::internal_error(
            &format!("Encounter an error at json_to_sea_orm_value, expect {type_kind} array", type_kind = self),
            "500-spi-stats-internal-error",
        )
    }

    pub fn to_pg_data_type(&self) -> &str {
        match self {
            SearchDataTypeKind::String => "character varying",
            SearchDataTypeKind::Int => "integer",
            SearchDataTypeKind::Float => "real",
            SearchDataTypeKind::Double => "double precision",
            SearchDataTypeKind::Boolean => "boolean",
            SearchDataTypeKind::Date => "date",
            SearchDataTypeKind::DateTime => "timestamp with time zone",
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
            SearchDataTypeKind::String => {
                if like_by_str {
                    sea_orm::Value::from(format!("{}%", json_value.as_str().ok_or(self.err_json_value_type())?))
                } else {
                    sea_orm::Value::from(json_value.as_str().ok_or(self.err_json_value_type())?.to_string())
                }
            }
            SearchDataTypeKind::Int => sea_orm::Value::from(json_value.as_i64().ok_or(self.err_json_value_type())? as i32),
            SearchDataTypeKind::Float => sea_orm::Value::from(json_value.as_f64().ok_or(self.err_json_value_type())? as f32),
            SearchDataTypeKind::Double => sea_orm::Value::from(json_value.as_f64().ok_or(self.err_json_value_type())?),
            SearchDataTypeKind::Boolean => sea_orm::Value::from(json_value.as_bool().ok_or(self.err_json_value_type())?),
            SearchDataTypeKind::Date => {
                sea_orm::Value::from(NaiveDate::parse_from_str(json_value.as_str().ok_or(self.err_json_value_type())?, "%Y-%m-%d").map_err(|_| err_parse_time())?)
            }
            SearchDataTypeKind::DateTime => {
                sea_orm::Value::from(DateTime::parse_from_rfc3339(json_value.as_str().ok_or(self.err_json_value_type())?).map_err(|_| err_parse_time())?.with_timezone(&Utc))
            }
        })
    }

    pub fn json_to_sea_orm_value_array(&self, json_value: &serde_json::Value, like_by_str: bool) -> TardisResult<sea_orm::Value> {
        let sea_orm_data_type = match self {
            SearchDataTypeKind::String => sea_orm::sea_query::ArrayType::String,
            SearchDataTypeKind::Int => sea_orm::sea_query::ArrayType::Int,
            SearchDataTypeKind::Float => sea_orm::sea_query::ArrayType::Float,
            SearchDataTypeKind::Double => sea_orm::sea_query::ArrayType::Double,
            SearchDataTypeKind::Boolean => sea_orm::sea_query::ArrayType::Bool,
            SearchDataTypeKind::Date => sea_orm::sea_query::ArrayType::TimeDate,
            SearchDataTypeKind::DateTime => sea_orm::sea_query::ArrayType::TimeDateTimeWithTimeZone,
        };
        let values = json_value.as_array().ok_or(self.err_json_value_type())?.iter().map(|json| self.json_to_sea_orm_value(json, like_by_str)).collect::<TardisResult<Vec<_>>>()?;
        Ok(sea_orm::Value::Array(sea_orm_data_type, Some(Box::new(values))))
    }

    pub fn result_to_sea_orm_value(&self, query_result: &QueryResult, key: &str) -> TardisResult<sea_orm::Value> {
        Ok(match self {
            SearchDataTypeKind::String => sea_orm::Value::from(query_result.try_get::<String>("", key)?),
            SearchDataTypeKind::Int => sea_orm::Value::from(query_result.try_get::<i32>("", key)?),
            SearchDataTypeKind::Float => sea_orm::Value::from(query_result.try_get::<f32>("", key)?),
            SearchDataTypeKind::Double => sea_orm::Value::from(query_result.try_get::<f64>("", key)?),
            SearchDataTypeKind::Boolean => sea_orm::Value::from(query_result.try_get::<bool>("", key)?),
            SearchDataTypeKind::Date => sea_orm::Value::from(query_result.try_get::<NaiveDate>("", key)?),
            SearchDataTypeKind::DateTime => sea_orm::Value::from(query_result.try_get::<DateTimeWithTimeZone>("", key)?),
        })
    }

    pub fn result_to_sea_orm_value_array(&self, query_result: &QueryResult, key: &str) -> TardisResult<sea_orm::Value> {
        Ok(match self {
            SearchDataTypeKind::String => sea_orm::Value::from(query_result.try_get::<Vec<String>>("", key)?),
            SearchDataTypeKind::Int => sea_orm::Value::from(query_result.try_get::<Vec<i32>>("", key)?),
            SearchDataTypeKind::Float => sea_orm::Value::from(query_result.try_get::<Vec<f32>>("", key)?),
            SearchDataTypeKind::Double => sea_orm::Value::from(query_result.try_get::<Vec<f64>>("", key)?),
            SearchDataTypeKind::Boolean => sea_orm::Value::from(query_result.try_get::<Vec<bool>>("", key)?),
            SearchDataTypeKind::Date => sea_orm::Value::from(query_result.try_get::<Vec<NaiveDate>>("", key)?),
            SearchDataTypeKind::DateTime => sea_orm::Value::from(query_result.try_get::<Vec<DateTimeWithTimeZone>>("", key)?),
        })
    }

    pub(crate) fn to_pg_where(
        &self,
        multi_values: bool,
        column_name: &str,
        op: &BasicQueryOpKind,
        param_idx: usize,
        value: &serde_json::Value,
        time_window_fun: &Option<SearchQueryTimeWindowKind>,
    ) -> TardisResult<Option<(String, Vec<sea_orm::Value>)>> {
        if value.is_null() {
            return Ok(None);
        }
        let value = if (self == &SearchDataTypeKind::DateTime || self != &SearchDataTypeKind::Date) && value.is_string() {
            if time_window_fun.is_some() {
                Some(vec![sea_orm::Value::from(value.as_str().ok_or(self.err_json_value_type())?.to_string())])
            } else {
                let value = self.json_to_sea_orm_value(value, op == &BasicQueryOpKind::Like)?;
                Some(vec![value])
            }
        } else {
            db_helper::json_to_sea_orm_value(value, op == &BasicQueryOpKind::Like)
        };
        let Some(mut value) = value else {
            return Err(TardisError::internal_error("json_to_sea_orm_value result is empty", "spi-stats-inaternal-error"));
        };
        Ok(
            if multi_values && (time_window_fun.is_some() || op != &BasicQueryOpKind::In)
                || self == &SearchDataTypeKind::Int && (op == &BasicQueryOpKind::In || op == &BasicQueryOpKind::Like)
                || self == &SearchDataTypeKind::Float && (op == &BasicQueryOpKind::In || op == &BasicQueryOpKind::Like)
                || self == &SearchDataTypeKind::Double && (op == &BasicQueryOpKind::In || op == &BasicQueryOpKind::Like)
                || self == &SearchDataTypeKind::Boolean && (op != &BasicQueryOpKind::Eq && op != &BasicQueryOpKind::Ne)
                || self == &SearchDataTypeKind::Date && (op == &BasicQueryOpKind::In || op == &BasicQueryOpKind::Like)
                || self == &SearchDataTypeKind::DateTime && (op == &BasicQueryOpKind::In || op == &BasicQueryOpKind::Like)
                || (self != &SearchDataTypeKind::Date && self != &SearchDataTypeKind::DateTime) && time_window_fun.is_some()
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
                        format!("{} {} ${param_idx}", time_window_fun.to_sql(column_name, self == &SearchDataTypeKind::DateTime), op.to_sql()),
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
        fun: Option<&SearchQueryAggFunKind>,
    ) -> TardisResult<Option<(String, Vec<sea_orm::Value>)>> {
        let value = if (self == &SearchDataTypeKind::DateTime || self != &SearchDataTypeKind::Date) && value.is_string() {
            let value = self.json_to_sea_orm_value(value, op == &BasicQueryOpKind::Like)?;
            Some(vec![value])
        } else {
            db_helper::json_to_sea_orm_value(value, op == &BasicQueryOpKind::Like)
        };

        let Some(mut value) = value else {
            return Err(TardisError::internal_error(
                "to_pg_having: json_to_sea_orm_value result is empty",
                "spi-stats-inaternal-error",
            ));
        };
        Ok(
            if multi_values && (fun.is_some() || op != &BasicQueryOpKind::In)
                || self == &SearchDataTypeKind::Int && (op == &BasicQueryOpKind::In || op == &BasicQueryOpKind::Like)
                || self == &SearchDataTypeKind::Float && (op == &BasicQueryOpKind::In || op == &BasicQueryOpKind::Like)
                || self == &SearchDataTypeKind::Double && (op == &BasicQueryOpKind::In || op == &BasicQueryOpKind::Like)
                || self == &SearchDataTypeKind::Boolean && (op != &BasicQueryOpKind::Eq && op != &BasicQueryOpKind::Ne)
                || self == &SearchDataTypeKind::Date && (op == &BasicQueryOpKind::In || op == &BasicQueryOpKind::Like)
                || self == &SearchDataTypeKind::DateTime && (op == &BasicQueryOpKind::In || op == &BasicQueryOpKind::Like)
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

    pub(crate) fn to_pg_group(&self, column_name: &str, multi_values: bool, time_window_fun: &Option<SearchQueryTimeWindowKind>) -> Option<String> {
        if multi_values {
            Some(format!("unnest({})", column_name))
        } else if let Some(time_window_fun) = time_window_fun {
            if self != &SearchDataTypeKind::Date && self != &SearchDataTypeKind::DateTime {
                return None;
            }
            Some(time_window_fun.to_sql(column_name, self == &SearchDataTypeKind::DateTime))
        } else {
            Some(column_name.to_string())
        }
    }

    pub(crate) fn to_pg_select(&self, column_name: &str, fun: &SearchQueryAggFunKind) -> String {
        fun.to_sql(column_name)
    }
}

impl TryGetable for SearchDataTypeKind {
    fn try_get(res: &QueryResult, pre: &str, col: &str) -> Result<Self, TryGetError> {
        let s = String::try_get(res, pre, col)?;
        SearchDataTypeKind::from_str(&s).map_err(|_| TryGetError::DbErr(DbErr::RecordNotFound(format!("{pre}:{col}"))))
    }

    fn try_get_by<I: sea_orm::ColIdx>(_res: &QueryResult, _index: I) -> Result<Self, TryGetError> {
        panic!("not implemented")
    }
}

#[derive(Display, Clone, Debug, PartialEq, Eq, Deserialize, Serialize, poem_openapi::Enum, strum::EnumString)]
pub enum SearchFactColKind {
    #[oai(rename = "dimension")]
    Dimension,
    #[oai(rename = "measure")]
    Measure,
    #[oai(rename = "ext")]
    Ext,
}

impl TryGetable for SearchFactColKind {
    fn try_get(res: &QueryResult, pre: &str, col: &str) -> Result<Self, TryGetError> {
        let s = String::try_get(res, pre, col)?;
        SearchFactColKind::from_str(&s).map_err(|_| TryGetError::DbErr(DbErr::RecordNotFound(format!("{pre}:{col}"))))
    }

    fn try_get_by<I: sea_orm::ColIdx>(_res: &QueryResult, _index: I) -> Result<Self, TryGetError> {
        panic!("not implemented")
    }
}

#[derive(Display, Clone, Debug, PartialEq, Eq, Deserialize, Serialize, poem_openapi::Enum, strum::EnumString)]
pub enum SearchQueryAggFunKind {
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

impl SearchQueryAggFunKind {
    pub(crate) fn to_sql(&self, column_name: &str) -> String {
        match self {
            SearchQueryAggFunKind::Sum => format!("sum({column_name})"),
            SearchQueryAggFunKind::Avg => format!("avg({column_name})"),
            SearchQueryAggFunKind::Max => format!("max({column_name})"),
            SearchQueryAggFunKind::Min => format!("min({column_name})"),
            SearchQueryAggFunKind::Count => format!("count({column_name})"),
        }
    }
}

impl TryGetable for SearchQueryAggFunKind {
    fn try_get(res: &QueryResult, pre: &str, col: &str) -> Result<Self, TryGetError> {
        let s = String::try_get(res, pre, col)?;
        SearchQueryAggFunKind::from_str(&s).map_err(|_| TryGetError::DbErr(DbErr::RecordNotFound(format!("{pre}:{col}"))))
    }

    fn try_get_by<I: sea_orm::ColIdx>(_res: &QueryResult, _index: I) -> Result<Self, TryGetError> {
        unimplemented!()
    }
}

#[derive(Display, Clone, Debug, PartialEq, Eq, Deserialize, Serialize, poem_openapi::Enum, strum::EnumString)]
pub enum SearchQueryTimeWindowKind {
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

impl SearchQueryTimeWindowKind {
    pub fn to_sql(&self, column_name: &str, is_date_time: bool) -> String {
        if is_date_time {
            match self {
                SearchQueryTimeWindowKind::Date => format!("date(timezone('UTC', {column_name}))"),
                // SearchQueryTimeWindowKind::Hour => format!("date_part('hour',timezone('UTC', {column_name}))"),
                SearchQueryTimeWindowKind::Hour => format!(
                    "CONCAT(date_part('year', timezone('UTC', {column_name})), '-',
                LPAD(date_part('month', timezone('UTC', {column_name}))::text, 2, '0'), '-',
                LPAD(date_part('day', timezone('UTC', {column_name}))::text, 2, '0'), ' ',
                LPAD(date_part('hour', timezone('UTC', {column_name}))::text, 2, '0'))"
                ),
                // SearchQueryTimeWindowKind::Day => format!("date_part('day',timezone('UTC', {column_name}))"),
                SearchQueryTimeWindowKind::Day => format!(
                    "CONCAT(date_part('year', timezone('UTC', {column_name})), '-',
                LPAD(date_part('month', timezone('UTC', {column_name}))::text, 2, '0'), '-',
                LPAD(date_part('day', timezone('UTC', {column_name}))::text, 2, '0'))"
                ),
                SearchQueryTimeWindowKind::Week => format!(
                    "CONCAT(date_part('year', timezone('UTC', {column_name})), ' ',
                    date_part('week', timezone('UTC', {column_name})))"
                ),
                // SearchQueryTimeWindowKind::Month => format!("date_part('month',timezone('UTC', {column_name}))"),
                SearchQueryTimeWindowKind::Month => {
                    format!("CONCAT(date_part('year', timezone('UTC',{column_name})), '-',LPAD(date_part('month', timezone('UTC', {column_name}))::text, 2, '0'))")
                }
                SearchQueryTimeWindowKind::Year => format!("CONCAT(date_part('year',timezone('UTC', {column_name})),'')"),
            }
        } else {
            match self {
                SearchQueryTimeWindowKind::Date => column_name.to_string(),
                // SearchQueryTimeWindowKind::Hour => format!("date_part('hour', {column_name})"),
                // SearchQueryTimeWindowKind::Day => format!("date_part('day', {column_name})"),
                // SearchQueryTimeWindowKind::Month => format!("date_part('month', {column_name})"),
                // SearchQueryTimeWindowKind::Year => format!("date_part('year', {column_name})"),
                SearchQueryTimeWindowKind::Hour => format!(
                    "CONCAT(date_part('year', timezone('UTC', {column_name})), '-',
                LPAD(date_part('month', timezone('UTC', {column_name}))::text, 2, '0'), '-',
                LPAD(date_part('day', timezone('UTC', {column_name}))::text, 2, '0'), ' ',
                LPAD(date_part('hour', timezone('UTC', {column_name}))::text, 2, '0'))"
                ),
                SearchQueryTimeWindowKind::Day => format!(
                    "CONCAT(date_part('year', timezone('UTC', {column_name})), '-',
                LPAD(date_part('month', timezone('UTC', {column_name}))::text, 2, '0'), '-',
                LPAD(date_part('day', timezone('UTC', {column_name}))::text, 2, '0'))"
                ),
                SearchQueryTimeWindowKind::Week => format!(
                    "CONCAT(date_part('year', timezone('UTC', {column_name})), ' ',
                    date_part('week', timezone('UTC', {column_name})))"
                ),
                SearchQueryTimeWindowKind::Month => {
                    format!("CONCAT(date_part('year', timezone('UTC',{column_name})), '-',LPAD(date_part('month', timezone('UTC', {column_name}))::text, 2, '0'))")
                }
                SearchQueryTimeWindowKind::Year => format!("CONCAT(date_part('year',timezone('UTC', {column_name})),'')"),
            }
        }
    }
}

impl TryGetable for SearchQueryTimeWindowKind {
    fn try_get(res: &QueryResult, pre: &str, col: &str) -> Result<Self, TryGetError> {
        let s = String::try_get(res, pre, col)?;
        SearchQueryTimeWindowKind::from_str(&s).map_err(|_| TryGetError::DbErr(DbErr::RecordNotFound(format!("{pre}:{col}"))))
    }

    fn try_get_by<I: sea_orm::ColIdx>(_res: &QueryResult, _index: I) -> Result<Self, TryGetError> {
        panic!("not implemented")
    }
}
