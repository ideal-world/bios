//! LDAP App Result Builder
//!
//! 负责组装LDAP搜索响应结果，将IAM应用数据转换为LDAP协议格式

use ldap3_proto::simple::*;

use crate::iam_config::IamLdapConfig;
use crate::integration::ldap::ldap_parser::{extract_cn_from_base, LdapBaseDnLevel, LdapSearchQuery};

/// LDAP属性构建所需的应用字段
///
/// 该结构体包含了构建LDAP属性所需的所有应用字段，
/// 用于从 `LdapAppFields` 中提取必要信息。
#[derive(Debug, Clone)]
pub struct LdapAppFields {
    /// 应用ID（作为CN的fallback）
    pub id: String,
    /// 应用名称
    pub businessCategory: String,
    /// 排序
    pub sort: i64,
    /// 关联手机号
    pub phones: Vec<String>,
}

/// 构建LDAP应用搜索响应
pub fn build_app_search_response(req: &SearchRequest, query: &LdapSearchQuery, apps: Vec<LdapAppFields>, config: &IamLdapConfig) -> Vec<LdapMsg> {
    let mut results = Vec::new();

    // 如果没有应用，返回空结果
    if apps.is_empty() {
        results.push(req.gen_success());
        return results;
    }

    // 为每个应用构建LDAP条目
    for app in apps {
        // 从应用信息中提取CN（使用name或sys_code）
        let cn = extract_cn_from_app(&app, &query.base, config);

        // 构建LDAP属性
        let all_attributes = build_ldap_attributes(&app, config);

        // 根据请求的属性列表过滤属性
        let attributes = filter_attributes_by_request(&all_attributes, &query.attributes);

        // 创建结果条目（应用使用 ou=config.ou_app）
        results.push(req.gen_result_entry(LdapSearchResultEntry {
            dn: format!("cn={},ou={},{}", cn, config.ou_app, config.base_dn),
            attributes,
        }));
    }

    // 添加成功响应
    results.push(req.gen_success());
    results
}

/// 从应用信息中提取CN
fn extract_cn_from_app(app: &LdapAppFields, base: &str, _config: &IamLdapConfig) -> String {
    // 优先从base DN中提取CN
    if let Some(cn) = extract_cn_from_base(base) {
        return cn;
    }

    app.id.clone()
}

/// 根据请求的属性列表过滤属性
fn filter_attributes_by_request(all_attributes: &[LdapPartialAttribute], requested_attrs: &[String]) -> Vec<LdapPartialAttribute> {
    // 如果请求列表为空或包含"*"，返回所有属性
    if requested_attrs.is_empty() || requested_attrs.iter().any(|attr| attr == "*") {
        return all_attributes.to_vec();
    }

    // 如果请求了"+*"，返回所有属性（操作属性当前未实现）
    if requested_attrs.iter().any(|attr| attr == "+*") {
        return all_attributes.to_vec();
    }

    // 只返回请求的属性（不区分大小写）
    let requested_lower: Vec<String> = requested_attrs.iter().map(|a| a.to_lowercase()).collect();
    all_attributes.iter().filter(|attr| requested_lower.contains(&attr.atype.to_lowercase())).cloned().collect()
}

/// 构建LDAP属性列表
fn build_ldap_attributes(app: &LdapAppFields, config: &IamLdapConfig) -> Vec<LdapPartialAttribute> {
    // 使用name作为CN，如果没有则使用sys_code
    let cn = app.id.clone();

    // 构建属性列表
    let attributes = vec![
        LdapPartialAttribute {
            atype: "cn".to_string(),
            vals: vec![cn.clone().into()],
        },
        LdapPartialAttribute {
            atype: "ou".to_string(),
            vals: vec![config.ou_app.clone().into(), app.id.clone().into()],
        },
        LdapPartialAttribute {
            atype: "objectClass".to_string(),
            vals: vec!["groupOfUniqueNames".into(), "top".into()],
        },
        LdapPartialAttribute {
            atype: "businessCategory".to_string(),
            vals: vec![app.businessCategory.clone().into()],
        },
    ];

    attributes
}

// 判断search时是否返回应用节点
pub fn should_return_app_level_in_search(level: LdapBaseDnLevel, scope: LdapSearchScope, config: &IamLdapConfig) -> bool {
    match level {
        LdapBaseDnLevel::Domain => matches!(scope, LdapSearchScope::Subtree) || matches!(scope, LdapSearchScope::Children),
        LdapBaseDnLevel::Ou(ref ou) => {
            ou.to_lowercase() == config.ou_app.to_lowercase()
                && (matches!(scope, LdapSearchScope::OneLevel) || matches!(scope, LdapSearchScope::Subtree) || matches!(scope, LdapSearchScope::Children))
        }
        _ => false,
    }
}
