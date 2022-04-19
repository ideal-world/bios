use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi::{param::Path, param::Query, payload::Json, OpenApi};
use tardis::web::web_resp::{TardisApiResult, TardisPage, TardisResp, Void};

use bios_basic::rbum::dto::rbum_cert_conf_dto::{RbumCertConfDetailResp, RbumCertConfSummaryResp};

use crate::basic::dto::iam_cert_conf_dto::{IamMailVCodeCertConfAddOrModifyReq, IamPhoneVCodeCertConfAddOrModifyReq, IamUserPwdCertConfAddOrModifyReq};
use crate::console_tenant::serv::iam_ct_cert_serv::IamCtCertServ;
use crate::iam_constants;

pub struct IamCtCertConfApi;

/// Tenant Console Cert Config API
#[OpenApi(prefix_path = "/ct/cert-conf", tag = "crate::iam_enumeration::Tag::Tenant")]
impl IamCtCertConfApi {

    /// Modify Cert Config By UserPwd Kind
    #[oai(path = "/:id/user-pwd", method = "put")]
    async fn modify_cert_conf_user_pwd(&self, id: Path<String>, mut modify_req: Json<IamUserPwdCertConfAddOrModifyReq>, cxt: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamCtCertServ::modify_cert_conf_user_pwd(&id.0, &mut modify_req.0, &funs, &cxt.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Add Cert Config By MailVCode Kind
    #[oai(path = "/mail-vcode", method = "post")]
    async fn add_cert_conf_mail_vcode(&self, mut add_req: Json<IamMailVCodeCertConfAddOrModifyReq>, cxt: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamCtCertServ::add_cert_conf_mail_vcode(&mut add_req.0, &funs, &cxt.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Modify Cert Config By MailVCode Kind
    #[oai(path = "/:id/mail-vcode", method = "put")]
    async fn modify_cert_conf_mail_vcode(&self, id: Path<String>, mut modify_req: Json<IamMailVCodeCertConfAddOrModifyReq>, cxt: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamCtCertServ::modify_cert_conf_mail_vcode(&id.0, &mut modify_req.0, &funs, &cxt.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Add Cert Config By PhoneVCode Kind
    #[oai(path = "/phone-vcode", method = "post")]
    async fn add_cert_conf_phone_vcode(&self, mut add_req: Json<IamPhoneVCodeCertConfAddOrModifyReq>, cxt: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamCtCertServ::add_cert_conf_phone_vcode(&mut add_req.0, &funs, &cxt.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Modify Cert Config By PhoneVCode Kind
    #[oai(path = "/:id/phone-vcode", method = "put")]
    async fn modify_cert_conf_phone_vcode(
        &self,
        id: Path<String>,
        mut modify_req: Json<IamPhoneVCodeCertConfAddOrModifyReq>,
        cxt: TardisContextExtractor,
    ) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamCtCertServ::modify_cert_conf_phone_vcode(&id.0, &mut modify_req.0, &funs, &cxt.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Get Cert Config By Id
    #[oai(path = "/:id", method = "get")]
    async fn get_cert_conf(&self, id: Path<String>, cxt: TardisContextExtractor) -> TardisApiResult<RbumCertConfDetailResp> {
        let result = IamCtCertServ::get_cert_conf(&id.0, &iam_constants::get_tardis_inst(), &cxt.0).await?;
        TardisResp::ok(result)
    }

    /// Find Cert Configs
    #[oai(path = "/", method = "get")]
    async fn paginate_cert_conf(
        &self,
        q_id: Query<Option<String>>,
        q_name: Query<Option<String>>,
        desc_by_create: Query<Option<bool>>,
        desc_by_update: Query<Option<bool>>,
        page_number: Query<u64>,
        page_size: Query<u64>,
        cxt: TardisContextExtractor,
    ) -> TardisApiResult<TardisPage<RbumCertConfSummaryResp>> {
        let result = IamCtCertServ::paginate_cert_conf(
            q_id.0,
            q_name.0,
            page_number.0,
            page_size.0,
            desc_by_create.0,
            desc_by_update.0,
            &iam_constants::get_tardis_inst(),
            &cxt.0,
        )
        .await?;
        TardisResp::ok(result)
    }

    /// Delete Cert Config By Id
    #[oai(path = "/:id", method = "delete")]
    async fn delete(&self, id: Path<String>, cxt: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamCtCertServ::delete_cert_conf(&id.0, &funs, &cxt.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }
}
