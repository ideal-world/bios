//! LDAP Organization Query Handler
//!
//! 负责与IAM数据交互，执行组织查询操作

use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::TardisFunsInst;

use crate::basic::serv::iam_set_serv::IamSetServ;
use crate::iam_config::IamLdapConfig;
use crate::iam_constants;
use crate::iam_enumeration::IamSetKind;
use crate::integration::ldap::ldap_parser::{self, LdapQueryType};
use crate::integration::ldap::ldap_query::LdapSqlWhereBuilder;
use crate::integration::ldap::organization::org_result::LdapOrgFields;

/// 执行LDAP组织搜索查询
pub async fn execute_ldap_org_search(query: &ldap_parser::LdapSearchQuery, config: &IamLdapConfig) -> TardisResult<Vec<LdapOrgFields>> {
    let funs = iam_constants::get_tardis_inst();
    let ctx = TardisContext::default();

    // 处理简单存在性查询（从base DN提取CN）
    if ldap_parser::is_simple_present_query(query) {
        if let Some(cn) = ldap_parser::extract_cn_from_base(&query.base) {
            let mut simple_query = query.clone();
            simple_query.query_type = LdapQueryType::Equality { attribute: "cn".to_string(), value: cn };
            let orgs = build_and_execute_org_sql_query(&simple_query, config, &funs, &ctx).await?;
            return Ok(orgs);
        } else {
            return Ok(vec![]);
        }
    }

    // 使用原生SQL查询方式（参考 account 逻辑）
    let orgs = build_and_execute_org_sql_query(query, config, &funs, &ctx).await?;

    Ok(orgs)
}

/// 根据LDAP查询参数构建SQL并执行，返回符合条件的组织列表
async fn build_and_execute_org_sql_query(
    query: &ldap_parser::LdapSearchQuery,
    config: &IamLdapConfig,
    funs: &TardisFunsInst,
    ctx: &TardisContext,
) -> TardisResult<Vec<LdapOrgFields>> {
    // 构建SQL WHERE条件（参考 account 逻辑）
    let (join, where_clause) = if ldap_parser::is_full_query(query) {
        ("", "".to_string())
    } else {
        ("AND", build_sql_where_clause(&query.query_type, config)?)
    };

    // 获取组织Set ID
    let set_id = IamSetServ::get_default_set_id_by_ctx(&IamSetKind::Org, funs, ctx).await?;

    let sql = format!(
        r#"
        SELECT
            rbum_set_cate.id,
            rbum_set_cate.sys_code,
            rbum_set_cate.bus_code,
            rbum_set_cate.name,
            rbum_set_cate.icon,
            rbum_set_cate.owner,
            rbum_set_cate.create_time,
            rbum_set_cate.update_time
        FROM
            rbum_set_cate
            INNER JOIN rbum_set ON rbum_set_cate.rel_rbum_set_id = rbum_set.id
        WHERE
            rbum_set_cate.rel_rbum_set_id = '{}'
            AND rbum_set.kind = 'Org'
            AND rbum_set_cate.own_paths = '' {} {}
        "#,
        set_id, join, where_clause
    );

    let result = funs.db().query_all(&sql, vec![]).await?;

    // 将 rows 映射为 Vec<LdapOrgFields>
    let mut orgs: Vec<LdapOrgFields> = result
        .into_iter()
        .map(|row| {
            let id = row.try_get::<String>("", "id").unwrap_or_default();
            let sys_code = row.try_get::<String>("", "sys_code").unwrap_or_default();
            let bus_code = row.try_get::<String>("", "bus_code").unwrap_or_default();
            let name = row.try_get::<String>("", "name").unwrap_or_default();
            let icon = row.try_get::<String>("", "icon").unwrap_or_default();
            LdapOrgFields {
                id: id.clone(),
                name,
                sys_code: sys_code.clone(),
                bus_code: if bus_code.is_empty() { None } else { Some(bus_code) },
                icon: if icon.is_empty() { None } else { Some(icon) },
                create_time: row.try_get("", "create_time").unwrap_or_default(),
                update_time: row.try_get("", "update_time").unwrap_or_default(),
            }
        })
        .collect();

    // 按 sys_code 排序（与 get_tree 一致）
    orgs.sort_by(|a, b| a.sys_code.cmp(&b.sys_code));

    Ok(orgs)
}

/// 根据LDAP查询类型构建SQL WHERE条件
fn build_sql_where_clause(query_type: &ldap_parser::LdapQueryType, config: &IamLdapConfig) -> TardisResult<String> {
    OrgLdapSqlWhereBuilder::build_sql_where_clause(query_type, config)
}

/// 组织查询专用的 LDAP SQL WHERE 构建器
struct OrgLdapSqlWhereBuilder;

impl LdapSqlWhereBuilder for OrgLdapSqlWhereBuilder {
    /// objectClass 的固定值列表
    const OBJECT_CLASS_VALUES: &'static [&'static str] = &["organizationalUnit", "top"];

    /// LDAP 属性名 -> 数据库查询字段 映射表 (attr, db_field)
    const ATTR_TO_DB_FIELD: &'static [(&'static str, &'static str)] = &[
        ("cn", "rbum_set_cate.id"),
        ("name", "rbum_set_cate.name"),
        ("syscode", "rbum_set_cate.sys_code"),
        ("sys_code", "rbum_set_cate.sys_code"),
        ("buscode", "rbum_set_cate.bus_code"),
        ("bus_code", "rbum_set_cate.bus_code"),
        ("description", "rbum_set_cate.ext"),
        ("ext", "rbum_set_cate.ext"),
    ];
}
