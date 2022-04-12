use tardis::basic::dto::{TardisContext, TardisFunsInst};
use tardis::basic::result::TardisResult;
use tardis::web::web_resp::TardisPage;

use bios_basic::rbum::dto::rbum_cert_conf_dto::{RbumCertConfDetailResp, RbumCertConfSummaryResp};

use crate::basic::dto::iam_cert_conf_dto::{IamMailVCodeCertConfAddOrModifyReq, IamPhoneVCodeCertConfAddOrModifyReq, IamUserPwdCertConfAddOrModifyReq};
use crate::basic::serv::iam_cert_mail_vcode_serv::IamCertMailVCodeServ;
use crate::basic::serv::iam_cert_phone_vcode_serv::IamCertPhoneVCodeServ;
use crate::basic::serv::iam_cert_serv::IamCertServ;
use crate::basic::serv::iam_cert_user_pwd_serv::IamCertUserPwdServ;
use crate::basic::serv::iam_role_serv::IamRoleServ;
use crate::basic::serv::iam_tenant_serv::IamTenantServ;

pub struct IamCtCertServ;

impl<'a> IamCtCertServ {
    pub async fn add_cert_conf_user_pwd(add_req: &mut IamUserPwdCertConfAddOrModifyReq, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<()> {
        IamRoleServ::need_tenant_admin(funs, cxt).await?;
        IamCertUserPwdServ::add_cert_conf(add_req, Some(IamTenantServ::get_id_by_cxt(cxt)?), funs, cxt).await
    }

    pub async fn modify_cert_conf_user_pwd(id: &str, modify_req: &mut IamUserPwdCertConfAddOrModifyReq, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<()> {
        IamRoleServ::need_tenant_admin(funs, cxt).await?;
        IamCertUserPwdServ::modify_cert_conf(id, modify_req, funs, cxt).await
    }

    pub async fn add_cert_conf_mail_vcode(add_req: &mut IamMailVCodeCertConfAddOrModifyReq, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<()> {
        IamRoleServ::need_tenant_admin(funs, cxt).await?;
        IamCertMailVCodeServ::add_cert_conf(add_req, Some(IamTenantServ::get_id_by_cxt(cxt)?), funs, cxt).await
    }

    pub async fn modify_cert_conf_mail_vcode(id: &str, modify_req: &mut IamMailVCodeCertConfAddOrModifyReq, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<()> {
        IamRoleServ::need_tenant_admin(funs, cxt).await?;
        IamCertMailVCodeServ::modify_cert_conf(id, modify_req, funs, cxt).await
    }

    pub async fn add_cert_conf_phone_vcode(add_req: &mut IamPhoneVCodeCertConfAddOrModifyReq, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<()> {
        IamRoleServ::need_tenant_admin(funs, cxt).await?;
        IamCertPhoneVCodeServ::add_cert_conf(add_req, Some(IamTenantServ::get_id_by_cxt(cxt)?), funs, cxt).await
    }

    pub async fn modify_cert_conf_phone_vcode(id: &str, modify_req: &mut IamPhoneVCodeCertConfAddOrModifyReq, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<()> {
        IamRoleServ::need_tenant_admin(funs, cxt).await?;
        IamCertPhoneVCodeServ::modify_cert_conf(id, modify_req, funs, cxt).await
    }

    pub async fn get_cert_conf(id: &str, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<RbumCertConfDetailResp> {
        IamRoleServ::need_tenant_admin(funs, cxt).await?;
        IamCertServ::get_cert_conf(id, Some(IamTenantServ::get_id_by_cxt(cxt)?), funs, cxt).await
    }

    pub async fn paginate_cert_conf(
        q_name: Option<String>,
        page_number: u64,
        page_size: u64,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<TardisPage<RbumCertConfSummaryResp>> {
        IamRoleServ::need_tenant_admin(funs, cxt).await?;
        IamCertServ::paginate_cert_conf(
            q_name,
            Some(IamTenantServ::get_id_by_cxt(cxt)?),
            page_number,
            page_size,
            desc_sort_by_create,
            desc_sort_by_update,
            funs,
            cxt,
        )
        .await
    }

    pub async fn delete_cert_conf(id: &str, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<u64> {
        IamRoleServ::need_tenant_admin(funs, cxt).await?;
        IamCertServ::delete_cert_conf(id, funs, cxt).await
    }
}
