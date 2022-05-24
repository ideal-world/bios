use tardis::basic::dto::TardisFunsInst;
use tardis::TardisFuns;

use bios_basic::rbum::rbum_enumeration::RbumScopeLevelKind;

pub const COMPONENT_CODE: &str = "iam";
pub const RBUM_KIND_SCHEME_IAM_TENANT: &str = "iam-tenant";
pub const RBUM_KIND_SCHEME_IAM_APP: &str = "iam-app";
pub const RBUM_KIND_SCHEME_IAM_ACCOUNT: &str = "iam-account";
pub const RBUM_KIND_SCHEME_IAM_ROLE: &str = "iam-role";
pub const RBUM_KIND_SCHEME_IAM_RES: &str = "iam-res";

pub const RBUM_ITEM_NAME_SYS_ADMIN_ACCOUNT: &str = "bios";
pub const RBUM_ITEM_NAME_SYS_ADMIN_ROLE: &str = "sys_admin";
pub const RBUM_ITEM_NAME_TENANT_ADMIN_ROLE: &str = "tenant_admin";
pub const RBUM_ITEM_NAME_APP_ADMIN_ROLE: &str = "app_admin";

pub const RBUM_ITEM_ID_TENANT_LEN: u8 = 6;
pub const RBUM_ITEM_ID_APP_LEN: u8 = 6;

pub const RBUM_SCOPE_LEVEL_PRIVATE: RbumScopeLevelKind = RbumScopeLevelKind::Private;
pub const RBUM_SCOPE_LEVEL_GLOBAL: RbumScopeLevelKind = RbumScopeLevelKind::Root;
pub const RBUM_SCOPE_LEVEL_TENANT: RbumScopeLevelKind = RbumScopeLevelKind::L1;
pub const RBUM_SCOPE_LEVEL_APP: RbumScopeLevelKind = RbumScopeLevelKind::L2;

pub const RBUM_CERT_CONF_TOKEN_EXPIRE_SEC: u32 = 60 * 60 * 24 * 7;
pub const RBUM_CERT_CONF_TOKEN_DEFAULT_COEXIST_NUM: u32 = 5;

pub fn get_tardis_inst<'a>() -> TardisFunsInst<'a> {
    TardisFuns::inst_with_db_conn(COMPONENT_CODE.to_string())
}
