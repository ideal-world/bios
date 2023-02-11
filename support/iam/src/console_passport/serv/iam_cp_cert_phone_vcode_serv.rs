use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::TardisFunsInst;

use bios_basic::rbum::helper::rbum_scope_helper::get_max_level_id_by_context;
use bios_basic::rbum::serv::rbum_cert_serv::RbumCertServ;

use crate::basic::dto::iam_account_dto::IamAccountInfoResp;
use crate::basic::dto::iam_cert_dto::IamCertPhoneVCodeAddReq;

use crate::basic::serv::iam_cert_phone_vcode_serv::IamCertPhoneVCodeServ;
use crate::basic::serv::iam_cert_serv::IamCertServ;
use crate::console_passport::dto::iam_cp_cert_dto::IamCpPhoneVCodeLoginSendVCodeReq;
use crate::iam_enumeration::IamCertKernelKind;

pub struct IamCpCertPhoneVCodeServ;

impl IamCpCertPhoneVCodeServ {
    pub async fn add_cert_phone_vocde(add_req: &IamCertPhoneVCodeAddReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<String> {
        let rbum_cert_conf_id = IamCertServ::get_cert_conf_id_by_kind(IamCertKernelKind::PhoneVCode.to_string().as_str(), get_max_level_id_by_context(ctx), funs).await?;
        IamCertPhoneVCodeServ::add_cert(add_req, &ctx.owner, &rbum_cert_conf_id, funs, ctx).await
    }

    pub async fn login_by_phone_vocde(login_req: &IamCpPhoneVCodeLoginSendVCodeReq, funs: &TardisFunsInst) -> TardisResult<IamAccountInfoResp> {
        let rbum_cert_conf_id = IamCertServ::get_cert_conf_id_by_kind(&IamCertKernelKind::PhoneVCode.to_string(), Some(login_req.tenant_id.clone()), funs).await?;
        let (_, _, rbum_item_id) = RbumCertServ::validate_by_spec_cert_conf(&login_req.phone, &login_req.vcode.0, &rbum_cert_conf_id, false, &login_req.tenant_id, funs).await?;
        let resp = IamCertServ::package_tardis_context_and_resp(Some(login_req.tenant_id.clone()), &rbum_item_id, login_req.flag.clone(), None, funs).await?;
        Ok(resp)
    }
}
