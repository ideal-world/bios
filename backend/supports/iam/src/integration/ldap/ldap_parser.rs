//! LDAP Search Parser
//!
//! 负责解析LDAP搜索请求和过滤器，将LDAP协议层的请求转换为内部查询结构

use lazy_static::lazy_static;
use ldap3_proto::proto::LdapSubstringFilter;
use ldap3_proto::simple::*;
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
    /// Root DSE（协议能力入口）
    RootDse,
    /// Subschema Entry（Schema 定义）
    Subschema,
    /// 普通目录条目（DIT 中的一切节点）
    Entry,
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
    /// 精确匹配查询
    Equality { attribute: String, value: String },
    /// 存在性查询
    Present { attribute: String },
    /// AND组合查询
    And { filters: Vec<LdapQueryType> },
    /// OR组合查询
    Or { filters: Vec<LdapQueryType> },
    /// NOT查询
    Not { filter: Box<LdapQueryType> },
    /// 子串匹配查询
    Substring { attribute: String, substrings: LdapSubstringFilter },
    /// 大于等于查询
    GreaterOrEqual { attribute: String, value: String },
    /// 小于等于查询
    LessOrEqual { attribute: String, value: String },
    /// 近似匹配查询
    ApproxMatch { attribute: String, value: String },
}

/// LDAP Base 层级类型枚举
/// 用于表示当前 base 的层级类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LdapBaseDnLevel {
    /// 域节点
    /// 位于目录树的最顶层，表示域组件
    Domain,
    /// OU 节点
    /// 位于域节点之下，表示组织单位
    /// 值为实际的 ou 值
    Ou(String),
    /// 条目节点
    /// 位于 OU 节点之下，可能是账号、组织或其他条目
    /// 第一个值存放的是 ou，第二个值存放的是 cn
    Item(String, String),
}

/// 解析LDAP搜索请求
pub fn parse_search_request(req: &SearchRequest, entity_type: LdapEntityType, config: &IamLdapConfig) -> TardisResult<LdapSearchQuery> {
    // 验证base DN
    if !validate_base_dn(&req.base, entity_type, config) {
        return Err(tardis::basic::error::TardisError::format_error("Invalid base DN", "406-iam-ldap-invalid-base-dn"));
    }

    // 解析过滤器
    let query_type = parse_filter(&req.filter)?;

    Ok(LdapSearchQuery {
        query_type,
        base: req.base.clone(),
        scope: req.scope.clone(),
        attributes: req.attrs.clone(),
    })
}

/// 验证base DN是否有效
fn validate_base_dn(base: &str, entity_type: LdapEntityType, config: &IamLdapConfig) -> bool {
    match entity_type {
        LdapEntityType::RootDse => base.is_empty(),
        LdapEntityType::Subschema => base.to_lowercase() == config.schema_dn.to_lowercase(),
        LdapEntityType::Entry => base.to_lowercase().contains(&config.base_dn.to_lowercase()),
    }
}

/// 解析LDAP过滤器
fn parse_filter(filter: &LdapFilter) -> TardisResult<LdapQueryType> {
    match filter {
        LdapFilter::Equality(attr, value) => Ok(LdapQueryType::Equality {
            attribute: attr.clone(),
            value: value.clone(),
        }),
        LdapFilter::Present(attr) => Ok(LdapQueryType::Present { attribute: attr.clone() }),
        LdapFilter::And(filters) => {
            let parsed_filters: Result<Vec<LdapQueryType>, _> = filters.iter().map(parse_filter).collect();
            Ok(LdapQueryType::And { filters: parsed_filters? })
        }
        LdapFilter::Or(filters) => {
            let parsed_filters: Result<Vec<LdapQueryType>, _> = filters.iter().map(parse_filter).collect();
            Ok(LdapQueryType::Or { filters: parsed_filters? })
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
        _ => Err(tardis::basic::error::TardisError::format_error("Unsupported filter", "406-iam-ldap-unsupported-filter")),
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
pub fn is_root_dse_query(req: &SearchRequest) -> bool {
    req.base.is_empty()
}

/// 检查是否为Schema查询
pub fn is_subschema_query(req: &SearchRequest, config: &IamLdapConfig) -> bool {
    let schema_dn = config.schema_dn.clone();
    req.base.to_lowercase() == schema_dn.to_lowercase()
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

/// 从base DN中提取OU
pub fn extract_ou_from_base(base: &str) -> Option<String> {
    match OU_R.captures(base) {
        None => None,
        Some(cap) => cap.get(2).map(|ou| ou.as_str().to_string()),
    }
}

/// 识别查询的实体类型（根查询、schema查询、账号或组织）
pub fn identify_entity_type(req: &SearchRequest, config: &IamLdapConfig) -> LdapEntityType {
    // 优先检查是否为根DSE查询
    if is_root_dse_query(req) {
        return LdapEntityType::RootDse;
    }

    // 检查是否为Schema查询
    if is_subschema_query(req, config) {
        return LdapEntityType::Subschema;
    }

    LdapEntityType::Entry
}

pub fn parse_base_dn_components(base: &str, config: &IamLdapConfig) -> (Option<String>, Option<String>, Option<String>) {
    let mut cn = None;
    let mut ou = None;
    let mut dc = None;

    // 提取 CN
    if let Some(cn_val) = extract_cn_from_base(base) {
        cn = Some(cn_val);
    }

    // 提取 OU
    if let Some(ou_val) = extract_ou_from_base(base) {
        ou = Some(ou_val);
    }

    // 提取 DC（通过检查是否包含配置的 base_dn）
    if base.to_lowercase().contains(&config.base_dn.to_lowercase()) {
        dc = Some(config.dc.clone());
    }

    (cn, ou, dc)
}

pub fn get_base_dn_level(base: &str, config: &IamLdapConfig) -> Option<LdapBaseDnLevel> {
    if base.is_empty() {
        return Some(LdapBaseDnLevel::Domain);
    }

    let (cn, ou, _) = parse_base_dn_components(base, config);

    if let Some(cn_val) = cn {
        // Item 中第一个值存放的是 ou，第二个值存放的是 cn
        if let Some(ou_val) = ou {
            Some(LdapBaseDnLevel::Item(ou_val, cn_val))
        } else {
            // 如果没有 ou，使用空字符串作为 ou
            Some(LdapBaseDnLevel::Item(String::new(), cn_val))
        }
    } else if let Some(ou_val) = ou {
        // Ou 枚举值中增加了 String 用于存放实际的 ou 值
        Some(LdapBaseDnLevel::Ou(ou_val))
    } else if base.to_lowercase().contains(&config.base_dn.to_lowercase()) {
        Some(LdapBaseDnLevel::Domain)
    } else {
        None
    }
}

/// 检查 LdapSearchResultEntry 是否匹配 LdapQueryType
///
/// 该函数用于在内存中对搜索结果进行过滤，检查条目是否满足查询条件
pub fn entry_matches_query(entry: &LdapSearchResultEntry, query: &LdapQueryType) -> bool {
    match query {
        LdapQueryType::Equality { attribute, value } => get_attribute_values(entry, attribute).iter().any(|v| v.eq_ignore_ascii_case(value)),
        LdapQueryType::Present { attribute } => entry.attributes.iter().any(|attr| attr.atype.eq_ignore_ascii_case(attribute)),
        LdapQueryType::And { filters } => filters.iter().all(|filter| entry_matches_query(entry, filter)),
        LdapQueryType::Or { filters } => filters.iter().any(|filter| entry_matches_query(entry, filter)),
        LdapQueryType::Not { filter } => !entry_matches_query(entry, filter),
        LdapQueryType::Substring { attribute, substrings } => match_substring(entry, attribute, substrings),
        LdapQueryType::GreaterOrEqual { attribute, value } => get_attribute_values(entry, attribute).iter().any(|v| compare_values(v, value) >= 0),
        LdapQueryType::LessOrEqual { attribute, value } => get_attribute_values(entry, attribute).iter().any(|v| compare_values(v, value) <= 0),
        LdapQueryType::ApproxMatch { attribute, value } => {
            // 近似匹配通常使用不区分大小写的比较
            get_attribute_values(entry, attribute).iter().any(|v| v.eq_ignore_ascii_case(value))
        }
    }
}

/// 从 LdapSearchResultEntry 中获取指定属性的所有值
fn get_attribute_values(entry: &LdapSearchResultEntry, attribute: &str) -> Vec<String> {
    entry
        .attributes
        .iter()
        .filter(|attr| attr.atype.eq_ignore_ascii_case(attribute))
        .flat_map(|attr| {
            attr.vals.iter().map(|val| {
                // 将字节值转换为字符串
                // val 是 Vec<u8>，使用 as_slice() 或 &val[..] 都可以
                String::from_utf8_lossy(&val[..]).to_string()
            })
        })
        .collect()
}

/// 匹配子串过滤器
fn match_substring(entry: &LdapSearchResultEntry, attribute: &str, substrings: &LdapSubstringFilter) -> bool {
    let values = get_attribute_values(entry, attribute);

    // 如果没有任何值，不匹配
    if values.is_empty() {
        return false;
    }

    // 如果所有子串部分都为空，匹配所有值
    if substrings.initial.is_none() && substrings.any.is_empty() && substrings.final_.is_none() {
        return true;
    }

    // 检查每个值是否匹配子串模式
    values.iter().any(|value| {
        let value_lower = value.to_lowercase();

        // 检查初始子串
        if let Some(initial) = &substrings.initial {
            if !value_lower.starts_with(&initial.to_lowercase()) {
                return false;
            }
        }

        // 检查中间任意子串
        let mut remaining_value = if let Some(initial) = &substrings.initial {
            if value_lower.len() < initial.len() {
                return false;
            }
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

        // 检查结尾子串
        if let Some(final_part) = &substrings.final_ {
            if !remaining_value.to_lowercase().ends_with(&final_part.to_lowercase()) {
                return false;
            }
        }

        true
    })
}

/// 比较两个值（用于 GreaterOrEqual 和 LessOrEqual）
/// 返回：负数表示 v1 < v2，0 表示 v1 == v2，正数表示 v1 > v2
fn compare_values(v1: &str, v2: &str) -> i32 {
    // 首先尝试数值比较
    if let (Ok(n1), Ok(n2)) = (v1.parse::<f64>(), v2.parse::<f64>()) {
        return (n1 - n2).signum() as i32;
    }

    // 如果无法解析为数值，使用字符串比较（不区分大小写）
    v1.to_lowercase().cmp(&v2.to_lowercase()) as i32
}

/// 使用 LdapQueryType 筛选 LdapSearchResultEntry 列表
///
/// 返回所有匹配查询条件的条目
pub fn filter_entries_by_query(entries: &[LdapSearchResultEntry], query: &LdapQueryType) -> Vec<LdapSearchResultEntry> {
    entries.iter().filter(|entry| entry_matches_query(entry, query)).cloned().collect()
}
