//! LDAP Account Query Handler
//!
//! 负责与IAM数据交互，执行账户查询操作

use std::collections::HashMap;

use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;
use lazy_static::lazy_static;
use ldap3_proto::proto::LdapSubstringFilter;
use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::TardisFunsInst;

use bios_basic::rbum::dto::rbum_filer_dto::RbumBasicFilterReq;

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
use crate::integration::ldap::ldap_parser;

/// 执行LDAP账户搜索查询
pub async fn execute_ldap_account_search(
    query: &ldap_parser::LdapSearchQuery,
    config: &IamLdapConfig,
) -> TardisResult<Vec<IamAccountDetailAggResp>> {
    let funs = iam_constants::get_tardis_inst();

    // 处理根DSE查询（特殊 case，不需要查询账户）
    if ldap_parser::is_root_dse_query(query) {
        return Ok(vec![]);
    }

    // 处理简单存在性查询（从base DN提取CN，检查账户是否存在）
    if ldap_parser::is_simple_present_query(query) {
        if let Some(cn) = ldap_parser::extract_cn_from_base(&query.base) {
            match check_account_exists(&cn).await {
                Ok(true) => {
                    // 账户存在，返回账户详情
                    if let Ok(Some(account)) = get_account_by_cn(&cn).await {
                        return Ok(vec![account]);
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
    let account_ids = build_and_execute_sql_query(query, config, &funs, &ctx).await?;

    // 如果查询结果为空，直接返回
    if account_ids.is_empty() {
        return Ok(vec![]);
    }

    // 调用IamAccountServ::get_account_detail_aggs获取账户详情
    let mut account_details = Vec::new();
    for account_id in account_ids {
        if let Ok(detail) = IamAccountServ::get_account_detail_aggs(
            &account_id,
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
        .await
        {
            account_details.push(detail);
        }
    }

    Ok(account_details)
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
    _ctx: &TardisContext,
) -> TardisResult<Vec<String>> {
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

    let sql = format!(
        r#"
        SELECT iam_account.id
        FROM iam_account
        LEFT JOIN rbum_cert AS user_pwd_cert ON user_pwd_cert.rel_rbum_id = iam_account.id 
            AND user_pwd_cert.rel_rbum_kind = 0 AND user_pwd_cert.rel_rbum_cert_conf_id = '{}'
        LEFT JOIN rbum_cert AS mail_vcode_cert ON mail_vcode_cert.rel_rbum_id = iam_account.id 
            AND mail_vcode_cert.rel_rbum_kind = 0 AND mail_vcode_cert.rel_rbum_cert_conf_id = '{}'
        LEFT JOIN rbum_item ON rbum_item.id = iam_account.id
        WHERE rbum_item.disabled = false AND rbum_item.scope_level = 0 AND rbum_item.own_paths = '' {} {}
        "#,
        user_pwd_conf_id, mail_vcode_conf_id, join, where_clause
    );

    let result = funs.db().query_all(&sql, vec![]).await?;

    // 提取account id列表
    let account_ids: Vec<String> = result
        .into_iter()
        .filter_map(|row| row.try_get::<String>("", "id").ok())
        .collect();

    Ok(account_ids)
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
        ldap_parser::LdapQueryType::RootDse => {
            Ok("1=0".to_string()) // 根DSE查询不返回任何结果
        }
    }
}

/// 根据 LDAP 属性名获取对应的数据库查询字段
fn get_db_field(attr: &str) -> TardisResult<&'static str> {
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
    let escaped_value = value.replace("'", "''"); // SQL注入防护
    let field = get_db_field(attribute)?;
    Ok(format!("{} = '{}'", field, escaped_value))
}

/// 构建存在性查询的WHERE条件
fn build_present_where_clause(attribute: &str) -> TardisResult<String> {
    let field = get_db_field(attribute)?;
    Ok(format!("{} IS NOT NULL AND {} != ''", field, field))
}

/// 构建子串匹配的WHERE条件
fn build_substring_where_clause(
    attribute: &str,
    substrings: &ldap3_proto::proto::LdapSubstringFilter,
) -> TardisResult<String> {
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
    let field = get_db_field(attribute)?;
    let escaped_value = value.replace("'", "''");
    Ok(format!("{} {} '{}'", field, operator, escaped_value))
}
