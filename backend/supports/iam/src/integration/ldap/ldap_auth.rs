//! LDAP Authentication Handler
//!
//! 负责处理LDAP认证相关的逻辑

use tardis::basic::result::TardisResult;

use crate::basic::serv::iam_cert_serv::IamCertServ;
use crate::iam_constants;
use crate::iam_enumeration::IamCertKernelKind;

/// 验证账户凭证
pub async fn check_cert(ak: &str, pwd: &str) -> TardisResult<bool> {
    let funs = iam_constants::get_tardis_inst();
    let rbum_cert_conf_id = IamCertServ::get_cert_conf_id_by_kind(&IamCertKernelKind::UserPwd.to_string(), Some("".to_string()), &funs).await?;
    match IamCertServ::validate_by_ak_and_sk(ak, pwd, Some(&rbum_cert_conf_id), None, false, Some("".to_string()), None, None, &funs).await {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}
