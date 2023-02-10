use crate::basic::dto::iam_cert_dto::{IamCertAkSkAddReq, IamCertAkSkResp, IamOauth2AkSkResp};
use crate::console_interface::serv::iam_ci_cert_aksk_serv::IamCiCertAkSkServ;
use crate::console_interface::serv::iam_ci_oauth2_token_serv::IamCiOauth2AkSkServ;
use crate::iam_constants;
use crate::iam_enumeration::Oauth2GrantType;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::param::Query;
use tardis::web::poem_openapi::payload::Json;
use tardis::web::web_resp::{TardisApiResult, TardisResp, Void};

pub struct IamCiCertApi;

/// Interface Console Cert API
#[poem_openapi::OpenApi(prefix_path = "/ci", tag = "bios_basic::ApiTag::Interface")]
impl IamCiCertApi {
    /// add aksk cert
    #[oai(path = "/aksk", method = "put")]
    async fn add_aksk(&self, add_req: Json<IamCertAkSkAddReq>, ctx: TardisContextExtractor) -> TardisApiResult<IamCertAkSkResp> {
        let funs = iam_constants::get_tardis_inst();
        let result = IamCiCertAkSkServ::general_cert(add_req.0, &funs, &ctx.0).await?;
        TardisResp::ok(result)
    }

    #[oai(path = "/aksk", method = "delete")]
    async fn delete_aksk(&self, id: Query<String>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let funs = iam_constants::get_tardis_inst();
        IamCiCertAkSkServ::delete_cert(&id.0, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    #[oai(path = "/token", method = "get")]
    async fn get_token(
        &self,
        grant_type: Query<String>,
        client_id: Query<String>,
        client_secret: Query<String>,
        scope: Query<Option<String>>,
    ) -> TardisApiResult<IamOauth2AkSkResp> {
        let grant_type = Oauth2GrantType::parse(&grant_type.0)?;
        let funs = iam_constants::get_tardis_inst();
        let resp = IamCiOauth2AkSkServ::generate_token(grant_type, &client_id.0, &client_secret.0, scope.0, funs).await?;
        TardisResp::ok(resp)
    }
}
