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

pub const RBUM_EXT_TABLE_IAM_TENANT: &str = "iam_tenant";
pub const RBUM_EXT_TABLE_IAM_APP: &str = "iam_app";
pub const RBUM_EXT_TABLE_IAM_ACCOUNT: &str = "iam_account";
pub const RBUM_EXT_TABLE_IAM_ROLE: &str = "iam_role";
pub const RBUM_EXT_TABLE_IAM_RES: &str = "iam_res";
pub const RBUM_EXT_TABLE_IAM_SUB_DEPLOY: &str = "iam_sub_deploy";

pub const RBUM_SYSTEM_OWNER: &str = "_system_";

pub const RBUM_ITEM_NAME_SYS_ADMIN_ACCOUNT: &str = "bios";
pub const RBUM_ITEM_NAME_SYS_ADMIN_ROLE: &str = "sys_admin";
pub const RBUM_ITEM_NAME_TENANT_ADMIN_ROLE: &str = "tenant_admin";
pub const RBUM_ITEM_NAME_TENANT_AUDIT_ROLE: &str = "tenant_audit";
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

pub const OPENAPI_GATEWAY_PLUGIN_TIME_RANGE: &str = "redis-time-range:opres-time-range";
pub const OPENAPI_GATEWAY_PLUGIN_LIMIT: &str = "redis-limit:opres-limit";
pub const OPENAPI_GATEWAY_PLUGIN_COUNT: &str = "redis-count:opres-count";
pub const OPENAPI_GATEWAY_PLUGIN_DYNAMIC_ROUTE: &str = "redis-dynamic-route:opres-dynamic-route";

pub const DEFAULT_V_CODE_CD_IN_SEC: u32 = 60;

pub fn get_tardis_inst() -> TardisFunsInst {
    TardisFuns::inst_with_db_conn(COMPONENT_CODE.to_string(), None)
}
