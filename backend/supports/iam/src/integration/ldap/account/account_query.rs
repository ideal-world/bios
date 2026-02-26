//! LDAP Account Query Handler
//!
//! 负责与IAM数据交互，执行账户查询操作

use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;
use bios_basic::rbum::serv::rbum_kind_serv::RbumKindAttrServ;
use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::TardisFunsInst;

use bios_basic::rbum::dto::rbum_filer_dto::{RbumBasicFilterReq, RbumKindAttrFilterReq};

use crate::basic::dto::iam_account_dto::IamAccountDetailAggResp;
use crate::basic::dto::iam_filer_dto::IamAccountFilterReq;
use crate::basic::serv::iam_account_serv::IamAccountServ;
use crate::basic::serv::iam_cert_serv::IamCertServ;
use crate::iam_config::IamLdapConfig;
use crate::iam_constants;
use crate::iam_enumeration::IamCertKernelKind;
use crate::integration::ldap::account::account_result::LdapAccountFields;
use crate::integration::ldap::ldap_parser;
use crate::integration::ldap::ldap_query::LdapSqlWhereBuilder;

/// 执行LDAP账户搜索查询
pub async fn execute_ldap_account_search(query: &ldap_parser::LdapSearchQuery, config: &IamLdapConfig) -> TardisResult<Vec<LdapAccountFields>> {
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
    let rbum_cert_conf_id = IamCertServ::get_cert_conf_id_by_kind(&IamCertKernelKind::UserPwd.to_string(), Some("".to_string()), &funs).await?;
    bios_basic::rbum::serv::rbum_cert_serv::RbumCertServ::check_exist(ak, &rbum_cert_conf_id, "", &funs).await
}

/// 根据CN获取账户详情
async fn get_account_by_cn(ak: &str) -> TardisResult<Option<IamAccountDetailAggResp>> {
    let funs = iam_constants::get_tardis_inst();
    let rbum_cert_conf_id = IamCertServ::get_cert_conf_id_by_kind(&IamCertKernelKind::UserPwd.to_string(), Some("".to_string()), &funs).await?;

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
    let user_pwd_conf_id = IamCertServ::get_cert_conf_id_by_kind(&IamCertKernelKind::UserPwd.to_string(), Some("".to_string()), funs).await?;
    let mail_vcode_conf_id = IamCertServ::get_cert_conf_id_by_kind(&IamCertKernelKind::MailVCode.to_string(), Some("".to_string()), funs).await?;
    let phone_vcode_conf_id = IamCertServ::get_cert_conf_id_by_kind(&IamCertKernelKind::PhoneVCode.to_string(), Some("".to_string()), funs).await?;
    let rbum_item_attr_kind_id = RbumKindAttrServ::find_one_rbum(
        &RbumKindAttrFilterReq {
            basic: RbumBasicFilterReq {
                name: Some("primary".to_string()),
                own_paths: Some("".to_string()),
                ..Default::default()
            },
            ..Default::default()
        },
        funs,
        ctx,
    )
    .await?
    .map(|r| r.id)
    .unwrap_or_default();

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
            AND user_pwd_cert.status = 1
            AND user_pwd_cert.rel_rbum_cert_conf_id = '{}'
            INNER JOIN rbum_item ON rbum_item.id = iam_account.id
            LEFT JOIN rbum_cert AS mail_vcode_cert ON mail_vcode_cert.rel_rbum_id = iam_account.id
            AND mail_vcode_cert.rel_rbum_kind = 0
            AND mail_vcode_cert.status = 1
            AND mail_vcode_cert.rel_rbum_cert_conf_id = '{}'
            LEFT JOIN rbum_cert AS phone_vcode_cert ON phone_vcode_cert.rel_rbum_id = iam_account.id
            AND phone_vcode_cert.rel_rbum_kind = 0
            AND phone_vcode_cert.status = 1
            AND phone_vcode_cert.rel_rbum_cert_conf_id = '{}'
            LEFT JOIN rbum_item_attr AS rbum_ext ON rbum_ext.rel_rbum_item_id = iam_account.id
            AND rbum_ext.rel_rbum_kind_attr_id = '{}'
        WHERE
            rbum_item.disabled = false
            AND rbum_item.scope_level = 0
            AND iam_account.status = 0
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
fn build_sql_where_clause(query_type: &ldap_parser::LdapQueryType, config: &IamLdapConfig) -> TardisResult<String> {
    AccountLdapSqlWhereBuilder::build_sql_where_clause(query_type, config)
}

/// 账户查询专用的 LDAP SQL WHERE 构建器
struct AccountLdapSqlWhereBuilder;

impl LdapSqlWhereBuilder for AccountLdapSqlWhereBuilder {
    /// objectClass 的固定值列表
    const OBJECT_CLASS_VALUES: &'static [&'static str] = &["inetOrgPerson", "uidObject", "top"];

    /// LDAP 属性名 -> 数据库查询字段 映射表 (attr, db_field)
    const ATTR_TO_DB_FIELD: &'static [(&'static str, &'static str)] = &[
        ("cn", "phone_vcode_cert.ak"),
        ("uid", "user_pwd_cert.ak"),
        ("samaccountname", "user_pwd_cert.ak"),
        ("mail", "mail_vcode_cert.ak"),
        ("employeenumber", "iam_account.employee_code"),
        ("displayname", "rbum_item.name"),
        ("givenname", "rbum_item.name"),
        ("sn", "rbum_item.name"),
    ];
}
