//! LDAP Account Result Builder
//!
//! 负责组装LDAP搜索响应结果，将IAM账户数据转换为LDAP协议格式

use ldap3_proto::simple::*;

use crate::basic::dto::iam_account_dto::IamAccountDetailAggResp;
use crate::iam_config::IamLdapConfig;
use crate::integration::ldap::ldap_parser::{extract_cn_from_base, LdapSearchQuery};

/// 构建LDAP账户搜索响应
pub fn build_account_search_response(
    req: &SearchRequest,
    query: &LdapSearchQuery,
    accounts: Vec<IamAccountDetailAggResp>,
    config: &IamLdapConfig,
) -> Vec<LdapMsg> {
    let mut results = Vec::new();

    // 如果没有账户，返回空结果
    if accounts.is_empty() {
        results.push(req.gen_success());
        return results;
    }

    // 为每个账户构建LDAP条目
    for account in accounts {
        // 从账户信息中提取CN
        let cn = extract_cn_from_account(&account, &query.base, config);

        // 构建LDAP属性
        let all_attributes = build_ldap_attributes(&account, config);

        // 根据请求的属性列表过滤属性
        let attributes = filter_attributes_by_request(&all_attributes, &query.attributes);

        // 创建结果条目（账号使用 ou=staff）
        results.push(req.gen_result_entry(LdapSearchResultEntry {
            dn: format!("cn={},ou=staff,dc={}", cn, config.dc),
            attributes,
        }));
    }

    // 添加成功响应
    results.push(req.gen_success());
    results
}

/// 构建简化结果（用于Present过滤器等简单查询）
pub fn build_simple_account_result(
    req: &SearchRequest,
    cn: &str,
    config: &IamLdapConfig,
) -> Vec<LdapMsg> {
    vec![
        req.gen_result_entry(LdapSearchResultEntry {
            dn: format!("CN={},OU=staff,DC={}", cn, config.dc),
            attributes: vec![
                LdapPartialAttribute {
                    atype: "sAMAccountName".to_string(),
                    vals: vec![cn.to_string().into()],
                },
                LdapPartialAttribute {
                    atype: "mail".to_string(),
                    vals: vec![format!("{}@example.com", cn).into()],
                },
                LdapPartialAttribute {
                    atype: "cn".to_string(),
                    vals: vec![cn.to_string().into()],
                },
                LdapPartialAttribute {
                    atype: "givenName".to_string(),
                    vals: vec!["".to_string().into()],
                },
                LdapPartialAttribute {
                    atype: "sn".to_string(),
                    vals: vec![cn.to_string().into()],
                },
            ],
        }),
        req.gen_success(),
    ]
}

/// 从账户信息中提取CN（账户名）
fn extract_cn_from_account(account: &IamAccountDetailAggResp, base: &str, _config: &IamLdapConfig) -> String {
    // 优先从base DN中提取CN
    if let Some(cn) = extract_cn_from_base(base) {
        return cn;
    }

    // 如果base中没有CN，尝试从账户凭证中获取账户名
    // 查找UserPwd类型的凭证
    if let Some(account_name) = account.certs.get("UserPwd") {
        return account_name.clone();
    }

    // 如果都没有，使用员工编号或ID作为fallback
    if !account.employee_code.is_empty() {
        return account.employee_code.clone();
    }

    // 最后使用账户ID（不推荐，但作为最后的fallback）
    account.id.clone()
}

/// 获取劳动类型标签
fn get_labor_type_label(labor_type_code: &str, config: &IamLdapConfig) -> String {
    if labor_type_code.is_empty() {
        return String::new();
    }
    if let Some(ref labor_type_map) = config.labor_type_map {
        labor_type_map
            .get(labor_type_code)
            .cloned()
            .unwrap_or_else(|| labor_type_code.to_string())
    } else {
        labor_type_code.to_string()
    }
}

/// 获取职位标签
fn get_position_label(position_code: &str, config: &IamLdapConfig) -> String {
    if position_code.is_empty() {
        return String::new();
    }
    if let Some(ref position_map) = config.position_map {
        position_map
            .get(position_code)
            .cloned()
            .unwrap_or_else(|| position_code.to_string())
    } else {
        position_code.to_string()
    }
}

/// 根据请求的属性列表过滤属性
/// 根据LDAP协议：
/// - 如果请求列表为空或包含"*"，返回所有用户属性
/// - 如果请求了"+*"，返回所有操作属性（当前实现不返回操作属性）
/// - 否则只返回请求的属性
fn filter_attributes_by_request(
    all_attributes: &[LdapPartialAttribute],
    requested_attrs: &[String],
) -> Vec<LdapPartialAttribute> {
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
    all_attributes
        .iter()
        .filter(|attr| requested_lower.contains(&attr.atype.to_lowercase()))
        .cloned()
        .collect()
}

/// 构建LDAP属性列表
fn build_ldap_attributes(account: &IamAccountDetailAggResp, config: &IamLdapConfig) -> Vec<LdapPartialAttribute> {
    // 获取劳动类型标签
    let labor_type_label = get_labor_type_label(&account.labor_type, config);

    // 获取职位标签
    let primary_code = account
        .exts
        .iter()
        .find(|ext| ext.name == "primary")
        .map(|ext| ext.value.clone())
        .unwrap_or_default();
    let primary_label = get_position_label(&primary_code, config);

    // 提取账户名（CN）
    let cn = account
        .certs
        .get("UserPwd")
        .cloned()
        .unwrap_or_else(|| {
            if !account.employee_code.is_empty() {
                account.employee_code.clone()
            } else {
                account.id.clone()
            }
        });

    // 提取邮箱
    let mail = account
        .certs
        .get("MailVCode")
        .cloned()
        .unwrap_or_default();

    // 构建属性列表
    let mut attributes = vec![
        LdapPartialAttribute {
            atype: "uid".to_string(),
            vals: vec![cn.clone().into()],
        },
        LdapPartialAttribute {
            atype: "cn".to_string(),
            vals: vec![cn.clone().into()],
        },
        LdapPartialAttribute {
            atype: "sAMAccountName".to_string(),
            vals: vec![cn.clone().into()],
        },
        LdapPartialAttribute {
            atype: "employeeType".to_string(),
            vals: vec![labor_type_label.clone().into()],
        },
        LdapPartialAttribute {
            atype: "ou".to_string(),
            vals: vec!["staff".to_string().into()],
        },
        LdapPartialAttribute {
            atype: "objectClass".to_string(),
            vals: vec!["inetOrgPerson".into(), "uidObject".into(), "top".into()],
        },
        LdapPartialAttribute {
            atype: "mail".to_string(),
            vals: vec![mail.clone().into()],
        },
        LdapPartialAttribute {
            atype: "employeeNumber".to_string(),
            vals: vec![account.employee_code.clone().into()],
        },
        LdapPartialAttribute {
            atype: "title".to_string(),
            vals: vec![primary_label.clone().into()],
        },
        LdapPartialAttribute {
            atype: "businessCategory".to_string(),
            vals: vec![labor_type_label.clone().into()],
        },
        LdapPartialAttribute {
            atype: "givenName".to_string(),
            vals: vec![account.name.clone().into()],
        },
        LdapPartialAttribute {
            atype: "displayName".to_string(),
            vals: vec![account.name.clone().into()],
        },
        LdapPartialAttribute {
            atype: "sn".to_string(),
            vals: vec![account.name.clone().into()],
        },
    ];

    // 添加手机号（如果有）
    if let Some(phone) = account.certs.get("PhoneVCode") {
        attributes.push(LdapPartialAttribute {
            atype: "mobile".to_string(),
            vals: vec![phone.clone().into()],
        });
    }

    attributes
}
