use tardis::TardisFuns;
use tardis::TardisFunsInst;

use bios_basic::rbum::rbum_enumeration::RbumScopeLevelKind;

pub const COMPONENT_CODE: &str = "iam";
pub const RBUM_KIND_CODE_IAM_TENANT: &str = "iam-tenant";
pub const RBUM_KIND_CODE_IAM_APP: &str = "iam-app";
pub const RBUM_KIND_CODE_IAM_ACCOUNT: &str = "iam-account";
pub const RBUM_KIND_CODE_IAM_ROLE: &str = "iam-role";
pub const RBUM_KIND_CODE_IAM_RES: &str = "iam-res";

pub const RBUM_EXT_TABLE_IAM_TENANT: &str = "iam_tenant";
pub const RBUM_EXT_TABLE_IAM_APP: &str = "iam_app";
pub const RBUM_EXT_TABLE_IAM_ACCOUNT: &str = "iam_account";
pub const RBUM_EXT_TABLE_IAM_ROLE: &str = "iam_role";
pub const RBUM_EXT_TABLE_IAM_RES: &str = "iam_res";

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

pub const LOG_IAM_ACCOUNT_OP_LOGOUT: &str = "Logout";
pub const LOG_IAM_ACCOUNT_OP_DORMANTACCOUNT: &str = "DormantAccount";
pub const LOG_IAM_ACCOUNT_OP_ACTIVATEACCOUNT: &str = "ActivateAccount";
pub const LOG_IAM_ACCOUNT_OP_UNLOCKACCOUNT: &str = "UnlockAccount";
pub const LOG_IAM_ACCOUNT_OP_MODIFYACCOUNTICON: &str = "ModifyAccountIcon";
pub const LOG_IAM_ACCOUNT_OP_MODIFYNAME: &str = "ModifyName";
pub const LOG_IAM_ACCOUNT_OP_MODIFYUSERNAME: &str = "ModifyUserName";
pub const LOG_IAM_ACCOUNT_OP_MODIFYPASSWORD: &str = "ModifyPassword";
pub const LOG_IAM_ACCOUNT_OP_RESETACCOUNTPASSWORD: &str = "ResetAccountPassword";
pub const LOG_IAM_ACCOUNT_OP_ADDLONGTERMACCOUNT: &str = "AddLongTermAccount";
pub const LOG_IAM_ACCOUNT_OP_ADDTEMPACCOUNT: &str = "AddTempAccount";
pub const LOG_IAM_ACCOUNT_OP_BINDACCOUNT: &str = "BindAccount";
pub const LOG_IAM_ACCOUNT_OP_BIND5AACCOUNT: &str = "Bind5aAccount";
pub const LOG_IAM_ACCOUNT_OP_BINDMAILBOX: &str = "BindMailbox";
pub const LOG_IAM_ACCOUNT_OP_BINDPHONE: &str = "BindPhone";
pub const LOG_IAM_ACCOUNT_OP_PASSWORDLOCKACCOUNT: &str = "PasswordLockAccount";
pub const LOG_IAM_ACCOUNT_OP_OFFLINEACCOUNT: &str = "OfflineAccount";
pub const LOG_IAM_ACCOUNT_OP_QUIT: &str = "Quit";
pub const LOG_IAM_ACCOUNT_OP_ADDTENANTROLEASADMIN: &str = "AddTenantRoleAsAdmin";
pub const LOG_IAM_ACCOUNT_OP_REMOVETENANTROLEASADMIN: &str = "RemoveTenantRoleAsAdmin";
pub const LOG_IAM_ACCOUNT_OP_MANUALLYLOCKACCOUNT: &str = "ManuallyLockAccount";

pub const LOG_SECURITY_VISIT_OP_CONTINULOGINFAIL: &str = "ContinuLoginFail";
pub const LOG_SECURITY_VISIT_OP_LOGIN: &str = "Login";

pub const LOG_IAM_ROLE_OP_ADDROLEACCOUNT: &str = "AddRoleAccount";
pub const LOG_IAM_ROLE_OP_ADDCUSTOMIZEROLE: &str = "AddCustomizeRole";
pub const LOG_IAM_ROLE_OP_MODIFYCUSTOMIZEROLENAME: &str = "ModifyCustomizeRoleName";
pub const LOG_IAM_ROLE_OP_MODIFYBUILTROLENAME: &str = "ModifyBuiltRoleName";
pub const LOG_IAM_ROLE_OP_REMOVEROLEACCOUNT: &str = "RemoveRoleAccount";
pub const LOG_IAM_ROLE_OP_DELETECUSTOMIZEROLE: &str = "DeleteCustomizeRole";
pub const LOG_IAM_ROLE_OP_MODIFYCUSTOMIZEROLEPERMISSIONS: &str = "ModifyCustomizeRolePermissions";
pub const LOG_IAM_ROLE_OP_MODIFYBUILTROLEPERMISSIONS: &str = "ModifyBuiltRolePermissions";

pub const LOG_IAM_ORG_OP_ADD: &str = "Add";
pub const LOG_IAM_ORG_OP_RENAME: &str = "Rename";
pub const LOG_IAM_ORG_OP_DELETE: &str = "Delete";
pub const LOG_IAM_ORG_OP_ADDACCOUNT: &str = "AddAccount";
pub const LOG_IAM_ORG_OP_REMOVEACCOUNT: &str = "RemoveAccount";

pub const LOG_IAM_RES_OP_ADD: &str = "Add";
pub const LOG_IAM_RES_OP_ADDCONTENTPAGEASPERSONAL: &str = "AddContentPageaspersonal";
pub const LOG_IAM_RES_OP_ADDAPI: &str = "AddApi";
pub const LOG_IAM_RES_OP_ADDCONTENTPAGEBUTTON: &str = "AddContentPageButton";
pub const LOG_IAM_RES_OP_ADDPROJECT: &str = "AddProduct";
pub const LOG_IAM_RES_OP_ADDSPECIFICATION: &str = "AddSpecification";
pub const LOG_IAM_RES_OP_MODIFYCONTENTPAGE: &str = "ModifyContentPage";
pub const LOG_IAM_RES_OP_MODIFYAPI: &str = "ModifyApi";
pub const LOG_IAM_RES_OP_MODIFYELE: &str = "ModifyEle";
pub const LOG_IAM_RES_OP_MODIFYPROJECT: &str = "ModifyProduct";
pub const LOG_IAM_RES_OP_MODIFYSPECIFICATION: &str = "ModifySpecification";
pub const LOG_IAM_RES_OP_DELETECONTENTPAGEASPERSONAL: &str = "DeleteContentPageaspersonal";
pub const LOG_IAM_RES_OP_DELETEAPI: &str = "DeleteApi";
pub const LOG_IAM_RES_OP_REMOVECONTENTPAGEBUTTON: &str = "RemoveContentPageButton";
pub const LOG_IAM_RES_OP_REMOVEPROJECT: &str = "RemoveProduct";
pub const LOG_IAM_RES_OP_REMOVESPECIFICATION: &str = "RemoveSpecification";
pub const LOG_IAM_RES_OP_DELETE: &str = "Delete";
pub const LOG_IAM_RES_OP_ADDELEMENTAPI: &str = "AddElementApi";
pub const LOG_IAM_RES_OP_MODIFYCONTENT: &str = "ModifyContent";
pub const LOG_IAM_RES_OP_ADDCONTENTPAGEAPI: &str = "AddContentPageApi";
pub const LOG_IAM_RES_OP_REMOVEELEMENTAPI: &str = "RemoveElementApi";
pub const LOG_IAM_RES_OP_REMOVECONTENTPAGEAPI: &str = "RemoveContentPageApi";

pub const LOG_IAM_TENANT_OP_ADD: &str = "Add";
pub const LOG_IAM_TENANT_OP_MODIFY: &str = "Modify";
pub const LOG_IAM_TENANT_OP_DISABLED: &str = "Disabled";
pub const LOG_IAM_TENANT_OP_ENABLED: &str = "Enabled";

pub const LOG_SECURITY_ALARM_OP_MODIFYPASSWORDLENGTH: &str = "ModifyPasswordLength";
pub const LOG_SECURITY_ALARM_OP_MODIFYPASSWORDCOMPLEXITY: &str = "ModifyPasswordComplexity";
pub const LOG_SECURITY_ALARM_OP_MODIFYPASSWORDVALIDTYPERIOD: &str = "ModifyPasswordValidityPeriod";
pub const LOG_SECURITY_ALARM_OP_MODIFYPASSWORDERRORSETTING: &str = "ModifyPasswordErrorSetting";
pub const LOG_SECURITY_ALARM_OP_SETUPUNUSEDACCOUNTSTOLOCK: &str = "SetUpUnusedAccountsToLock";
pub const LOG_SECURITY_ALARM_OP_SETUPSESSIONINVALIDATION: &str = "SetUpSessionInvalidation";
pub const LOG_SECURITY_ALARM_OP_SETUPTMPACCOUNTUSEPERIOD: &str = "SetUpTmpAccountUsePeriod";
pub const LOG_SECURITY_ALARM_OP_MODIFYCERTIFIEDWAY: &str = "ModifyCertifiedWay";

pub const DEFAULT_V_CODE_CD_IN_SEC: u32 = 60;

pub fn get_tardis_inst() -> TardisFunsInst {
    TardisFuns::inst_with_db_conn(COMPONENT_CODE.to_string(), None)
}
