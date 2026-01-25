//! LDAP Organization Query Handler
//!
//! 负责与IAM数据交互，执行组织查询操作

use std::collections::HashMap;

use bios_basic::rbum::dto::rbum_filer_dto::{RbumBasicFilterReq, RbumSetTreeFilterReq};
use bios_basic::rbum::dto::rbum_set_dto::RbumSetTreeNodeResp;
use bios_basic::rbum::rbum_enumeration::RbumSetCateLevelQueryKind;
use lazy_static::lazy_static;
use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::TardisFunsInst;

use crate::iam_config::IamLdapConfig;
use crate::iam_constants;
use crate::iam_enumeration::IamSetKind;
use crate::integration::ldap::ldap_parser;
use crate::basic::serv::iam_cert_serv::IamCertServ;
use crate::basic::serv::iam_set_serv::IamSetServ;

// LDAP 属性名 -> 组织字段 映射
lazy_static! {
    static ref LDAP_ATTR_TO_ORG_FIELD: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        m.insert("cn", "name");
        m.insert("ou", "name");
        m.insert("name", "name");
        m.insert("syscode", "sys_code");
        m.insert("sys_code", "sys_code");
        m.insert("buscode", "bus_code");
        m.insert("bus_code", "bus_code");
        m.insert("description", "ext");
        m.insert("ext", "ext");
        m
    };
}

/// 执行LDAP组织搜索查询
pub async fn execute_ldap_org_search(
    query: &ldap_parser::LdapSearchQuery,
    config: &IamLdapConfig,
) -> TardisResult<Vec<RbumSetTreeNodeResp>> {
    let funs = iam_constants::get_tardis_inst();
    let ctx = TardisContext::default();

    // 获取组织Set ID
    let set_id = IamSetServ::get_default_set_id_by_ctx(&IamSetKind::Org, &funs, &ctx).await?;

    // 构建查询过滤器
    let mut filter = RbumSetTreeFilterReq {
        fetch_cate_item: false,
        hide_item_with_disabled: true,
        ..Default::default()
    };

    // 根据LDAP查询条件设置过滤器
    apply_ldap_query_to_filter(&query.query_type, &mut filter, config)?;

    // 查询组织树
    let tree_result = IamSetServ::get_tree(&set_id, &mut filter, &funs, &ctx).await?;

    // 过滤结果
    let mut orgs = tree_result.main;
    
    // 如果查询条件不是全量查询，需要进一步过滤
    if !ldap_parser::is_full_query(query) {
        orgs = filter_orgs_by_query(&orgs, &query.query_type, config)?;
    }

    Ok(orgs)
}

/// 将LDAP查询条件应用到组织树过滤器
fn apply_ldap_query_to_filter(
    query_type: &ldap_parser::LdapQueryType,
    filter: &mut RbumSetTreeFilterReq,
    _config: &IamLdapConfig,
) -> TardisResult<()> {
    match query_type {
        ldap_parser::LdapQueryType::Equality { attribute, value } => {
            // 如果是 sys_code 查询，可以优化查询
            if attribute.to_lowercase() == "syscode" || attribute.to_lowercase() == "sys_code" {
                filter.sys_codes = Some(vec![value.clone()]);
                filter.sys_code_query_kind = Some(RbumSetCateLevelQueryKind::CurrentAndSub);
            }
        }
        ldap_parser::LdapQueryType::Substring { attribute, substrings } => {
            // 如果是 sys_code 子串查询，可以优化查询
            if attribute.to_lowercase() == "syscode" || attribute.to_lowercase() == "sys_code" {
                if let Some(initial) = &substrings.initial {
                    filter.sys_codes = Some(vec![initial.clone()]);
                    filter.sys_code_query_kind = Some(RbumSetCateLevelQueryKind::Sub);
                }
            }
        }
        _ => {
            // 其他查询类型，使用全量查询然后过滤
        }
    }
    Ok(())
}

/// 根据LDAP查询条件过滤组织列表
fn filter_orgs_by_query(
    orgs: &[RbumSetTreeNodeResp],
    query_type: &ldap_parser::LdapQueryType,
    _config: &IamLdapConfig,
) -> TardisResult<Vec<RbumSetTreeNodeResp>> {
    let mut filtered = Vec::new();

    for org in orgs {
        if matches_ldap_query(org, query_type, _config)? {
            filtered.push(org.clone());
        }
    }

    Ok(filtered)
}

/// 检查组织是否匹配LDAP查询条件
fn matches_ldap_query(
    org: &RbumSetTreeNodeResp,
    query_type: &ldap_parser::LdapQueryType,
    _config: &IamLdapConfig,
) -> TardisResult<bool> {
    match query_type {
        ldap_parser::LdapQueryType::Equality { attribute, value } => {
            let field = get_org_field(attribute)?;
            let org_value = get_org_field_value(org, field);
            Ok(org_value.to_lowercase() == value.to_lowercase())
        }
        ldap_parser::LdapQueryType::Present { attribute } => {
            let field = get_org_field(attribute)?;
            let org_value = get_org_field_value(org, field);
            Ok(!org_value.is_empty())
        }
        ldap_parser::LdapQueryType::Substring { attribute, substrings } => {
            let field = get_org_field(attribute)?;
            let org_value = get_org_field_value(org, field).to_lowercase();
            
            let mut matches = true;
            if let Some(initial) = &substrings.initial {
                if !org_value.starts_with(&initial.to_lowercase()) {
                    matches = false;
                }
            }
            for any_part in &substrings.any {
                if !org_value.contains(&any_part.to_lowercase()) {
                    matches = false;
                }
            }
            if let Some(final_part) = &substrings.final_ {
                if !org_value.ends_with(&final_part.to_lowercase()) {
                    matches = false;
                }
            }
            Ok(matches)
        }
        ldap_parser::LdapQueryType::And { filters } => {
            for filter in filters {
                if !matches_ldap_query(org, filter, _config)? {
                    return Ok(false);
                }
            }
            Ok(true)
        }
        ldap_parser::LdapQueryType::Or { filters } => {
            for filter in filters {
                if matches_ldap_query(org, filter, _config)? {
                    return Ok(true);
                }
            }
            Ok(false)
        }
        ldap_parser::LdapQueryType::Not { filter } => {
            Ok(!matches_ldap_query(org, filter, _config)?)
        }
        _ => Ok(true), // 其他查询类型默认匹配
    }
}

/// 根据 LDAP 属性名获取对应的组织字段
fn get_org_field(attr: &str) -> TardisResult<&'static str> {
    LDAP_ATTR_TO_ORG_FIELD
        .get(attr.to_lowercase().as_str())
        .copied()
        .ok_or_else(|| {
            tardis::basic::error::TardisError::format_error(
                &format!("Unsupported LDAP attribute for organization: {}", attr),
                "406-iam-ldap-unsupported-org-attribute",
            )
        })
}

/// 获取组织的字段值
fn get_org_field_value(org: &RbumSetTreeNodeResp, field: &str) -> String {
    match field {
        "name" => org.name.clone(),
        "sys_code" => org.sys_code.clone(),
        "bus_code" => org.bus_code.clone(),
        "ext" => org.ext.clone(),
        _ => String::new(),
    }
}
