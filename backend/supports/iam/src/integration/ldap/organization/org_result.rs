//! LDAP Organization Result Builder
//!
//! 负责组装LDAP搜索响应结果，将IAM组织数据转换为LDAP协议格式

use ldap3_proto::simple::*;

use bios_basic::rbum::dto::rbum_set_dto::RbumSetTreeNodeResp;
use crate::iam_config::IamLdapConfig;
use crate::integration::ldap::ldap_parser::{extract_cn_from_base, is_root_dse_query, LdapSearchQuery};

/// 构建LDAP组织搜索响应
pub fn build_org_search_response(
    req: &SearchRequest,
    query: &LdapSearchQuery,
    orgs: Vec<RbumSetTreeNodeResp>,
    config: &IamLdapConfig,
) -> Vec<LdapMsg> {
    let mut results = Vec::new();

    // 处理根DSE查询
    if is_root_dse_query(query) {
        let root_dse_attributes = build_root_dse_attributes(config, &query.attributes);
        results.push(req.gen_result_entry(LdapSearchResultEntry {
            dn: format!("DC={}", config.dc),
            attributes: root_dse_attributes,
        }));
        results.push(req.gen_success());
        return results;
    }

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

        // 创建结果条目（组织使用 ou=organizations）
        results.push(req.gen_result_entry(LdapSearchResultEntry {
            dn: format!("cn={},ou=organizations,dc={}", cn, config.dc),
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
fn build_ldap_attributes(org: &RbumSetTreeNodeResp, _config: &IamLdapConfig) -> Vec<LdapPartialAttribute> {
    // 使用name作为CN，如果没有则使用sys_code
    let cn = if !org.name.is_empty() {
        org.name.clone()
    } else {
        org.sys_code.clone()
    };

    // 构建属性列表
    let mut attributes = vec![
        LdapPartialAttribute {
            atype: "cn".to_string(),
            vals: vec![cn.clone().into()],
        },
        LdapPartialAttribute {
            atype: "ou".to_string(),
            vals: vec!["organizations".to_string().into()],
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

/// 构建 RootDSE 属性
/// RootDSE (Root Directory Service Entry) 是 LDAP 服务器的根入口点
/// 它包含服务器的能力信息和配置信息
fn build_root_dse_attributes(config: &IamLdapConfig, requested_attrs: &[String]) -> Vec<LdapPartialAttribute> {
    let base_dn = format!("DC={}", config.dc);
    
    // 构建所有可用的 RootDSE 属性
    let mut all_attributes = vec![
        // namingContexts: 命名上下文（base DN）
        LdapPartialAttribute {
            atype: "namingContexts".to_string(),
            vals: vec![base_dn.clone().into()],
        },
        // subschemaSubentry: Schema 子条目位置（Apache Directory Studio 需要此属性）
        LdapPartialAttribute {
            atype: "subschemaSubentry".to_string(),
            vals: vec![format!("cn=schema,{}", base_dn).into()],
        },
        // supportedLDAPVersion: 支持的 LDAP 版本
        LdapPartialAttribute {
            atype: "supportedLDAPVersion".to_string(),
            vals: vec!["3".to_string().into()],
        },
        // supportedControl: 支持的控件（可选）
        LdapPartialAttribute {
            atype: "supportedControl".to_string(),
            vals: vec!["1.2.840.113556.1.4.319".to_string().into()], // Paged results control
        },
        // supportedExtension: 支持的扩展（可选）
        LdapPartialAttribute {
            atype: "supportedExtension".to_string(),
            vals: vec!["1.3.6.1.4.1.4203.1.11.3".to_string().into()], // Who am I extension
        },
        // supportedSASLMechanisms: 支持的 SASL 机制
        LdapPartialAttribute {
            atype: "supportedSASLMechanisms".to_string(),
            vals: vec!["PLAIN".to_string().into()],
        },
        // vendorName: 供应商名称
        LdapPartialAttribute {
            atype: "vendorName".to_string(),
            vals: vec!["BIOS".to_string().into()],
        },
        // vendorVersion: 供应商版本
        LdapPartialAttribute {
            atype: "vendorVersion".to_string(),
            vals: vec!["1.0".to_string().into()],
        },
    ];
    
    // 根据请求的属性列表过滤属性
    filter_attributes_by_request(&all_attributes, requested_attrs)
}
