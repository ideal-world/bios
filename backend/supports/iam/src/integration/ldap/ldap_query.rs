//! LDAP 查询统一入口
//!
//! 根据 Base DN 与查询类型，将 LDAP 搜索请求分发到账号查询或组织查询执行。

use tardis::basic::result::TardisResult;

use crate::iam_config::IamLdapConfig;
use crate::integration::ldap::account::account_query;
use crate::integration::ldap::account::account_result::LdapAccountFields;
use crate::integration::ldap::ldap_parser;
use crate::integration::ldap::organization::{org_query, org_result::LdapOrgFields};

// ========== LdapSqlWhereBuilder trait ==========

/// 构建 LDAP 查询对应 SQL WHERE 子句的 trait。
/// 将 [build_sql_where_clause] 及其内部使用的各子方法统一绑定在此 trait 上，便于扩展与测试。
pub trait LdapSqlWhereBuilder {
    /// objectClass 的固定值列表
    const OBJECT_CLASS_VALUES: &'static [&'static str];

    /// LDAP 属性名 -> 数据库查询字段 映射表 (attr, db_field)
    const ATTR_TO_DB_FIELD: &'static [(&'static str, &'static str)];

    /// 根据 LDAP 查询类型构建 SQL WHERE 条件（主入口，会递归处理 And/Or/Not）
    fn build_sql_where_clause(query_type: &ldap_parser::LdapQueryType, config: &IamLdapConfig) -> TardisResult<String> {
        match query_type {
            ldap_parser::LdapQueryType::Equality { attribute, value } => {
                Self::build_equality_where_clause(attribute, value)
            }
            ldap_parser::LdapQueryType::Present { attribute } => Self::build_present_where_clause(attribute),
            ldap_parser::LdapQueryType::And { filters } => {
                let conditions: Vec<String> = filters
                    .iter()
                    .map(|f| Self::build_sql_where_clause(f, config))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(format!("({})", conditions.join(" AND ")))
            }
            ldap_parser::LdapQueryType::Or { filters } => {
                let conditions: Vec<String> = filters
                    .iter()
                    .map(|f| Self::build_sql_where_clause(f, config))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(format!("({})", conditions.join(" OR ")))
            }
            ldap_parser::LdapQueryType::Not { filter } => {
                let condition = Self::build_sql_where_clause(filter, config)?;
                Ok(format!("NOT ({})", condition))
            }
            ldap_parser::LdapQueryType::Substring { attribute, substrings } => {
                Self::build_substring_where_clause(attribute, substrings)
            }
            ldap_parser::LdapQueryType::GreaterOrEqual { attribute, value } => {
                Self::build_comparison_where_clause(attribute, value, ">=")
            }
            ldap_parser::LdapQueryType::LessOrEqual { attribute, value } => {
                Self::build_comparison_where_clause(attribute, value, "<=")
            }
            ldap_parser::LdapQueryType::ApproxMatch { attribute, value } => {
                Self::build_substring_where_clause(
                    attribute,
                    &ldap3_proto::proto::LdapSubstringFilter {
                        initial: None,
                        any: vec![value.clone()],
                        final_: None,
                    },
                )
            }
        }
    }

    /// 检查是否为 objectClass 属性
    fn is_object_class(attr: &str) -> bool {
        attr.to_lowercase() == "objectclass"
    }

    /// 检查 objectClass 值是否在固定列表中
    fn is_valid_object_class_value(value: &str) -> bool {
        Self::OBJECT_CLASS_VALUES.iter().any(|&v| v.eq_ignore_ascii_case(value))
    }

    /// 根据 LDAP 属性名获取对应的数据库查询字段（使用 [Self::ATTR_TO_DB_FIELD]）
    fn get_db_field(attr: &str) -> TardisResult<String> {
        if Self::is_object_class(attr) {
            return Err(tardis::basic::error::TardisError::format_error(
                "objectClass should be handled by special functions",
                "406-iam-ldap-objectclass-special-handling",
            ));
        }
        Self::ATTR_TO_DB_FIELD
            .iter()
            .find(|(k, _)| (*k).eq_ignore_ascii_case(attr))
            .map(|(_, v)| (*v).to_string())
            .ok_or_else(|| {
                tardis::basic::error::TardisError::format_error(
                    &format!("Unsupported LDAP attribute: {}", attr),
                    "406-iam-ldap-unsupported-attribute",
                )
            })
    }

    /// 构建精确匹配的 WHERE 条件
    fn build_equality_where_clause(attribute: &str, value: &str) -> TardisResult<String> {
        if Self::is_object_class(attribute) {
            if Self::is_valid_object_class_value(value) {
                return Ok("1=1".to_string());
            } else {
                return Ok("1=0".to_string());
            }
        }
        let escaped_value = value.replace("'", "''");
        let field = Self::get_db_field(attribute)?;
        Ok(format!("{} = '{}'", field, escaped_value))
    }

    /// 构建存在性查询的 WHERE 条件
    fn build_present_where_clause(attribute: &str) -> TardisResult<String> {
        if Self::is_object_class(attribute) {
            return Ok("1=1".to_string());
        }
        let field = Self::get_db_field(attribute)?;
        Ok(format!("{} IS NOT NULL AND {} != ''", field, field))
    }

    /// 构建子串匹配的 WHERE 条件
    fn build_substring_where_clause(
        attribute: &str,
        substrings: &ldap3_proto::proto::LdapSubstringFilter,
    ) -> TardisResult<String> {
        if Self::is_object_class(attribute) {
            if substrings.initial.is_none() && substrings.any.is_empty() && substrings.final_.is_none() {
                return Ok("1=1".to_string());
            }
            let matched = Self::OBJECT_CLASS_VALUES.iter().any(|&value| {
                let value_lower = value.to_lowercase();
                if let Some(initial) = &substrings.initial {
                    if !value_lower.starts_with(&initial.to_lowercase()) {
                        return false;
                    }
                }
                let mut remaining_value = if let Some(initial) = &substrings.initial {
                    value_lower[initial.len()..].to_string()
                } else {
                    value_lower.clone()
                };
                for any_part in &substrings.any {
                    let any_lower = any_part.to_lowercase();
                    if let Some(pos) = remaining_value.to_lowercase().find(&any_lower) {
                        remaining_value = remaining_value[pos + any_lower.len()..].to_string();
                    } else {
                        return false;
                    }
                }
                if let Some(final_part) = &substrings.final_ {
                    if !remaining_value.to_lowercase().ends_with(&final_part.to_lowercase()) {
                        return false;
                    }
                }
                true
            });
            return if matched { Ok("1=1".to_string()) } else { Ok("1=0".to_string()) };
        }
        let escaped_initial = substrings.initial.as_ref().map(|s| s.replace("'", "''"));
        let escaped_any = substrings.any.iter().map(|s| s.replace("'", "''")).collect::<Vec<_>>();
        let escaped_final = substrings.final_.as_ref().map(|s| s.replace("'", "''"));
        let field = Self::get_db_field(attribute)?;
        let mut patterns = Vec::new();
        if let Some(initial) = &escaped_initial {
            patterns.push(format!("{} LIKE '{}%'", field, initial));
        }
        for any_part in &escaped_any {
            patterns.push(format!("{} LIKE '%{}%'", field, any_part));
        }
        if let Some(final_part) = &escaped_final {
            patterns.push(format!("{} LIKE '%{}'", field, final_part));
        }
        Ok(patterns.join(" AND "))
    }

    /// 构建比较查询的 WHERE 条件（仅支持 employeenumber 等可比较字段）
    fn build_comparison_where_clause(attribute: &str, value: &str, operator: &str) -> TardisResult<String> {
        if Self::is_object_class(attribute) {
            return Err(tardis::basic::error::TardisError::format_error(
                "objectClass does not support comparison operations",
                "406-iam-ldap-objectclass-no-comparison",
            ));
        }
        let field = Self::get_db_field(attribute)?;
        let escaped_value = value.replace("'", "''");
        Ok(format!("{} {} '{}'", field, operator, escaped_value))
    }
}

/// 执行 LDAP 账号搜索
///
/// 委托给 [account_query::execute_ldap_account_search]。
pub async fn execute_account_search(
    query: &ldap_parser::LdapSearchQuery,
    config: &IamLdapConfig,
) -> TardisResult<Vec<LdapAccountFields>> {
    account_query::execute_ldap_account_search(query, config).await
}

/// 执行 LDAP 组织搜索
///
/// 委托给 [org_query::execute_ldap_org_search]。
pub async fn execute_org_search(
    query: &ldap_parser::LdapSearchQuery,
    config: &IamLdapConfig,
) -> TardisResult<Vec<LdapOrgFields>> {
    org_query::execute_ldap_org_search(query, config).await
}

/// 根据 LDAP 查询类型构建 SQL WHERE 条件（委托给 [LdapSqlWhereBuilder] 默认实现，使用默认 [LdapSqlWhereBuilder::ATTR_TO_DB_FIELD]）
pub fn build_sql_where_clause(query_type: &ldap_parser::LdapQueryType, config: &IamLdapConfig) -> TardisResult<String> {
    // 这里保留一个最简单的默认实现，使用空的映射表与 objectClass 列表。
    // 如需实际业务逻辑，请在具体模块中实现 [LdapSqlWhereBuilder] 并直接调用对应实现。
    match query_type {
        ldap_parser::LdapQueryType::Equality { attribute, value } => {
            if attribute.to_lowercase() == "objectclass" {
                // 默认没有任何有效的 objectClass
                return Ok("1=0".to_string());
            }
            let escaped_value = value.replace("'", "''");
            let field = attribute.to_lowercase();
            Ok(format!("{} = '{}'", field, escaped_value))
        }
        ldap_parser::LdapQueryType::Present { attribute } => {
            if attribute.to_lowercase() == "objectclass" {
                return Ok("1=1".to_string());
            }
            let field = attribute.to_lowercase();
            Ok(format!("{} IS NOT NULL AND {} != ''", field, field))
        }
        ldap_parser::LdapQueryType::And { filters } => {
            let conditions: Vec<String> = filters
                .iter()
                .map(|f| build_sql_where_clause(f, config))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(format!("({})", conditions.join(" AND ")))
        }
        ldap_parser::LdapQueryType::Or { filters } => {
            let conditions: Vec<String> = filters
                .iter()
                .map(|f| build_sql_where_clause(f, config))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(format!("({})", conditions.join(" OR ")))
        }
        ldap_parser::LdapQueryType::Not { filter } => {
            let condition = build_sql_where_clause(filter, config)?;
            Ok(format!("NOT ({})", condition))
        }
        ldap_parser::LdapQueryType::Substring { attribute, substrings } => {
            if attribute.to_lowercase() == "objectclass" {
                if substrings.initial.is_none() && substrings.any.is_empty() && substrings.final_.is_none() {
                    return Ok("1=1".to_string());
                } else {
                    return Ok("1=0".to_string());
                }
            }
            let field = attribute.to_lowercase();
            let escaped_initial = substrings.initial.as_ref().map(|s| s.replace("'", "''"));
            let escaped_any = substrings.any.iter().map(|s| s.replace("'", "''")).collect::<Vec<_>>();
            let escaped_final = substrings.final_.as_ref().map(|s| s.replace("'", "''"));
            let mut patterns = Vec::new();
            if let Some(initial) = &escaped_initial {
                patterns.push(format!("{} LIKE '{}%'", field, initial));
            }
            for any_part in &escaped_any {
                patterns.push(format!("{} LIKE '%{}%'", field, any_part));
            }
            if let Some(final_part) = &escaped_final {
                patterns.push(format!("{} LIKE '%{}'", field, final_part));
            }
            Ok(patterns.join(" AND "))
        }
        ldap_parser::LdapQueryType::GreaterOrEqual { attribute, value } => {
            if attribute.to_lowercase() == "objectclass" {
                return Err(tardis::basic::error::TardisError::format_error(
                    "objectClass does not support comparison operations",
                    "406-iam-ldap-objectclass-no-comparison",
                ));
            }
            let field = attribute.to_lowercase();
            let escaped_value = value.replace("'", "''");
            Ok(format!("{} >= '{}'", field, escaped_value))
        }
        ldap_parser::LdapQueryType::LessOrEqual { attribute, value } => {
            if attribute.to_lowercase() == "objectclass" {
                return Err(tardis::basic::error::TardisError::format_error(
                    "objectClass does not support comparison operations",
                    "406-iam-ldap-objectclass-no-comparison",
                ));
            }
            let field = attribute.to_lowercase();
            let escaped_value = value.replace("'", "''");
            Ok(format!("{} <= '{}'", field, escaped_value))
        }
        ldap_parser::LdapQueryType::ApproxMatch { attribute, value } => {
            build_sql_where_clause(
                &ldap_parser::LdapQueryType::Substring {
                    attribute: attribute.clone(),
                    substrings: ldap3_proto::proto::LdapSubstringFilter {
                        initial: None,
                        any: vec![value.clone()],
                        final_: None,
                    },
                },
                config,
            )
        }
    }
}