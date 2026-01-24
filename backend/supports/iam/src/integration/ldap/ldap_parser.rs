//! LDAP Search Parser
//!
//! 负责解析LDAP搜索请求和过滤器，将LDAP协议层的请求转换为内部查询结构

use ldap3_proto::proto::LdapSubstringFilter;
use ldap3_proto::simple::*;
use lazy_static::lazy_static;
use tardis::basic::result::TardisResult;
use tardis::regex::Regex;

use crate::iam_config::IamLdapConfig;

lazy_static! {
    static ref CN_R: Regex = Regex::new(r"(,|^)[cC][nN]=(.+?)(,|$)").expect("Regular parsing error");
    static ref OU_R: Regex = Regex::new(r"(,|^)[oO][uU]=(.+?)(,|$)").expect("Regular parsing error");
}

/// LDAP查询实体类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LdapEntityType {
    /// 根DSE查询
    RootDse,
    /// Schema查询
    Subschema,
    /// 账号
    Account,
    /// 组织
    Organization,
    /// 未知（需要根据其他条件判断）
    Unknown,
}

/// LDAP搜索查询结构
/// 将LDAP过滤器转换为统一的查询结构
#[derive(Debug, Clone)]
pub struct LdapSearchQuery {
    /// 查询类型
    pub query_type: LdapQueryType,
    /// Base DN
    pub base: String,
    /// Scope
    pub scope: LdapSearchScope,
    /// 属性列表
    pub attributes: Vec<String>,
}

/// LDAP查询类型
#[derive(Debug, Clone)]
pub enum LdapQueryType {
    /// 根DSE查询
    RootDse,
    /// Schema查询（subschema）
    Subschema,
    /// 精确匹配查询
    Equality {
        attribute: String,
        value: String,
    },
    /// 存在性查询
    Present {
        attribute: String,
    },
    /// AND组合查询
    And {
        filters: Vec<LdapQueryType>,
    },
    /// OR组合查询
    Or {
        filters: Vec<LdapQueryType>,
    },
    /// NOT查询
    Not {
        filter: Box<LdapQueryType>,
    },
    /// 子串匹配查询
    Substring {
        attribute: String,
        substrings: LdapSubstringFilter,
    },
    /// 大于等于查询
    GreaterOrEqual {
        attribute: String,
        value: String,
    },
    /// 小于等于查询
    LessOrEqual {
        attribute: String,
        value: String,
    },
    /// 近似匹配查询
    ApproxMatch {
        attribute: String,
        value: String,
    },
}

/// 解析LDAP搜索请求
pub fn parse_search_request(req: &SearchRequest, config: &IamLdapConfig) -> TardisResult<LdapSearchQuery> {
    // 验证base DN
    if !validate_base_dn(&req.base, config) {
        return Err(tardis::basic::error::TardisError::format_error(
            "Invalid base DN",
            "406-iam-ldap-invalid-base-dn",
        ));
    }

    // 解析过滤器
    let mut query_type = parse_filter(&req.filter)?;

    // 检查是否为根DSE查询：base为空且过滤器为Present(objectClass)
    if req.base.is_empty() {
        if let LdapQueryType::Present { attribute } = &query_type {
            if attribute == "objectClass" {
                query_type = LdapQueryType::RootDse;
            }
        }
    }

    // 检查是否为Schema查询：base是cn=schema,DC=xxx且过滤器为Equality(objectClass=subschema)
    if is_schema_base_dn(&req.base, config) {
        if let LdapQueryType::Equality { attribute, value } = &query_type {
            if attribute.to_lowercase() == "objectclass" && value.to_lowercase() == "subschema" {
                query_type = LdapQueryType::Subschema;
            }
        }
    }

    Ok(LdapSearchQuery {
        query_type,
        base: req.base.clone(),
        scope: req.scope.clone(),
        attributes: req.attrs.clone(),
    })
}

/// 验证base DN是否有效
fn validate_base_dn(base: &str, config: &IamLdapConfig) -> bool {
    if base.is_empty() {
        return true; // 空base用于根DSE查询
    }
    base.to_lowercase().contains(&format!("DC={}", config.dc).to_lowercase())
}

/// 解析LDAP过滤器
fn parse_filter(filter: &LdapFilter) -> TardisResult<LdapQueryType> {
    match filter {
        LdapFilter::Equality(attr, value) => Ok(LdapQueryType::Equality {
            attribute: attr.clone(),
            value: value.clone(),
        }),
        LdapFilter::Present(attr) => Ok(LdapQueryType::Present {
            attribute: attr.clone(),
        }),
        LdapFilter::And(filters) => {
            let parsed_filters: Result<Vec<LdapQueryType>, _> = filters.iter().map(parse_filter).collect();
            Ok(LdapQueryType::And {
                filters: parsed_filters?,
            })
        }
        LdapFilter::Or(filters) => {
            let parsed_filters: Result<Vec<LdapQueryType>, _> = filters.iter().map(parse_filter).collect();
            Ok(LdapQueryType::Or {
                filters: parsed_filters?,
            })
        }
        LdapFilter::Not(filter) => Ok(LdapQueryType::Not {
            filter: Box::new(parse_filter(filter)?),
        }),
        LdapFilter::Substring(attr, substrings) => Ok(LdapQueryType::Substring {
            attribute: attr.clone(),
            substrings: substrings.clone(),
        }),
        LdapFilter::GreaterOrEqual(attr, value) => Ok(LdapQueryType::GreaterOrEqual {
            attribute: attr.clone(),
            value: value.clone(),
        }),
        LdapFilter::LessOrEqual(attr, value) => Ok(LdapQueryType::LessOrEqual {
            attribute: attr.clone(),
            value: value.clone(),
        }),
        LdapFilter::Approx(attr, value) => Ok(LdapQueryType::ApproxMatch {
            attribute: attr.clone(),
            value: value.clone(),
        }),
        _ => Err(tardis::basic::error::TardisError::format_error(
            "Unsupported filter",
            "406-iam-ldap-unsupported-filter",
        )),
    }
}

/// 从base DN中提取CN
pub fn extract_cn_from_base(base: &str) -> Option<String> {
    match CN_R.captures(base) {
        None => None,
        Some(cap) => cap.get(2).map(|cn| cn.as_str().to_string()),
    }
}

/// 从DN中提取CN（用于bind操作）
pub fn extract_cn_from_dn(dn: &str) -> Option<String> {
    extract_cn_from_base(dn)
}

/// 检查是否为根DSE查询
pub fn is_root_dse_query(query: &LdapSearchQuery) -> bool {
    matches!(query.query_type, LdapQueryType::RootDse)
}

/// 检查是否为Schema查询
pub fn is_subschema_query(query: &LdapSearchQuery) -> bool {
    matches!(query.query_type, LdapQueryType::Subschema)
}

/// 检查base DN是否为schema DN（cn=schema,DC=xxx）
fn is_schema_base_dn(base: &str, config: &IamLdapConfig) -> bool {
    let schema_dn = format!("cn=schema,DC={}", config.dc);
    base.to_lowercase() == schema_dn.to_lowercase()
}

/// 检查是否为简单存在性查询（用于检查账户是否存在）
pub fn is_simple_present_query(query: &LdapSearchQuery) -> bool {
    matches!(query.query_type, LdapQueryType::Present { .. }) && !query.base.is_empty() && query.scope == LdapSearchScope::Base
}

/// 检查是否为精确匹配查询
pub fn is_equality_query(query: &LdapSearchQuery) -> bool {
    matches!(query.query_type, LdapQueryType::Equality { .. })
}

/// 检查是否为全量查询
pub fn is_full_query(query: &LdapSearchQuery) -> bool {
    matches!(query.query_type, LdapQueryType::Present { attribute: ref attr } if attr == "objectClass")
}

/// 从Equality查询中提取属性名和值
pub fn extract_equality_values(query: &LdapSearchQuery) -> Option<(&str, &str)> {
    match &query.query_type {
        LdapQueryType::Equality { attribute, value } => Some((attribute, value)),
        _ => None,
    }
}

/// 从Present查询中提取属性名
pub fn extract_present_attribute(query: &LdapSearchQuery) -> Option<&str> {
    match &query.query_type {
        LdapQueryType::Present { attribute } => Some(attribute),
        _ => None,
    }
}

/// 从base DN中提取OU
pub fn extract_ou_from_base(base: &str) -> Option<String> {
    match OU_R.captures(base) {
        None => None,
        Some(cap) => cap.get(2).map(|ou| ou.as_str().to_string()),
    }
}

/// 识别查询的实体类型（根查询、schema查询、账号或组织）
pub fn identify_entity_type(query: &LdapSearchQuery) -> LdapEntityType {
    // 优先检查是否为根DSE查询
    if is_root_dse_query(query) {
        return LdapEntityType::RootDse;
    }

    // 检查是否为Schema查询
    if is_subschema_query(query) {
        return LdapEntityType::Subschema;
    }

    // 从base DN中提取OU
    if let Some(ou) = extract_ou_from_base(&query.base) {
        let ou_lower = ou.to_lowercase();
        if ou_lower == "staff" {
            return LdapEntityType::Account;
        } else if ou_lower == "organizations" || ou_lower == "organization" {
            return LdapEntityType::Organization;
        }
    }

    // 如果没有明确的OU，尝试根据查询的属性来判断
    // 如果查询的是账号特有的属性（如mail, employeeNumber等），则可能是账号查询
    // 如果查询的是组织特有的属性（如sysCode, busCode等），则可能是组织查询
    if let LdapQueryType::Equality { attribute, .. } = &query.query_type {
        let attr_lower = attribute.to_lowercase();
        if matches!(
            attr_lower.as_str(),
            "mail" | "employeenumber" | "samaccountname" | "uid" | "givenname" | "sn" | "displayname"
        ) {
            return LdapEntityType::Account;
        } else if matches!(
            attr_lower.as_str(),
            "syscode" | "sys_code" | "buscode" | "bus_code"
        ) {
            return LdapEntityType::Organization;
        }
    }

    // 如果查询的是objectClass，根据值判断
    if let LdapQueryType::Equality { attribute, value } = &query.query_type {
        if attribute.to_lowercase() == "objectclass" {
            let value_lower = value.to_lowercase();
            if value_lower == "inetorgperson" || value_lower == "uidobject" {
                return LdapEntityType::Account;
            } else if value_lower == "organizationalunit" {
                return LdapEntityType::Organization;
            }
        }
    }

    // 默认返回Unknown，由调用方根据上下文决定
    LdapEntityType::Unknown
}
