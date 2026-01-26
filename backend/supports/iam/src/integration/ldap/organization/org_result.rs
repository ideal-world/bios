//! LDAP Organization Result Builder
//!
//! 负责组装LDAP搜索响应结果，将IAM组织数据转换为LDAP协议格式

use ldap3_proto::simple::*;

use crate::iam_config::IamLdapConfig;
use crate::integration::ldap::ldap_parser::{extract_cn_from_base, LdapBaseDnLevel, LdapSearchQuery};
use bios_basic::rbum::dto::rbum_set_dto::RbumSetTreeNodeResp;

/// 构建LDAP组织搜索响应
pub fn build_org_search_response(req: &SearchRequest, query: &LdapSearchQuery, orgs: Vec<RbumSetTreeNodeResp>, config: &IamLdapConfig) -> Vec<LdapMsg> {
    let mut results = Vec::new();

    // 如果没有组织，返回空结果
    if orgs.is_empty() {
        results.push(req.gen_success());
        return results;
    }

    // 为每个组织构建LDAP条目
    for org in orgs {
        // 从组织信息中提取CN（使用name或sys_code）
        let cn = extract_cn_from_org(&org, &query.base, config);

        // 构建LDAP属性
        let all_attributes = build_ldap_attributes(&org, config);

        // 根据请求的属性列表过滤属性
        let attributes = filter_attributes_by_request(&all_attributes, &query.attributes);

        // 创建结果条目（组织使用 ou=config.ou_organization）
        results.push(req.gen_result_entry(LdapSearchResultEntry {
            dn: format!("cn={},ou={},{}", cn, config.ou_organization, config.base_dn),
            attributes,
        }));
    }

    // 添加成功响应
    results.push(req.gen_success());
    results
}

/// 从组织信息中提取CN
fn extract_cn_from_org(org: &RbumSetTreeNodeResp, base: &str, _config: &IamLdapConfig) -> String {
    // 优先从base DN中提取CN
    if let Some(cn) = extract_cn_from_base(base) {
        return cn;
    }

    // 优先使用name，如果没有则使用sys_code
    if !org.name.is_empty() {
        return org.name.clone();
    }

    if !org.sys_code.is_empty() {
        return org.sys_code.clone();
    }

    // 最后使用ID
    org.id.clone()
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
fn build_ldap_attributes(org: &RbumSetTreeNodeResp, config: &IamLdapConfig) -> Vec<LdapPartialAttribute> {
    // 使用name作为CN，如果没有则使用sys_code
    let cn = if !org.name.is_empty() { org.name.clone() } else { org.sys_code.clone() };

    // 构建属性列表
    let mut attributes = vec![
        LdapPartialAttribute {
            atype: "cn".to_string(),
            vals: vec![cn.clone().into()],
        },
        LdapPartialAttribute {
            atype: "ou".to_string(),
            vals: vec![config.ou_organization.clone().into()],
        },
        LdapPartialAttribute {
            atype: "objectClass".to_string(),
            vals: vec!["organizationalUnit".into(), "top".into()],
        },
        LdapPartialAttribute {
            atype: "name".to_string(),
            vals: vec![org.name.clone().into()],
        },
        LdapPartialAttribute {
            atype: "sysCode".to_string(),
            vals: vec![org.sys_code.clone().into()],
        },
        LdapPartialAttribute {
            atype: "description".to_string(),
            vals: vec![org.ext.clone().into()],
        },
    ];

    // 添加业务编码（如果有）
    if !org.bus_code.is_empty() {
        attributes.push(LdapPartialAttribute {
            atype: "busCode".to_string(),
            vals: vec![org.bus_code.clone().into()],
        });
    }

    // 添加图标（如果有）
    if !org.icon.is_empty() {
        attributes.push(LdapPartialAttribute {
            atype: "icon".to_string(),
            vals: vec![org.icon.clone().into()],
        });
    }

    // 添加父节点ID（如果有）
    if let Some(pid) = &org.pid {
        attributes.push(LdapPartialAttribute {
            atype: "parentId".to_string(),
            vals: vec![pid.clone().into()],
        });
    }

    // 添加关联对象ID（如果有）
    if let Some(rel) = &org.rel {
        attributes.push(LdapPartialAttribute {
            atype: "relId".to_string(),
            vals: vec![rel.clone().into()],
        });
    }

    attributes
}

// 判断search时是否返回组织节点
pub fn should_return_org_level_in_search(level: LdapBaseDnLevel, scope: LdapSearchScope, config: &IamLdapConfig) -> bool {
    match level {
        LdapBaseDnLevel::Domain => matches!(scope, LdapSearchScope::Subtree) || matches!(scope, LdapSearchScope::Children),
        LdapBaseDnLevel::Ou(ref ou) => {
            ou.to_lowercase() == config.ou_organization.to_lowercase()
                && (matches!(scope, LdapSearchScope::OneLevel) || matches!(scope, LdapSearchScope::Subtree) || matches!(scope, LdapSearchScope::Children))
        }
        _ => false,
    }
}
