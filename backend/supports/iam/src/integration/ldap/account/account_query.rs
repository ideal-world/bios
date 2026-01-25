//! LDAP Account Query Handler
//!
//! 负责与IAM数据交互，执行账户查询操作

use std::collections::HashMap;

use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemAttrServ;
use bios_basic::rbum::serv::rbum_kind_serv::RbumKindAttrServ;
use lazy_static::lazy_static;
use ldap3_proto::proto::LdapSubstringFilter;
use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::TardisFunsInst;

use bios_basic::rbum::dto::rbum_filer_dto::{RbumBasicFilterReq, RbumKindAttrFilterReq};

// LDAP 属性名 -> 数据库查询字段 映射，用于根据传入的 attr 拼接 SQL 条件
lazy_static! {
    static ref LDAP_ATTR_TO_DB_FIELD: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        m.insert("cn", "user_pwd_cert.ak");
        m.insert("uid", "user_pwd_cert.ak");
        m.insert("samaccountname", "user_pwd_cert.ak");
        m.insert("mail", "mail_vcode_cert.ak");
        m.insert("employeenumber", "iam_account.employee_code");
        m.insert("displayname", "rbum_item.name");
        m.insert("givenname", "rbum_item.name");
        m.insert("sn", "rbum_item.name");
        m
    };
}

use crate::basic::dto::iam_account_dto::IamAccountDetailAggResp;
use crate::basic::dto::iam_filer_dto::IamAccountFilterReq;
use crate::basic::serv::iam_account_serv::IamAccountServ;
use crate::basic::serv::iam_cert_serv::IamCertServ;
use crate::iam_config::IamLdapConfig;
use crate::iam_constants;
use crate::iam_enumeration::IamCertKernelKind;
use crate::integration::ldap::account::account_result::LdapAccountFields;
use crate::integration::ldap::ldap_parser;

/// 执行LDAP账户搜索查询
pub async fn execute_ldap_account_search(
    query: &ldap_parser::LdapSearchQuery,
    config: &IamLdapConfig,
) -> TardisResult<Vec<LdapAccountFields>> {
    let funs = iam_constants::get_tardis_inst();

    // 处理简单存在性查询（从base DN提取CN，检查账户是否存在）
    if ldap_parser::is_simple_present_query(query) {
        if let Some(cn) = ldap_parser::extract_cn_from_base(&query.base) {
            match check_account_exists(&cn).await {
                Ok(true) => {
                    // 账户存在，返回账户详情
                    if let Ok(Some(account)) = get_account_by_cn(&cn).await {
                        return Ok(vec![LdapAccountFields {
                            id: account.id.clone(),
                            name: account.name.clone(),
                            employee_code: account.employee_code,
                            labor_type: account.labor_type.clone(),
                            user_pwd: account.certs.get(&IamCertKernelKind::UserPwd.to_string()).cloned().unwrap_or_default(),
                            phone: account.certs.get(&IamCertKernelKind::PhoneVCode.to_string()).cloned(),
                            mail: account.certs.get(&IamCertKernelKind::MailVCode.to_string()).cloned(),
                            primary_code: account.exts.iter().find(|e| e.name == "primary").map(|e| e.value.clone()),
                        }]);
                    }
                }
                Ok(false) => return Ok(vec![]),
                Err(e) => return Err(e),
            }
        }
        return Ok(vec![]);
    }

    // 使用原生SQL查询方式
    let ctx = TardisContext::default();

    // 根据query参数构建SQL查询，获取符合条件的account id列表
    let account_fields = build_and_execute_sql_query(query, config, &funs, &ctx).await?;

    Ok(account_fields)
}

/// 检查账户是否存在
pub async fn check_account_exists(ak: &str) -> TardisResult<bool> {
    let funs = iam_constants::get_tardis_inst();
    let rbum_cert_conf_id = IamCertServ::get_cert_conf_id_by_kind(
        &IamCertKernelKind::UserPwd.to_string(),
        Some("".to_string()),
        &funs,
    )
    .await?;
    bios_basic::rbum::serv::rbum_cert_serv::RbumCertServ::check_exist(ak, &rbum_cert_conf_id, "", &funs).await
}

/// 根据CN获取账户详情
pub async fn get_account_by_cn(ak: &str) -> TardisResult<Option<IamAccountDetailAggResp>> {
    let funs = iam_constants::get_tardis_inst();
    let rbum_cert_conf_id = IamCertServ::get_cert_conf_id_by_kind(
        &IamCertKernelKind::UserPwd.to_string(),
        Some("".to_string()),
        &funs,
    )
    .await?;

    let ctx = IamCertServ::try_use_tenant_ctx(Default::default(), Some("".to_string()))?;

    if let Some(cert) = bios_basic::rbum::serv::rbum_cert_serv::RbumCertServ::find_one_detail_rbum(
        &bios_basic::rbum::dto::rbum_filer_dto::RbumCertFilterReq {
            basic: RbumBasicFilterReq {
                own_paths: Some("".to_string()),
                ..Default::default()
            },
            ak: Some(ak.to_string()),
            rel_rbum_cert_conf_ids: Some(vec![rbum_cert_conf_id]),
            ..Default::default()
        },
        &funs,
        &ctx,
    )
    .await?
    {
        let account = IamAccountServ::get_account_detail_aggs(
            &cert.rel_rbum_id,
            &IamAccountFilterReq {
                basic: RbumBasicFilterReq {
                    own_paths: Some("".to_string()),
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                ..Default::default()
            },
            true,
            true,
            true,
            &funs,
            &ctx,
        )
        .await?;
        Ok(Some(account))
    } else {
        Ok(None)
    }
}

/// 根据LDAP查询参数构建SQL并执行，返回符合条件的account id列表
async fn build_and_execute_sql_query(
    query: &ldap_parser::LdapSearchQuery,
    config: &IamLdapConfig,
    funs: &TardisFunsInst,
    ctx: &TardisContext,
) -> TardisResult<Vec<LdapAccountFields>> {
    // 构建SQL WHERE条件
    let (join, where_clause) = if ldap_parser::is_full_query(query) {
        ("", "".to_string())
    } else {
        ("AND", build_sql_where_clause(&query.query_type, config)?)
    };
    // 构建完整的SQL查询语句
    let user_pwd_conf_id = IamCertServ::get_cert_conf_id_by_kind(
        &IamCertKernelKind::UserPwd.to_string(),
        Some("".to_string()),
        &funs,
    )
    .await?;
    let mail_vcode_conf_id = IamCertServ::get_cert_conf_id_by_kind(
        &IamCertKernelKind::MailVCode.to_string(),
        Some("".to_string()),
        &funs,
    )
    .await?;
    let phone_vcode_conf_id = IamCertServ::get_cert_conf_id_by_kind(
        &IamCertKernelKind::PhoneVCode.to_string(),
        Some("".to_string()),
        &funs,
    )
    .await?;
    let rbum_item_attr_kind_id = RbumKindAttrServ::find_one_rbum(&RbumKindAttrFilterReq {
        basic: RbumBasicFilterReq {
            name: Some("primary".to_string()),
            own_paths: Some("".to_string()),
            ..Default::default()
        },
        ..Default::default()
    }, funs, ctx).await?.map(|r| r.id).unwrap_or_default();

    let sql = format!(
        r#"
        SELECT
            iam_account.id,
            rbum_item.name,
            iam_account.employee_code,
            iam_account.labor_type,
            user_pwd_cert.ak as user_pwd,
            phone_vcode_cert.ak as phone,
            mail_vcode_cert.ak as mail,
            rbum_ext.value as primary_code
        FROM
            iam_account
            INNER JOIN rbum_cert AS user_pwd_cert ON user_pwd_cert.rel_rbum_id = iam_account.id
            AND user_pwd_cert.rel_rbum_kind = 0
            AND user_pwd_cert.rel_rbum_cert_conf_id = '{}'
            INNER JOIN rbum_item ON rbum_item.id = iam_account.id
            LEFT JOIN rbum_cert AS mail_vcode_cert ON mail_vcode_cert.rel_rbum_id = iam_account.id
            AND mail_vcode_cert.rel_rbum_kind = 0
            AND mail_vcode_cert.rel_rbum_cert_conf_id = '{}'
            LEFT JOIN rbum_cert AS phone_vcode_cert ON phone_vcode_cert.rel_rbum_id = iam_account.id
            AND phone_vcode_cert.rel_rbum_kind = 0
            AND phone_vcode_cert.rel_rbum_cert_conf_id = '{}'
            LEFT JOIN rbum_item_attr AS rbum_ext ON rbum_ext.rel_rbum_item_id = iam_account.id
            AND rbum_ext.rel_rbum_kind_attr_id = '{}'
        WHERE
            rbum_item.disabled = false
            AND rbum_item.scope_level = 0
            AND rbum_item.own_paths = '' {} {}
        "#,
        user_pwd_conf_id, mail_vcode_conf_id, phone_vcode_conf_id, rbum_item_attr_kind_id, join, where_clause
    );

    let result = funs.db().query_all(&sql, vec![]).await?;

    // 提取account id列表
    let account_fields: Vec<LdapAccountFields> = result
        .into_iter()
        .map(|row| LdapAccountFields {
            id: row.try_get::<String>("", "id").unwrap_or_default(),
            name: row.try_get::<String>("", "name").unwrap_or_default(),
            employee_code: row.try_get::<String>("", "employee_code").unwrap_or_default(),
            labor_type: row.try_get::<String>("", "labor_type").unwrap_or_default(),
            user_pwd: row.try_get::<String>("", "user_pwd").unwrap_or_default(),
            phone: row.try_get::<Option<String>>("", "phone").unwrap_or(None),
            mail: row.try_get::<Option<String>>("", "mail").unwrap_or(None),
            primary_code: row.try_get::<Option<String>>("", "primary_code").unwrap_or(None),
        })
        .collect();

    Ok(account_fields)
}

/// 根据LDAP查询类型构建SQL WHERE条件
fn build_sql_where_clause(
    query_type: &ldap_parser::LdapQueryType,
    _config: &IamLdapConfig,
) -> TardisResult<String> {
    match query_type {
        ldap_parser::LdapQueryType::Equality { attribute, value } => {
            build_equality_where_clause(attribute, value)
        }
        ldap_parser::LdapQueryType::Present { attribute } => {
            build_present_where_clause(attribute)
        }
        ldap_parser::LdapQueryType::And { filters } => {
            let conditions: Vec<String> = filters
                .iter()
                .map(|f| build_sql_where_clause(f, _config))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(format!("({})", conditions.join(" AND ")))
        }
        ldap_parser::LdapQueryType::Or { filters } => {
            let conditions: Vec<String> = filters
                .iter()
                .map(|f| build_sql_where_clause(f, _config))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(format!("({})", conditions.join(" OR ")))
        }
        ldap_parser::LdapQueryType::Not { filter } => {
            let condition = build_sql_where_clause(filter, _config)?;
            Ok(format!("NOT ({})", condition))
        }
        ldap_parser::LdapQueryType::Substring { attribute, substrings } => {
            build_substring_where_clause(attribute, substrings)
        }
        ldap_parser::LdapQueryType::GreaterOrEqual { attribute, value } => {
            build_comparison_where_clause(attribute, value, ">=")
        }
        ldap_parser::LdapQueryType::LessOrEqual { attribute, value } => {
            build_comparison_where_clause(attribute, value, "<=")
        }
        ldap_parser::LdapQueryType::ApproxMatch { attribute, value } => {
            // 近似匹配使用LIKE查询
            build_substring_where_clause(
                attribute,
                &LdapSubstringFilter {
                    initial: None,
                    any: vec![value.clone()],
                    final_: None,
                },
            )
        }
    }
}

/// objectClass 的固定值列表
const OBJECT_CLASS_VALUES: &[&str] = &["inetOrgPerson", "uidObject", "top"];

/// 检查是否为 objectClass 属性
fn is_object_class(attr: &str) -> bool {
    attr.to_lowercase() == "objectclass"
}

/// 检查 objectClass 值是否在固定列表中
fn is_valid_object_class_value(value: &str) -> bool {
    OBJECT_CLASS_VALUES.iter().any(|&v| v.eq_ignore_ascii_case(value))
}

/// 根据 LDAP 属性名获取对应的数据库查询字段
fn get_db_field(attr: &str) -> TardisResult<&'static str> {
    // 如果是 objectClass，直接返回错误，因为 objectClass 需要特殊处理
    if is_object_class(attr) {
        return Err(tardis::basic::error::TardisError::format_error(
            "objectClass should be handled by special functions",
            "406-iam-ldap-objectclass-special-handling",
        ));
    }
    LDAP_ATTR_TO_DB_FIELD
        .get(attr.to_lowercase().as_str())
        .copied()
        .ok_or_else(|| {
            tardis::basic::error::TardisError::format_error(
                &format!("Unsupported LDAP attribute: {}", attr),
                "406-iam-ldap-unsupported-attribute",
            )
        })
}

/// 构建精确匹配的WHERE条件
fn build_equality_where_clause(attribute: &str, value: &str) -> TardisResult<String> {
    // 特殊处理 objectClass
    if is_object_class(attribute) {
        // 检查查询的值是否在固定列表中
        if is_valid_object_class_value(value) {
            // 如果在列表中，返回总是为真的条件（所有账户都有这些 objectClass）
            return Ok("1=1".to_string());
        } else {
            // 如果不在列表中，返回总是为假的条件（没有账户有这个 objectClass）
            return Ok("1=0".to_string());
        }
    }
    
    let escaped_value = value.replace("'", "''"); // SQL注入防护
    let field = get_db_field(attribute)?;
    Ok(format!("{} = '{}'", field, escaped_value))
}

/// 构建存在性查询的WHERE条件
fn build_present_where_clause(attribute: &str) -> TardisResult<String> {
    // 特殊处理 objectClass：所有账户都有 objectClass，所以总是返回真
    if is_object_class(attribute) {
        return Ok("1=1".to_string());
    }
    
    let field = get_db_field(attribute)?;
    Ok(format!("{} IS NOT NULL AND {} != ''", field, field))
}

/// 构建子串匹配的WHERE条件
fn build_substring_where_clause(
    attribute: &str,
    substrings: &ldap3_proto::proto::LdapSubstringFilter,
) -> TardisResult<String> {
    // 特殊处理 objectClass：检查子串过滤器是否匹配固定值列表中的任何一个
    if is_object_class(attribute) {
        // 如果所有部分都为空，则视为存在性查询，返回真
        if substrings.initial.is_none() && substrings.any.is_empty() && substrings.final_.is_none() {
            return Ok("1=1".to_string());
        }
        
        // 检查是否有任何一个固定值匹配整个子串过滤器
        let matched = OBJECT_CLASS_VALUES.iter().any(|&value| {
            let value_lower = value.to_lowercase();
            
            // 检查 initial 部分
            if let Some(initial) = &substrings.initial {
                if !value_lower.starts_with(&initial.to_lowercase()) {
                    return false;
                }
            }
            
            // 检查 any 部分（所有 any 部分都必须在值中出现）
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
            
            // 检查 final 部分
            if let Some(final_part) = &substrings.final_ {
                if !remaining_value.to_lowercase().ends_with(&final_part.to_lowercase()) {
                    return false;
                }
            }
            
            true
        });
        
        return if matched {
            Ok("1=1".to_string())
        } else {
            Ok("1=0".to_string())
        };
    }
    
    let escaped_initial = substrings.initial.as_ref().map(|s| s.replace("'", "''"));
    let escaped_any = substrings
        .any
        .iter()
        .map(|s| s.replace("'", "''"))
        .collect::<Vec<_>>();
    let escaped_final = substrings.final_.as_ref().map(|s| s.replace("'", "''"));
    let field = get_db_field(attribute)?;
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

/// 构建比较查询的WHERE条件（仅支持 employeenumber 等可比较字段）
fn build_comparison_where_clause(attribute: &str, value: &str, operator: &str) -> TardisResult<String> {
    // objectClass 不支持比较查询，返回错误
    if is_object_class(attribute) {
        return Err(tardis::basic::error::TardisError::format_error(
            "objectClass does not support comparison operations",
            "406-iam-ldap-objectclass-no-comparison",
        ));
    }
    
    let field = get_db_field(attribute)?;
    let escaped_value = value.replace("'", "''");
    Ok(format!("{} {} '{}'", field, operator, escaped_value))
}
