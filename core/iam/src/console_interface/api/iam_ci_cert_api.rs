use crate::console_interface::serv::iam_ci_cert_aksk_serv::IamCiCertAkSkServ;
use crate::iam_constants;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::web_resp::{TardisApiResult, TardisResp};
use crate::basic::dto::iam_cert_dto::IamCertAkSkResp;

pub struct IamCiCertApi;

/// Interface Console Cert API
#[poem_openapi::OpenApi(prefix_path = "/ci", tag = "bios_basic::ApiTag::Interface")]
impl IamCiCertApi {
    /// add aksk cert
    #[oai(path = "/aksk", method = "put")]
    async fn aksk(&self, app_id: &str, ctx: TardisContextExtractor) -> TardisApiResult<IamCertAkSkResp> {
        let funs = iam_constants::get_tardis_inst();
        let result = IamCiCertAkSkServ::general_cert(app_id, &funs, &ctx.0).await?;
        TardisResp::ok(result)
    }
}
