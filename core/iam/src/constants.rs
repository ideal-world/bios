use tardis::basic::result::TardisResult;

pub const RBUM_KIND_SCHEME_IAM_TENANT: &str = bios_basic::rbum::constants::RBUM_KIND_SCHEME_IAM_TENANT;
pub const RBUM_KIND_SCHEME_IAM_APP: &str = bios_basic::rbum::constants::RBUM_KIND_SCHEME_IAM_APP;
pub const RBUM_KIND_SCHEME_IAM_ACCOUNT: &str = bios_basic::rbum::constants::RBUM_KIND_SCHEME_IAM_ACCOUNT;
pub const RBUM_KIND_SCHEME_IAM_ROLE: &str = "iam_role";
pub const RBUM_KIND_SCHEME_IAM_GROUP: &str = "iam_group";
pub const RBUM_KIND_SCHEME_IAM_RES_HTTP: &str = "iam_res_http";

static mut BASIC_INFO: BasicInfo = BasicInfo { info: None };

#[derive(Debug)]
struct BasicInfo {
    pub info: Option<BasicInfoPub>,
}

#[derive(Debug)]
pub struct BasicInfoPub {
    pub rbum_tenant_kind_id: String,
    pub rbum_app_kind_id: String,
    pub rbum_account_kind_id: String,
    pub rbum_role_kind_id: String,
    pub rbum_group_kind_id: String,
    pub rbum_res_http_kind_id: String,
    pub rbum_iam_domain_id: String,
}

pub fn set_basic_info(basic_info: BasicInfoPub) -> TardisResult<()> {
    unsafe {
        BASIC_INFO.info = Some(basic_info);
    }
    Ok(())
}

pub fn get_rbum_basic_info() -> &'static BasicInfoPub {
    unsafe {
        match BASIC_INFO.info {
            Some(ref info) => info,
            None => panic!("Basic info not set"),
        }
    }
}
