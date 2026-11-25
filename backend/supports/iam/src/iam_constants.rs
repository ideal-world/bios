use tardis::TardisFuns;
use tardis::TardisFunsInst;

use bios_basic::rbum::rbum_enumeration::RbumScopeLevelKind;

pub const COMPONENT_CODE: &str = "iam";
pub const RBUM_KIND_CODE_IAM_TENANT: &str = "iam-tenant";
pub const RBUM_KIND_CODE_IAM_APP: &str = "iam-app";
pub const RBUM_KIND_CODE_IAM_ACCOUNT: &str = "iam-account";
pub const RBUM_KIND_CODE_IAM_ROLE: &str = "iam-role";
pub const RBUM_KIND_CODE_IAM_RES: &str = "iam-res";
pub const RBUM_KIND_CODE_IAM_SUB_DEPLOY: &str = "iam-sub-deploy";
pub const RBUM_KIND_CODE_IAM_THIRD_PARTY_APP: &str = "iam-third-party-app";
pub const RBUM_KIND_CODE_IAM_PUBLISH_SYSTEM: &str = "iam-publish-system";

pub const RBUM_EXT_TABLE_IAM_TENANT: &str = "iam_tenant";
pub const RBUM_EXT_TABLE_IAM_APP: &str = "iam_app";
pub const RBUM_EXT_TABLE_IAM_ACCOUNT: &str = "iam_account";
pub const RBUM_EXT_TABLE_IAM_ROLE: &str = "iam_role";
pub const RBUM_EXT_TABLE_IAM_RES: &str = "iam_res";
pub const RBUM_EXT_TABLE_IAM_SUB_DEPLOY: &str = "iam_sub_deploy";
pub const RBUM_EXT_TABLE_IAM_THIRD_PARTY_APP: &str = "iam_third_party_app";
pub const RBUM_EXT_TABLE_IAM_PUBLISH_SYSTEM: &str = "iam_publish_system";

pub const RBUM_SYSTEM_OWNER: &str = "_system_";

pub const RBUM_ITEM_NAME_SYS_ADMIN_ACCOUNT: &str = "bios";
pub const RBUM_ITEM_NAME_SYS_ADMIN_ROLE: &str = "sys_admin";
pub const RBUM_ITEM_NAME_TENANT_ADMIN_ROLE: &str = "tenant_admin";
pub const RBUM_ITEM_NAME_TENANT_AUDIT_ROLE: &str = "tenant_audit";
pub const RBUM_ITEM_NAME_TENANT_APP_MANAGER: &str = "tenant_app_manager";
pub const RBUM_ITEM_NAME_APP_ADMIN_ROLE: &str = "app_admin";
pub const RBUM_ITEM_NAME_APP_ADMIN_OM_ROLE: &str = "app_admin_om";
pub const RBUM_ITEM_NAME_APP_ADMIN_DEVELOP_ROLE: &str = "app_admin_develop";
pub const RBUM_ITEM_NAME_APP_ADMIN_PRODUCT_ROLE: &str = "app_admin_product";
pub const RBUM_ITEM_NAME_APP_ADMIN_ITERATE_ROLE: &str = "app_admin_iterate";
pub const RBUM_ITEM_NAME_APP_ADMIN_TEST_ROLE: &str = "app_admin_test";
pub const RBUM_ITEM_NAME_APP_NORMAL_ROLE: &str = "app_normal";
pub const RBUM_ITEM_NAME_APP_NORMAL_OM_ROLE: &str = "app_normal_om";
pub const RBUM_ITEM_NAME_APP_NORMAL_DEVELOP_ROLE: &str = "app_normal_develop";
pub const RBUM_ITEM_NAME_APP_NORMAL_PRODUCT_ROLE: &str = "app_normal_product";
pub const RBUM_ITEM_NAME_APP_NORMAL_ITERATE_ROLE: &str = "app_normal_iterate";
pub const RBUM_ITEM_NAME_APP_NORMAL_TEST_ROLE: &str = "app_normal_test";
pub const RBUM_ITEM_NAME_APP_READ_ROLE: &str = "app_read";
pub const RBUM_ITEM_NAME_PROJECT_READ_ROLE: &str = "proj_read";

pub const RBUM_ITEM_ID_TENANT_LEN: u8 = 6;
pub const RBUM_ITEM_ID_APP_LEN: u8 = 6;
pub const RBUM_ITEM_ID_SUB_ROLE_LEN: u8 = 6;

pub const RBUM_SCOPE_LEVEL_PRIVATE: RbumScopeLevelKind = RbumScopeLevelKind::Private;
pub const RBUM_SCOPE_LEVEL_GLOBAL: RbumScopeLevelKind = RbumScopeLevelKind::Root;
pub const RBUM_SCOPE_LEVEL_TENANT: RbumScopeLevelKind = RbumScopeLevelKind::L1;
pub const RBUM_SCOPE_LEVEL_APP: RbumScopeLevelKind = RbumScopeLevelKind::L2;

pub const RBUM_CERT_CONF_TOKEN_EXPIRE_SEC: i64 = 60 * 60 * 24 * 7;
pub const RBUM_CERT_CONF_TOKEN_DEFAULT_COEXIST_NUM: i16 = 5;

pub const EVENT_EXECUTE_TASK_EXTERNAL: &str = "iam/execute_task_external";
pub const EVENT_STOP_TASK_EXTERNAL: &str = "iam/stop_task_external";
pub const EVENT_SET_TASK_PROCESS_DATA_EXTERNAL: &str = "iam/set_task_process_data";
pub const IAM_AVATAR: &str = env!("CARGO_PKG_NAME");

pub const IAM_OAUTH2_TOKEN_META_CACHE_KEY_PREFIX: &str = "iam:cache:token:oauth2:meta:";

// 账号上下文按租户隔离后的 hash field 约定。
// 注意：必须与 `bios_auth::auth_constants` 中的同名函数保持一致。
pub fn account_tenant_context_field(tenant_id: &str) -> String {
    format!("tenant:{tenant_id}")
}

pub fn account_app_context_field(tenant_id: &str, app_id: &str) -> String {
    format!("app:{tenant_id}:{app_id}")
}

/// `iam:cache:token:info:{token}` 的解析结果。
///
/// 值为位置化格式 `{token_kind},{account_id},{expire},{tenant_id}`，后两段允许为空；
/// 旧数据可能只有 2 段或 3 段，此时 `tenant_id` 为 `None`，表示未绑定租户（按旧逻辑读取）。
/// 注意：`tenant_id` 为 `Some("")` 表示绑定到平台上下文，与 `None` 语义不同。
///
/// 注意：必须与 `bios_auth::auth_constants` 中的同名结构/函数保持一致。
pub struct TokenInfoParts<'a> {
    pub token_kind: &'a str,
    pub account_id: &'a str,
    pub renewal_expire_sec: Option<i64>,
    pub tenant_id: Option<&'a str>,
}

pub fn parse_token_info(value: &str) -> TokenInfoParts<'_> {
    let mut parts = value.split(',');
    let token_kind = parts.next().unwrap_or_default();
    let account_id = parts.next().unwrap_or_default();
    let renewal_expire_sec = parts.next().and_then(|seg| if seg.is_empty() { None } else { seg.parse().ok() });
    let tenant_id = parts.next();
    TokenInfoParts {
        token_kind,
        account_id,
        renewal_expire_sec,
        tenant_id,
    }
}

pub fn build_token_info(token_kind: &str, account_id: &str, renewal_expire_sec: Option<i64>, tenant_id: Option<&str>) -> String {
    match (renewal_expire_sec, tenant_id) {
        (None, None) => format!("{token_kind},{account_id}"),
        (Some(expire), None) => format!("{token_kind},{account_id},{expire}"),
        (renewal, Some(tenant_id)) => format!("{token_kind},{account_id},{},{tenant_id}", renewal.map(|v| v.to_string()).unwrap_or_default()),
    }
}

pub const DEFAULT_V_CODE_CD_IN_SEC: u32 = 60;

pub fn get_tardis_inst() -> TardisFunsInst {
    TardisFuns::inst_with_db_conn(COMPONENT_CODE.to_string(), None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_legacy_token_info_without_tenant() {
        let parts = parse_token_info("TokenDefault,acct1");
        assert_eq!(parts.token_kind, "TokenDefault");
        assert_eq!(parts.account_id, "acct1");
        assert_eq!(parts.renewal_expire_sec, None);
        assert_eq!(parts.tenant_id, None);

        let parts = parse_token_info("TokenDefault,acct1,300");
        assert_eq!(parts.renewal_expire_sec, Some(300));
        assert_eq!(parts.tenant_id, None);
    }

    #[test]
    fn parse_token_info_with_tenant_and_empty_expire() {
        let parts = parse_token_info("TokenDefault,acct1,,tenant1");
        assert_eq!(parts.renewal_expire_sec, None);
        assert_eq!(parts.tenant_id, Some("tenant1"));

        let parts = parse_token_info("TokenDefault,acct1,300,tenant1");
        assert_eq!(parts.renewal_expire_sec, Some(300));
        assert_eq!(parts.tenant_id, Some("tenant1"));
    }

    #[test]
    fn empty_tenant_segment_means_platform_scope() {
        assert_eq!(parse_token_info("TokenDefault,acct1,,").tenant_id, Some(""));
        assert_eq!(parse_token_info("TokenDefault,acct1,300,").tenant_id, Some(""));
        assert_eq!(parse_token_info("TokenDefault,acct1").tenant_id, None);
    }

    #[test]
    fn build_token_info_roundtrip() {
        assert_eq!(build_token_info("TokenDefault", "acct1", None, None), "TokenDefault,acct1");
        assert_eq!(build_token_info("TokenDefault", "acct1", Some(300), None), "TokenDefault,acct1,300");
        assert_eq!(build_token_info("TokenDefault", "acct1", None, Some("tenant1")), "TokenDefault,acct1,,tenant1");
        assert_eq!(build_token_info("TokenDefault", "acct1", Some(300), Some("tenant1")), "TokenDefault,acct1,300,tenant1");
        assert_eq!(build_token_info("TokenDefault", "acct1", None, Some("")), "TokenDefault,acct1,,");

        for value in [
            "TokenDefault,acct1",
            "TokenDefault,acct1,300",
            "TokenDefault,acct1,,tenant1",
            "TokenDefault,acct1,300,tenant1",
        ] {
            let parts = parse_token_info(value);
            assert_eq!(build_token_info(parts.token_kind, parts.account_id, parts.renewal_expire_sec, parts.tenant_id), value);
        }
    }
}
