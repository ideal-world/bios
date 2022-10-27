use tardis::basic::result::TardisResult;
use tardis::{TardisFuns, TardisFunsInst};

use bios_basic::rbum::serv::rbum_cert_serv::RbumCertServ;

use crate::basic::serv::iam_cert_serv::IamCertServ;
use crate::console_passport::serv::iam_cp_cert_user_pwd_serv::IamCpCertUserPwdServ;
use crate::iam_constants;
use crate::iam_enumeration::IamCertKernelKind;

pub async fn check_exist(account_name_with_tenant: &str) -> TardisResult<bool> {
    //Ok(true)
    let funs = iam_constants::get_tardis_inst();
    let (tenant_id, ak) = get_basic_info(account_name_with_tenant, &funs).await?;
    let rbum_cert_conf_id = IamCertServ::get_cert_conf_id_by_code(&IamCertKernelKind::UserPwd.to_string(), Some(tenant_id.clone()), &funs).await?;
    RbumCertServ::check_exist(&ak, &rbum_cert_conf_id, Some(tenant_id), &funs).await
}

pub async fn check_cert(account_name_with_tenant: &str, pwd: &str) -> TardisResult<bool> {
    //Ok(true)
    let funs = iam_constants::get_tardis_inst();
    let (tenant_id, ak) = get_basic_info(account_name_with_tenant, &funs).await?;
    let rbum_cert_conf_id = IamCertServ::get_cert_conf_id_by_code(&IamCertKernelKind::UserPwd.to_string(), Some(tenant_id.clone()), &funs).await?;
    match RbumCertServ::validate_by_spec_cert_conf(&ak, pwd, &rbum_cert_conf_id, false, &tenant_id, &funs).await {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

// pub async fn get_account() -> TardisResult<bool> {
//
// }

async fn get_basic_info<'a>(account_name_with_tenant: &str, funs: &TardisFunsInst) -> TardisResult<(String, String)> {
    let mut account_name_with_tenant = account_name_with_tenant.split('/');
    let (tenant_id, ak) = if account_name_with_tenant.clone().count() == 2 {
        (
            // Ensure case sensitivity
            Some(String::from_utf8(TardisFuns::crypto.hex.decode(account_name_with_tenant.next().unwrap())?)?),
            account_name_with_tenant.next().unwrap().to_string(),
        )
    } else {
        (None, account_name_with_tenant.next().unwrap().to_string())
    };
    let tenant_id = IamCpCertUserPwdServ::get_tenant_id(tenant_id, funs).await?;
    Ok((tenant_id, ak))
}
