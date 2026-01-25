//! LDAP Account Result Builder
//!
//! 负责组装LDAP搜索响应结果，将IAM账户数据转换为LDAP协议格式

use ldap3_proto::simple::*;

use crate::iam_config::IamLdapConfig;
use crate::integration::ldap::ldap_parser::{LdapBaseDnLevel, LdapSearchQuery};

/// LDAP属性构建所需的账户字段
/// 
/// 该结构体包含了构建LDAP属性所需的所有账户字段，
/// 用于从 `IamAccountDetailAggResp` 中提取必要信息。
#[derive(Debug, Clone)]
pub struct LdapAccountFields {
    /// 账户ID（作为账户名的fallback）
    pub id: String,
    /// 账户名称
    pub name: String,
    /// 员工编号
    pub employee_code: String,
    /// 劳动类型代码
    pub labor_type: String,
    /// 证书信息映射
    /// - "UserPwd": 账户名
    /// - "MailVCode": 邮箱
    /// - "PhoneVCode": 手机号（可选）
    pub user_pwd: String,
    pub phone: Option<String>,

    pub mail: Option<String>,
    /// 扩展属性列表（用于查找职位信息，查找 name == "primary" 的项）
    pub primary_code: Option<String>,
}

/// 构建LDAP账户搜索响应
pub fn build_account_search_response(
    req: &SearchRequest,
    query: &LdapSearchQuery,
    accounts: Vec<LdapAccountFields>,
    specified_cn: Option<String>,
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
        let cn = account.user_pwd.clone();

        // 构建LDAP属性
        let all_attributes = build_ldap_attributes(&account, config);

        // 根据请求的属性列表过滤属性
        let attributes = filter_attributes_by_request(&all_attributes, &query.attributes);

        if let Some(specified_cn) = specified_cn.clone() {
            if specified_cn == cn {
                results.push(req.gen_result_entry(LdapSearchResultEntry {
                    dn: format!("cn={},ou={},{}", cn, config.ou_staff, config.base_dn),
                    attributes,
                }));
            }
        } else {
            // 创建结果条目（账号使用 ou=config.ou_staff）
            results.push(req.gen_result_entry(LdapSearchResultEntry {
                dn: format!("cn={},ou={},{}", cn, config.ou_staff, config.base_dn),
                attributes,
            }));
        }
    }
    results
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
fn build_ldap_attributes(fields: &LdapAccountFields, config: &IamLdapConfig) -> Vec<LdapPartialAttribute> {
    // 将账户信息转换为LDAP字段结构
    build_ldap_attributes_from_fields(&fields, config)
}

/// 从LDAP账户字段构建LDAP属性列表
fn build_ldap_attributes_from_fields(fields: &LdapAccountFields, config: &IamLdapConfig) -> Vec<LdapPartialAttribute> {
    // 获取劳动类型标签
    let labor_type_label = get_labor_type_label(&fields.labor_type, config);

    // 获取职位标签
    let primary_label = fields
        .primary_code
        .clone().unwrap_or_default();
    let primary_label = get_position_label(&primary_label, config);

    // 提取账户名（CN）
    let cn = fields
        .user_pwd
        .clone();

    // 提取邮箱
    let mail = fields
        .mail
        .clone().unwrap_or_default();

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
            vals: vec![config.ou_staff.clone().into()],
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
            atype: "hasSubordinates".to_string(),
            vals: vec!["FALSE".into()],
        },
        LdapPartialAttribute {
            atype: "employeeNumber".to_string(),
            vals: vec![fields.employee_code.clone().into()],
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
            vals: vec![fields.name.clone().into()],
        },
        LdapPartialAttribute {
            atype: "displayName".to_string(),
            vals: vec![fields.name.clone().into()],
        },
        LdapPartialAttribute {
            atype: "sn".to_string(),
            vals: vec![fields.name.clone().into()],
        },
    ];

    // 添加手机号（如果有）
    if let Some(phone) = &fields.phone {
        attributes.push(LdapPartialAttribute {
            atype: "mobile".to_string(),
            vals: vec![phone.clone().into()],
        });
    }

    attributes
}

// 判断search时是否返回账号节点
pub fn should_return_account_level_in_search(level: LdapBaseDnLevel, scope: LdapSearchScope, config: &IamLdapConfig) -> bool {
    match level {
        LdapBaseDnLevel::Domain => matches!(scope, LdapSearchScope::Subtree) || matches!(scope, LdapSearchScope::Children),
        LdapBaseDnLevel::Ou(ref ou) => ou.to_lowercase() == config.ou_staff.to_lowercase() && (matches!(scope, LdapSearchScope::OneLevel) || matches!(scope, LdapSearchScope::Subtree) || matches!(scope, LdapSearchScope::Children)),
        _ => false,
    }
}