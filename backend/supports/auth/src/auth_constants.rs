pub const DOMAIN_CODE: &str = "auth";
// see cargo.toml lib.name
pub const PACKAGE_NAME: &str = "bios_auth";

pub const OAUTH2_TOKEN_META_KEY_PREFIX: &str = "iam:cache:token:oauth2:meta:";
pub const OAUTH2_ALL_SCOPE: &str = "all";
pub const OAUTH2_USERINFO_READ_SCOPE: &str = "iam.userinfo.read";
pub const OAUTH2_APP_READ_SCOPE: &str = "iam.app.read";
pub const OAUTH2_APP_ROLE_MEMBER_READ_SCOPE: &str = "iam.app.role_member.read";

// 账号上下文按租户隔离后的 hash field 约定。
// 注意：必须与 `bios_iam::iam_constants` 中的同名函数保持一致。
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
/// 注意：必须与 `bios_iam::iam_constants` 中的同名结构/函数保持一致。
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
        // 4 段但租户为空 -> 平台上下文；3 段 -> 未绑定
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
