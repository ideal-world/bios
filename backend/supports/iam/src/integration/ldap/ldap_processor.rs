use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;
use tardis::basic::result::TardisResult;
use tardis::{TardisFuns, TardisFunsInst};

use bios_basic::rbum::dto::rbum_filer_dto::{RbumBasicFilterReq, RbumCertFilterReq};
use bios_basic::rbum::serv::rbum_cert_serv::RbumCertServ;

use crate::basic::dto::iam_account_dto::IamAccountDetailAggResp;
use crate::basic::dto::iam_filer_dto::IamAccountFilterReq;
use crate::basic::serv::iam_account_serv::IamAccountServ;
use crate::basic::serv::iam_cert_serv::IamCertServ;
use crate::console_passport::serv::iam_cp_cert_user_pwd_serv::IamCpCertUserPwdServ;
use crate::iam_config::IamLdapConfig;
use crate::iam_constants;
use crate::iam_enumeration::IamCertKernelKind;

pub async fn check_exist(account_name_with_tenant: &str) -> TardisResult<bool> {
    //Ok(true)
    let funs = iam_constants::get_tardis_inst();
    let (tenant_id, ak) = get_basic_info(account_name_with_tenant, &funs).await?;
    let rbum_cert_conf_id = IamCertServ::get_cert_conf_id_by_kind(&IamCertKernelKind::UserPwd.to_string(), Some(tenant_id.clone()), &funs).await?;
    RbumCertServ::check_exist(&ak, &rbum_cert_conf_id, &tenant_id, &funs).await
}

pub async fn check_cert(account_name_with_tenant: &str, pwd: &str) -> TardisResult<bool> {
    //Ok(true)
    let funs = iam_constants::get_tardis_inst();
    let (tenant_id, ak) = get_basic_info(account_name_with_tenant, &funs).await?;
    let rbum_cert_conf_id = IamCertServ::get_cert_conf_id_by_kind(&IamCertKernelKind::UserPwd.to_string(), Some(tenant_id.clone()), &funs).await?;
    match IamCertServ::validate_by_ak_and_sk(&ak, pwd, Some(&rbum_cert_conf_id), None, false, Some(tenant_id.clone()), None, None, &funs).await {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

pub async fn get_account_detail(account_name_with_tenant: &str) -> TardisResult<Option<IamAccountDetailAggResp>> {
    let funs = iam_constants::get_tardis_inst();
    let (tenant_id, ak) = get_basic_info(account_name_with_tenant, &funs).await?;
    let rbum_cert_conf_id = IamCertServ::get_cert_conf_id_by_kind(&IamCertKernelKind::UserPwd.to_string(), Some(tenant_id.clone()), &funs).await?;
    
    let ctx = IamCertServ::try_use_tenant_ctx(Default::default(), Some(tenant_id.clone()))?;
    
    if let Some(cert) = RbumCertServ::find_one_detail_rbum(
        &RbumCertFilterReq {
            basic: RbumBasicFilterReq {
                own_paths: Some(tenant_id.clone()),
                ..Default::default()
            },
            ak: Some(ak),
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

async fn get_basic_info<'a>(account_name_with_tenant: &str, funs: &TardisFunsInst) -> TardisResult<(String, String)> {
    let mut account_name_with_tenant = account_name_with_tenant.split('/');
    let (tenant_id, ak) = if account_name_with_tenant.clone().count() == 2 {
        (
            // Ensure case sensitivity
            Some(String::from_utf8(TardisFuns::crypto.hex.decode(account_name_with_tenant.next().unwrap_or_default())?)?),
            account_name_with_tenant.next().unwrap_or_default().to_string(),
        )
    } else {
        (None, account_name_with_tenant.next().unwrap_or_default().to_string())
    };
    let tenant_id = IamCpCertUserPwdServ::get_tenant_id(tenant_id, funs).await?;
    Ok((tenant_id, ak))
}

/// Get labor_type label from config by code
pub fn get_labor_type_label(labor_type_code: &str, config: &IamLdapConfig) -> String {
    if labor_type_code.is_empty() {
        return String::new();
    }
    if let Some(ref labor_type_map) = config.labor_type_map {
        labor_type_map
            .get(labor_type_code)
            .cloned()
            .unwrap_or_else(|| labor_type_code.to_string())
    } else {
        labor_type_code.to_string()
    }
}

/// Get position label from config by code
pub fn get_position_label(position_code: &str, config: &IamLdapConfig) -> String {
    if position_code.is_empty() {
        return String::new();
    }
    if let Some(ref position_map) = config.position_map {
        position_map
            .get(position_code)
            .cloned()
            .unwrap_or_else(|| position_code.to_string())
    } else {
        position_code.to_string()
    }
}
