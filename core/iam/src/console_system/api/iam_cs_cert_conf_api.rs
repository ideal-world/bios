use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi::{param::Path, param::Query, payload::Json, OpenApi};
use tardis::web::web_resp::{TardisApiResult, TardisResp, Void};

use bios_basic::rbum::dto::rbum_cert_conf_dto::RbumCertConfDetailResp;
use bios_basic::rbum::dto::rbum_filer_dto::{RbumBasicFilterReq, RbumCertConfFilterReq};
use bios_basic::rbum::serv::rbum_cert_serv::RbumCertConfServ;
use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;

use crate::basic::dto::iam_cert_conf_dto::{IamMailVCodeCertConfAddOrModifyReq, IamPhoneVCodeCertConfAddOrModifyReq, IamUserPwdCertConfAddOrModifyReq};
use crate::basic::serv::iam_cert_mail_vcode_serv::IamCertMailVCodeServ;
use crate::basic::serv::iam_cert_phone_vcode_serv::IamCertPhoneVCodeServ;
use crate::basic::serv::iam_cert_serv::IamCertServ;
use crate::basic::serv::iam_cert_user_pwd_serv::IamCertUserPwdServ;
use crate::iam_config::IamBasicInfoManager;
use crate::iam_constants;
use crate::iam_enumeration::IamCertKind;

pub struct IamCsCertConfApi;

/// System Console Cert Config API
#[OpenApi(prefix_path = "/cs/cert-conf", tag = "crate::iam_enumeration::Tag::System")]
impl IamCsCertConfApi {
    /// Find Cert Conf by Tenant Id
    #[oai(path = "/", method = "get")]
    async fn find_cert_conf(&self, tenant_id: Query<String>, cxt: TardisContextExtractor) -> TardisApiResult<Vec<RbumCertConfDetailResp>> {
        let funs = iam_constants::get_tardis_inst();
        let result = RbumCertConfServ::find_detail_rbums(
            &RbumCertConfFilterReq {
                basic: RbumBasicFilterReq {
                    own_paths: Some("".to_string()),
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                rel_rbum_domain_id: Some(IamBasicInfoManager::get().domain_iam_id.to_string()),
                rel_rbum_item_id: Some(tenant_id.0),
                ..Default::default()
            },
            None,
            None,
            &funs,
            &cxt.0,
        )
        .await?
        .into_iter()
        .filter(|r| r.code == IamCertKind::UserPwd.to_string() || r.code == IamCertKind::PhoneVCode.to_string() || r.code == IamCertKind::MailVCode.to_string())
        .collect();
        TardisResp::ok(result)
    }

    /// Modify Cert Config By UserPwd Kind
    #[oai(path = "/:id/user-pwd", method = "put")]
    async fn modify_cert_conf_user_pwd(&self, id: Path<String>, modify_req: Json<IamUserPwdCertConfAddOrModifyReq>, cxt: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamCertUserPwdServ::modify_cert_conf(&id.0, &modify_req.0, &funs, &cxt.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Add Cert Config By MailVCode Kind
    #[oai(path = "/mail-vcode", method = "post")]
    async fn add_cert_conf_mail_vcode(&self, tenant_id: Query<String>, add_req: Json<IamMailVCodeCertConfAddOrModifyReq>, cxt: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamCertMailVCodeServ::add_cert_conf(&add_req.0, Some(tenant_id.0), &funs, &cxt.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Modify Cert Config By MailVCode Kind
    #[oai(path = "/:id/mail-vcode", method = "put")]
    async fn modify_cert_conf_mail_vcode(&self, id: Path<String>, modify_req: Json<IamMailVCodeCertConfAddOrModifyReq>, cxt: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamCertMailVCodeServ::modify_cert_conf(&id.0, &modify_req.0, &funs, &cxt.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Add Cert Config By PhoneVCode Kind
    #[oai(path = "/phone-vcode", method = "post")]
    async fn add_cert_conf_phone_vcode(&self, tenant_id: Query<String>, add_req: Json<IamPhoneVCodeCertConfAddOrModifyReq>, cxt: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamCertPhoneVCodeServ::add_cert_conf(&add_req.0, Some(tenant_id.0), &funs, &cxt.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Modify Cert Config By PhoneVCode Kind
    #[oai(path = "/:id/phone-vcode", method = "put")]
    async fn modify_cert_conf_phone_vcode(&self, id: Path<String>, modify_req: Json<IamPhoneVCodeCertConfAddOrModifyReq>, cxt: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamCertPhoneVCodeServ::modify_cert_conf(&id.0, &modify_req.0, &funs, &cxt.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Delete Cert Config By Id
    #[oai(path = "/:id", method = "delete")]
    async fn delete(&self, id: Path<String>, cxt: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamCertServ::delete_cert_conf(&id.0, &funs, &cxt.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }
}
