use std::collections::HashMap;

use itertools::Itertools;
use tardis::{
    basic::{error::TardisError, result::TardisResult},
    db::sea_orm::{sea_query::extension::postgres::PgExpr, Iden, Value},
    regex::{self, Regex},
};

pub(crate) fn process_sql(sql: &str, fact_record: &HashMap<String, Value>) -> TardisResult<(String, Vec<Value>)> {
    let mut values = Vec::new();
    let mut placeholder_index = 1;
    // 正则匹配 `${key}` 形式的占位符
    let re = Regex::new(r"\$\{(\w+)\}").unwrap();
    let mut is_err = false;
    let mut err_msg = Err(TardisError::bad_request("The rel_sql is not a valid sql.", "400-spi-stats-fact-col-conf-rel-sql-not-valid"));
    // 替换 `${key}` 为 `$1`、`$2` 等
    let processed_sql = re.replace_all(sql, |caps: &regex::Captures| {
        // 提取键名并存入值列表
        let key = caps[1].to_string();
        // 从 record 中获取对应的 Value 值
        if let Some(value) = fact_record.get(&key) {
            values.push(value.clone()); // 将值推送到 Vec
        } else {
            is_err = true;
            err_msg = Err(TardisError::bad_request(
                &format!("The key [{}] not found in fact record", key),
                "400-spi-stats-fact-col-conf-key-not-found-in-fact-record",
            ));
        }
        let result = format!("${}", placeholder_index);
        placeholder_index += 1;
        result
    });
    if is_err {
        return err_msg;
    }
    // 返回替换后的 SQL 和提取的值列表
    Ok((processed_sql.to_string(), values))
}

pub(crate) fn process_sql_json(sql: &str, fact_record: serde_json::Value) -> TardisResult<(String, Vec<Value>)> {
    let mut values = Vec::new();
    let mut placeholder_index = 1;
    // 正则匹配 `${key}` 形式的占位符
    let re = Regex::new(r"\$\{(\w+)\}").unwrap();
    let mut is_err = false;
    let mut err_msg = Err(TardisError::bad_request("The rel_sql is not a valid sql.", "400-spi-stats-fact-col-conf-rel-sql-not-valid"));
    // 替换 `${key}` 为 `$1`、`$2` 等
    let processed_sql = re.replace_all(sql, |caps: &regex::Captures| {
        // 提取键名并存入值列表
        let key = caps[1].to_string();
        // 从 record 中获取对应的 Value 值
        if let Some(value) = fact_record.get(&key) {
            // 判断类型进行转换
            match value {
                serde_json::Value::String(s) => values.push(Value::from(s.clone())),
                serde_json::Value::Number(n) => values.push(Value::from(n.as_i64().unwrap())),
                serde_json::Value::Bool(b) => values.push(Value::from(*b)),
                serde_json::Value::Array(a) => {
                    values.push(Value::Json(Some(Box::new(serde_json::Value::Array(a.clone())))));
                }
                serde_json::Value::Null => values.push(Value::from("")),
                _ => {
                    is_err = true;
                    err_msg = Err(TardisError::bad_request(
                        &format!("The key [{}] has an unsupported type", key),
                        "400-spi-stats-fact-col-conf-key-unsupported-type",
                    ));
                }
            }
        } else {
            is_err = true;
            err_msg = Err(TardisError::bad_request(
                &format!("The key [{}] not found in fact record", key),
                "400-spi-stats-fact-col-conf-key-not-found-in-fact-record",
            ));
        }
        let result = format!("${}", placeholder_index);
        placeholder_index += 1;
        result
    });
    if is_err {
        return err_msg;
    }
    // 返回替换后的 SQL 和提取的值列表
    Ok((processed_sql.to_string(), values))
}

pub(crate) fn process_url_json(url: &str, fact_record: serde_json::Value) -> TardisResult<String> {
    // 正则匹配 `${key}` 形式的占位符
    let re = Regex::new(r"\$\{(\w+)\}").unwrap();
    // 替换 `${key}` 为对应值
    let processed_url = re.replace_all(url, |caps: &regex::Captures| {
        // 提取键名并存入值列表
        let key = caps[1].to_string();
        // 从 record 中获取对应的 Value 值
        let result = format!("{}", fact_record.get(&key).unwrap_or(&serde_json::Value::Null));
        result
    });
    // 返回替换后的 url
    Ok(processed_url.to_string())
}

/// validate fact and fact col sql is select sql
pub(crate) fn validate_select_sql(sql: &str) -> bool {
    if sql.is_empty() {
        return true;
    }
    let re = Regex::new(r"(?i)^\s*select\b").expect("should compile regex");
    re.is_match(&sql)
}

/// validate url
pub(crate) fn validate_url(url: &str) -> bool {
    if url.is_empty() {
        return true;
    }
    let re = Regex::new(r"(?i)^(http|https)://").expect("should compile regex");
    re.is_match(&url)
}
#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use tardis::{
        chrono::{DateTime, Utc},
        db::sea_orm::Value,
    };

    use crate::serv::stats_valid_serv::{process_sql, validate_select_sql};

    #[test]
    fn test_validate_select_sql() {
        let sql = "SELECT * FROM users";
        assert_eq!(validate_select_sql(sql), true);
        let sql = " select name FROM users";
        assert_eq!(validate_select_sql(sql), true);
        let sql = "INSERT INTO users (name) VALUES ('John')";
        assert_eq!(validate_select_sql(sql), false);
        let sql = "UPDATE users SET name = 'John'";
        assert_eq!(validate_select_sql(sql), false);
    }

    #[test]
    fn test_generate_sql_and_params() {
        let sql = "select id from table where id = ${id} and name = ${name} and age = ${age} and ct = ${ct}";
        let mut fact_record = HashMap::new();
        fact_record.insert("ct".to_string(), Value::from(DateTime::<Utc>::from_timestamp(1715260800, 0)));
        fact_record.insert("id".to_string(), Value::from("1"));
        fact_record.insert("age".to_string(), Value::from(18));
        fact_record.insert("name".to_string(), Value::from("name1"));
        let (sql, params) = process_sql(sql, &fact_record).unwrap();
        assert_eq!(sql, "select id from table where id = $1 and name = $2 and age = $3 and ct = $4");
        assert_eq!(
            params,
            vec![
                Value::from("1"),
                Value::from("name1"),
                Value::from(18),
                Value::from(DateTime::<Utc>::from_timestamp(1715260800, 0))
            ]
        );
    }
}
