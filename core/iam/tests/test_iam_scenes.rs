use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;

use bios_iam::basic::dto::iam_account_dto::AccountInfoResp;
use bios_iam::console_passport::dto::iam_cp_cert_dto::IamCpUserPwdLoginReq;
use bios_iam::iam_constants;
use bios_iam::iam_test_helper::BIOSWebTestClient;

pub async fn test(client: &BIOSWebTestClient, sysadmin_name: &str, sysadmin_password: &str) -> TardisResult<()> {
    let account_info: AccountInfoResp = client
        .put(
            "/cp/login/userpwd",
            &IamCpUserPwdLoginReq {
                ak: TrimString(sysadmin_name.to_string()),
                sk: TrimString(sysadmin_password.to_string()),
                tenant_id: None,
                flag: None,
            },
        )
        .await;

    assert_eq!(account_info.account_name, iam_constants::RBUM_ITEM_NAME_SYS_ADMIN_ACCOUNT);
    Ok(())
}
