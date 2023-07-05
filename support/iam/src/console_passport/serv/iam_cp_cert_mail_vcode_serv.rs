use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::TardisFunsInst;

use bios_basic::rbum::helper::rbum_scope_helper::get_max_level_id_by_context;

use crate::basic::dto::iam_account_dto::IamAccountInfoResp;
use crate::basic::dto::iam_cert_dto::IamCertMailVCodeAddReq;
use crate::basic::serv::iam_cert_mail_vcode_serv::IamCertMailVCodeServ;
use crate::basic::serv::iam_cert_serv::IamCertServ;
use crate::console_passport::dto::iam_cp_cert_dto::IamCpMailVCodeLoginReq;
use crate::iam_enumeration::IamCertKernelKind;

pub struct IamCpCertMailVCodeServ;

impl IamCpCertMailVCodeServ {
    pub async fn add_cert_mail_vocde(add_req: &IamCertMailVCodeAddReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<String> {
        let rbum_cert_conf_id = IamCertServ::get_cert_conf_id_by_kind(IamCertKernelKind::MailVCode.to_string().as_str(), get_max_level_id_by_context(ctx), funs).await?;
        IamCertMailVCodeServ::add_cert(add_req, &ctx.owner, &rbum_cert_conf_id, funs, ctx).await
    }

    pub async fn login_by_mail_vocde(login_req: &IamCpMailVCodeLoginReq, funs: &TardisFunsInst) -> TardisResult<IamAccountInfoResp> {
        let global_rbum_cert_conf_id = IamCertServ::get_cert_conf_id_by_kind(&IamCertKernelKind::MailVCode.to_string(), None, funs).await?;
        if let Some(tenant_id) = &login_req.tenant_id {
            let rbum_cert_conf_id = IamCertServ::get_cert_conf_id_by_kind(&IamCertKernelKind::MailVCode.to_string(), Some(tenant_id.clone()), funs).await?;
            let result = IamCertServ::validate_by_ak_and_sk(
                &login_req.mail,
                &login_req.vcode.0,
                Some(&rbum_cert_conf_id),
                None,
                false,
                Some(tenant_id.clone()),
                None,
                funs,
            )
            .await;
            let (_, _, rbum_item_id) = if let Some(e) = result.clone().err() {
                if e.code == "401-iam-cert-valid" {
                    IamCertServ::validate_by_ak_and_sk(
                        &login_req.mail,
                        &login_req.vcode.0,
                        Some(&global_rbum_cert_conf_id),
                        None,
                        false,
                        Some("".to_string()),
                        None,
                        funs,
                    )
                    .await?
                } else {
                    result?
                }
            } else {
                result?
            };
            let resp = IamCertServ::package_tardis_context_and_resp(Some(tenant_id.to_string()), &rbum_item_id, login_req.flag.clone(), None, funs).await?;
            Ok(resp)
        } else {
            let (_, _, rbum_item_id) = IamCertServ::validate_by_ak_and_sk(
                &login_req.mail,
                &login_req.vcode.0,
                Some(&global_rbum_cert_conf_id),
                None,
                false,
                Some("".to_string()),
                None,
                funs,
            )
            .await?;
            let resp = IamCertServ::package_tardis_context_and_resp(None, &rbum_item_id, login_req.flag.clone(), None, funs).await?;
            Ok(resp)
        }
    }
}
