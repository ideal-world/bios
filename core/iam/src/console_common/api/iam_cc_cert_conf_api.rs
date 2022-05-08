use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi::{param::Path, param::Query, payload::Json, OpenApi};
use tardis::web::web_resp::{TardisApiResult, TardisPage, TardisResp, Void};

use bios_basic::rbum::dto::rbum_cert_conf_dto::{RbumCertConfDetailResp, RbumCertConfSummaryResp};
use bios_basic::rbum::helper::rbum_scope_helper::get_max_level_id_by_context;

use crate::basic::dto::iam_cert_conf_dto::{IamMailVCodeCertConfAddOrModifyReq, IamPhoneVCodeCertConfAddOrModifyReq, IamUserPwdCertConfAddOrModifyReq};
use crate::basic::serv::iam_cert_mail_vcode_serv::IamCertMailVCodeServ;
use crate::basic::serv::iam_cert_phone_vcode_serv::IamCertPhoneVCodeServ;
use crate::basic::serv::iam_cert_serv::IamCertServ;
use crate::basic::serv::iam_cert_user_pwd_serv::IamCertUserPwdServ;
use crate::iam_constants;

pub struct IamCcCertConfApi;

/// Common Console Cert Config API
#[OpenApi(prefix_path = "/cc/cert-conf", tag = "crate::iam_enumeration::Tag::Common")]
impl IamCcCertConfApi {
    /// Modify Cert Config By UserPwd Kind
    #[oai(path = "/:id/user-pwd", method = "put")]
    async fn modify_cert_conf_user_pwd(&self, id: Path<String>, mut modify_req: Json<IamUserPwdCertConfAddOrModifyReq>, cxt: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamCertUserPwdServ::modify_cert_conf(&id.0, &mut modify_req.0, &funs, &cxt.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Add Cert Config By MailVCode Kind
    #[oai(path = "/mail-vcode", method = "post")]
    async fn add_cert_conf_mail_vcode(&self, mut add_req: Json<IamMailVCodeCertConfAddOrModifyReq>, cxt: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamCertMailVCodeServ::add_cert_conf(&mut add_req.0, get_max_level_id_by_context(&cxt.0), &funs, &cxt.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Modify Cert Config By MailVCode Kind
    #[oai(path = "/:id/mail-vcode", method = "put")]
    async fn modify_cert_conf_mail_vcode(&self, id: Path<String>, mut modify_req: Json<IamMailVCodeCertConfAddOrModifyReq>, cxt: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamCertMailVCodeServ::modify_cert_conf(&id.0, &mut modify_req.0, &funs, &cxt.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Add Cert Config By PhoneVCode Kind
    #[oai(path = "/phone-vcode", method = "post")]
    async fn add_cert_conf_phone_vcode(&self, mut add_req: Json<IamPhoneVCodeCertConfAddOrModifyReq>, cxt: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamCertPhoneVCodeServ::add_cert_conf(&mut add_req.0, get_max_level_id_by_context(&cxt.0), &funs, &cxt.0).await?;
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
        IamCertPhoneVCodeServ::modify_cert_conf(&id.0, &mut modify_req.0, &funs, &cxt.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Get Cert Config By Id
    #[oai(path = "/:id", method = "get")]
    async fn get_cert_conf(&self, id: Path<String>, cxt: TardisContextExtractor) -> TardisApiResult<RbumCertConfDetailResp> {
        let funs = iam_constants::get_tardis_inst();
        let result = IamCertServ::get_cert_conf(&id.0, get_max_level_id_by_context(&cxt.0), &funs, &cxt.0).await?;
        TardisResp::ok(result)
    }

    /// Find Cert Configs
    #[oai(path = "/", method = "get")]
    async fn paginate_cert_conf(
        &self,
        q_id: Query<Option<String>>,
        q_name: Query<Option<String>>,
        page_number: Query<u64>,
        page_size: Query<u64>,
        desc_by_create: Query<Option<bool>>,
        desc_by_update: Query<Option<bool>>,
        cxt: TardisContextExtractor,
    ) -> TardisApiResult<TardisPage<RbumCertConfSummaryResp>> {
        let funs = iam_constants::get_tardis_inst();
        let result = IamCertServ::paginate_cert_conf(
            q_id.0,
            q_name.0,
            get_max_level_id_by_context(&cxt.0),
            page_number.0,
            page_size.0,
            desc_by_create.0,
            desc_by_update.0,
            &funs,
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
        IamCertServ::delete_cert_conf(&id.0, &funs, &cxt.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }
}
