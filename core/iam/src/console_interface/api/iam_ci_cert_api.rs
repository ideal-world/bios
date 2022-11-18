use crate::basic::dto::iam_cert_dto::IamCertAkSkResp;
use crate::console_interface::serv::iam_ci_cert_aksk_serv::IamCiCertAkSkServ;
use crate::iam_constants;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::param::Query;
use tardis::web::poem_openapi::{param::Path, payload::Json, Tags};
use tardis::web::web_resp::{TardisApiResult, TardisResp, Void};

pub struct IamCiCertApi;

/// Interface Console Cert API
#[poem_openapi::OpenApi(prefix_path = "/ci", tag = "bios_basic::ApiTag::Interface")]
impl IamCiCertApi {
    /// add aksk cert
    // #[oai(path = "/aksk", method = "put")]
    async fn add_aksk(&self, app_id: &str, ctx: TardisContextExtractor) -> TardisApiResult<IamCertAkSkResp> {
        let funs = iam_constants::get_tardis_inst();
        let result = IamCiCertAkSkServ::general_cert(app_id, &funs, &ctx.0).await?;
        TardisResp::ok(result)
    }

    // #[oai(path = "/conf/aksk", method = "delete")]
    async fn delete_aksk(&self, id: &str, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let funs = iam_constants::get_tardis_inst();
        let result = IamCiCertAkSkServ::delete_cert(id, &funs, &ctx.0).await?;
        TardisResp::ok(Void)
    }
}
