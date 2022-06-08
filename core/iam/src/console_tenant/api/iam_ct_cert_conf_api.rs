use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi::{param::Path, payload::Json, OpenApi};
use tardis::web::web_resp::{TardisApiResult, TardisResp, Void};

use bios_basic::rbum::dto::rbum_cert_conf_dto::RbumCertConfDetailResp;

use crate::basic::dto::iam_cert_conf_dto::{IamMailVCodeCertConfAddOrModifyReq, IamPhoneVCodeCertConfAddOrModifyReq, IamUserPwdCertConfAddOrModifyReq};
use crate::basic::serv::iam_cert_mail_vcode_serv::IamCertMailVCodeServ;
use crate::basic::serv::iam_cert_phone_vcode_serv::IamCertPhoneVCodeServ;
use crate::basic::serv::iam_cert_serv::IamCertServ;
use crate::basic::serv::iam_cert_user_pwd_serv::IamCertUserPwdServ;
use crate::basic::serv::iam_tenant_serv::IamTenantServ;
use crate::iam_constants;

pub struct IamCtCertConfApi;

/// Tenant Console Cert Conf API
///
/// Note: the current cert conf only supports sys & tenant levels.
#[OpenApi(prefix_path = "/ct/cert-conf", tag = "crate::iam_enumeration::Tag::Tenant")]
impl IamCtCertConfApi {
    /// Find Cert Conf By Current Tenant
    #[oai(path = "/", method = "get")]
    async fn find_cert_conf(&self, cxt: TardisContextExtractor) -> TardisApiResult<Vec<RbumCertConfDetailResp>> {
        let funs = iam_constants::get_tardis_inst();
        let result =
            IamCertServ::find_cert_conf_detail_without_token_kind(None, None, None, true, Some(IamTenantServ::get_id_by_cxt(&cxt.0, &funs)?), None, None, &funs, &cxt.0).await?;
        TardisResp::ok(result)
    }

    /// Modify UserPwd Kind Cert Conf By Cert Conf Id
    #[oai(path = "/:id/user-pwd", method = "put")]
    async fn modify_cert_conf_user_pwd(&self, id: Path<String>, modify_req: Json<IamUserPwdCertConfAddOrModifyReq>, cxt: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamCertUserPwdServ::modify_cert_conf(&id.0, &modify_req.0, &funs, &cxt.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Add MailVCode Kind Cert Conf By Current Tenant
    #[oai(path = "/mail-vcode", method = "post")]
    async fn add_cert_conf_mail_vcode(&self, add_req: Json<IamMailVCodeCertConfAddOrModifyReq>, cxt: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamCertMailVCodeServ::add_cert_conf(&add_req.0, Some(IamTenantServ::get_id_by_cxt(&cxt.0, &funs)?), &funs, &cxt.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Modify MailVCode Kind Cert Conf By Cert Conf Id
    #[oai(path = "/:id/mail-vcode", method = "put")]
    async fn modify_cert_conf_mail_vcode(&self, id: Path<String>, modify_req: Json<IamMailVCodeCertConfAddOrModifyReq>, cxt: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamCertMailVCodeServ::modify_cert_conf(&id.0, &modify_req.0, &funs, &cxt.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Add PhoneVCode Kind Cert Conf By Current Tenant
    #[oai(path = "/phone-vcode", method = "post")]
    async fn add_cert_conf_phone_vcode(&self, add_req: Json<IamPhoneVCodeCertConfAddOrModifyReq>, cxt: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamCertPhoneVCodeServ::add_cert_conf(&add_req.0, Some(IamTenantServ::get_id_by_cxt(&cxt.0, &funs)?), &funs, &cxt.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Modify PhoneVCode Kind Cert Conf By Cert Conf Id
    #[oai(path = "/:id/phone-vcode", method = "put")]
    async fn modify_cert_conf_phone_vcode(&self, id: Path<String>, modify_req: Json<IamPhoneVCodeCertConfAddOrModifyReq>, cxt: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamCertPhoneVCodeServ::modify_cert_conf(&id.0, &modify_req.0, &funs, &cxt.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Delete Cert Conf By Cert Conf Id
    #[oai(path = "/:id", method = "delete")]
    async fn delete(&self, id: Path<String>, cxt: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamCertServ::delete_cert_conf(&id.0, &funs, &cxt.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }
}
