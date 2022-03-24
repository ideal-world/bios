use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::db::reldb_client::TardisRelDBlConnection;
use tardis::web::web_resp::TardisPage;

use bios_basic::rbum::dto::rbum_cert_conf_dto::{RbumCertConfDetailResp, RbumCertConfSummaryResp};

use crate::basic::dto::iam_cert_conf_dto::{IamMailVCodeCertConfAddOrModifyReq, IamPhoneVCodeCertConfAddOrModifyReq, IamUserPwdCertConfAddOrModifyReq};
use crate::basic::serv::iam_cert_serv::IamCertServ;
use crate::basic::serv::iam_role_serv::IamRoleServ;
use crate::basic::serv::iam_tenant_serv::IamTenantServ;

pub struct IamCtCertServ;

impl<'a> IamCtCertServ {
    pub async fn add_cert_conf_user_pwd(add_req: &mut IamUserPwdCertConfAddOrModifyReq, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<()> {
        IamRoleServ::need_tenant_admin(db, cxt).await?;
        IamCertServ::add_cert_conf_user_pwd(add_req, Some(IamTenantServ::get_id_by_cxt(cxt)?), db, cxt).await
    }

    pub async fn modify_cert_conf_user_pwd(id: &str, modify_req: &mut IamUserPwdCertConfAddOrModifyReq, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<()> {
        IamRoleServ::need_tenant_admin(db, cxt).await?;
        IamCertServ::modify_cert_conf_user_pwd(id, modify_req, db, cxt).await
    }

    pub async fn add_cert_conf_mail_vcode(add_req: &mut IamMailVCodeCertConfAddOrModifyReq, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<()> {
        IamRoleServ::need_tenant_admin(db, cxt).await?;
        IamCertServ::add_cert_conf_mail_vcode(add_req, Some(IamTenantServ::get_id_by_cxt(cxt)?), db, cxt).await
    }

    pub async fn modify_cert_conf_mail_vcode(
        id: &str,
        modify_req: &mut IamMailVCodeCertConfAddOrModifyReq,
        db: &TardisRelDBlConnection<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<()> {
        IamRoleServ::need_tenant_admin(db, cxt).await?;
        IamCertServ::modify_cert_conf_mail_vcode(id, modify_req, db, cxt).await
    }

    pub async fn add_cert_conf_phone_vcode(add_req: &mut IamPhoneVCodeCertConfAddOrModifyReq, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<()> {
        IamRoleServ::need_tenant_admin(db, cxt).await?;
        IamCertServ::add_cert_conf_phone_vcode(add_req, Some(IamTenantServ::get_id_by_cxt(cxt)?), db, cxt).await
    }

    pub async fn modify_cert_conf_phone_vcode(
        id: &str,
        modify_req: &mut IamPhoneVCodeCertConfAddOrModifyReq,
        db: &TardisRelDBlConnection<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<()> {
        IamRoleServ::need_tenant_admin(db, cxt).await?;
        IamCertServ::modify_cert_conf_phone_vcode(id, modify_req, db, cxt).await
    }

    pub async fn get_cert_conf(id: &str, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<RbumCertConfDetailResp> {
        IamRoleServ::need_tenant_admin(db, cxt).await?;
        IamCertServ::get_cert_conf(id, Some(IamTenantServ::get_id_by_cxt(cxt)?), db, cxt).await
    }

    pub async fn paginate_cert_conf(
        q_name: Option<String>,
        page_number: u64,
        page_size: u64,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        db: &TardisRelDBlConnection<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<TardisPage<RbumCertConfSummaryResp>> {
        IamRoleServ::need_tenant_admin(db, cxt).await?;
        IamCertServ::paginate_cert_conf(
            q_name,
            Some(IamTenantServ::get_id_by_cxt(cxt)?),
            page_number,
            page_size,
            desc_sort_by_create,
            desc_sort_by_update,
            db,
            cxt,
        )
        .await
    }

    pub async fn delete_cert_conf(id: &str, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext)
        -> TardisResult<u64> {
        IamRoleServ::need_tenant_admin(db, cxt).await?;
        IamCertServ::delete_cert_conf(id, db, cxt).await
    }
}
