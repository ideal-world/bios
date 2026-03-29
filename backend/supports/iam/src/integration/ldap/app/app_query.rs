//! LDAP App Query Handler
//!
//! 负责与IAM数据交互，执行应用查询操作

use itertools::Itertools;
use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::TardisFunsInst;

use crate::basic::serv::iam_cert_serv::IamCertServ;
use crate::iam_config::IamLdapConfig;
use crate::iam_constants;
use crate::iam_enumeration::IamCertKernelKind;
use crate::integration::ldap::app::app_result::LdapAppFields;
use crate::integration::ldap::ldap_parser::{self, LdapQueryType};
use crate::integration::ldap::ldap_query::LdapSqlWhereBuilder;

/// 执行LDAP应用搜索查询
pub async fn execute_ldap_app_search(query: &ldap_parser::LdapSearchQuery, config: &IamLdapConfig) -> TardisResult<Vec<LdapAppFields>> {
    let funs = iam_constants::get_tardis_inst();
    let ctx = TardisContext::default();

    // 处理简单存在性查询（从base DN提取CN）
    if ldap_parser::is_simple_present_query(query) {
        if let Some(cn) = ldap_parser::extract_cn_from_base(&query.base) {
            let mut simple_query = query.clone();
            simple_query.query_type = LdapQueryType::Equality { attribute: "cn".to_string(), value: cn };
            let apps = build_and_execute_app_sql_query(&simple_query, config, &funs, &ctx).await?;
            return Ok(apps);
        } else {
            return Ok(vec![]);
        }
    }

    // 使用原生SQL查询方式（参考 account 逻辑）
    let apps = build_and_execute_app_sql_query(query, config, &funs, &ctx).await?;

    Ok(apps)
}

/// 根据LDAP查询参数构建SQL并执行，返回符合条件的应用列表
async fn build_and_execute_app_sql_query(
    query: &ldap_parser::LdapSearchQuery,
    config: &IamLdapConfig,
    funs: &TardisFunsInst,
    _ctx: &TardisContext,
) -> TardisResult<Vec<LdapAppFields>> {
    let phone_vcode_conf_id = IamCertServ::get_cert_conf_id_by_kind(&IamCertKernelKind::PhoneVCode.to_string(), Some("".to_string()), funs).await?;

    // 构建SQL WHERE条件（参考 account 逻辑）
    let (join, where_clause) = if ldap_parser::is_full_query(query) {
        ("", "".to_string())
    } else {
        ("AND", build_sql_where_clause(&query.query_type, config)?)
    };

    let sql = format!(
        r#"
        SELECT
            iam_third_party_app.external_id,
            rbum_item.name,
            iam_third_party_app.sort
        FROM
            iam_third_party_app
            INNER JOIN rbum_item ON iam_third_party_app.id = rbum_item.id
        WHERE
            1 = 1 {} {}
        "#,
        join, where_clause
    );

    let rel_sql = format!(
        r#"
        SELECT
            iam_third_party_app.external_id,
            rbum_rel_account.to_rbum_item_id,
            phone_vcode_cert.ak as phone,
            iam_third_party_app.sort
        FROM
            iam_third_party_app
        LEFT JOIN rbum_rel AS rbum_rel_account ON iam_third_party_app.id = rbum_rel_account.to_rbum_item_id
            AND rbum_rel_account.tag = 'IamThirdPartyAppAccount'
            INNER JOIN rbum_cert AS phone_vcode_cert ON phone_vcode_cert.rel_rbum_id = rbum_rel_account.from_rbum_id
            AND phone_vcode_cert.rel_rbum_kind = 0
            AND phone_vcode_cert.status = 1
            AND phone_vcode_cert.rel_rbum_cert_conf_id = '{}'
        WHERE
        1 = 1
        "#,
        phone_vcode_conf_id
    );

    let result = funs.db().query_all(&sql, vec![]).await?;
    let rel_result = funs.db().query_all(&rel_sql, vec![]).await?;

    // 将 rows 映射为 Vec<LdapAppFields>
    let mut apps: Vec<LdapAppFields> = result
        .into_iter()
        .map(|row| {
            let id = row.try_get::<String>("", "external_id").unwrap_or_default();
            let name = row.try_get::<String>("", "name").unwrap_or_default();
            let sort = row.try_get::<i64>("", "sort").unwrap_or_default();
            let rel_phones = rel_result
            .iter()
            .filter(|r| r.try_get::<String>("", "external_id").unwrap_or_default() == id)
            .map(|r| r.try_get::<String>("", "phone").unwrap_or_default()).collect_vec();
            LdapAppFields {
                id: id.clone(),
                business_category: name.clone(),
                sort,
                phones: rel_phones,
            }
        })
        .collect();

    // 按 sys_code 排序（与 get_tree 一致）
    apps.sort_by(|a, b| a.sort.cmp(&b.sort));

    Ok(apps)
}

/// 根据LDAP查询类型构建SQL WHERE条件
fn build_sql_where_clause(query_type: &ldap_parser::LdapQueryType, config: &IamLdapConfig) -> TardisResult<String> {
    AppLdapSqlWhereBuilder::build_sql_where_clause(query_type, config)
}

/// 应用查询专用的 LDAP SQL WHERE 构建器
struct AppLdapSqlWhereBuilder;

impl LdapSqlWhereBuilder for AppLdapSqlWhereBuilder {
    /// objectClass 的固定值列表
    const OBJECT_CLASS_VALUES: &'static [&'static str] = &["groupOfUniqueNames", "top"];

    /// LDAP 属性名 -> 数据库查询字段 映射表 (attr, db_field)
    const ATTR_TO_DB_FIELD: &'static [(&'static str, &'static str)] = &[
        ("cn", "iam_third_party_app.external_id"),
        ("name", "rbum_item.name"),
        ("id", "rbum_item.id"),
    ];
}
