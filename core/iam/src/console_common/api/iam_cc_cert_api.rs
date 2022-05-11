use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi::{OpenApi, param::Path, payload::Json};
use tardis::web::web_resp::{TardisApiResult, TardisResp, Void};

use bios_basic::rbum::helper::rbum_scope_helper::get_max_level_id_by_context;

use crate::basic::dto::iam_cert_dto::IamUserPwdCertRestReq;
use crate::basic::serv::iam_cert_serv::IamCertServ;
use crate::basic::serv::iam_cert_user_pwd_serv::IamCertUserPwdServ;
use crate::iam_constants;
use crate::iam_enumeration::IamCertKind;

pub struct IamCcCertApi;

/// Common Console Cert API
#[OpenApi(prefix_path = "/cc/cert", tag = "crate::iam_enumeration::Tag::Common")]
impl IamCcCertApi {
    /// Rest Password
    #[oai(path = "/user-pwd/:account_id", method = "put")]
    async fn rest_password(&self, account_id: Path<String>, modify_req: Json<IamUserPwdCertRestReq>, cxt: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let rbum_cert_conf_id = IamCertServ::get_cert_conf_id_by_code(IamCertKind::UserPwd.to_string().as_str(), get_max_level_id_by_context(&cxt.0), &funs).await?;
        IamCertUserPwdServ::reset_sk(&modify_req.0, &account_id.0, &rbum_cert_conf_id, &funs, &cxt.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }
}
