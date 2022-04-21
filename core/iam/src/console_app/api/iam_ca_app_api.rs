use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi::{payload::Json, OpenApi};
use tardis::web::web_resp::{TardisApiResult, TardisResp, Void};

use crate::basic::dto::iam_app_dto::IamAppDetailResp;
use crate::console_app::dto::iam_ca_app_dto::IamCaAppModifyReq;
use crate::console_app::serv::iam_ca_app_serv::IamCaAppServ;
use crate::iam_constants;

pub struct IamCtAppApi;

/// App Console App API
#[OpenApi(prefix_path = "/ca/app", tag = "crate::iam_enumeration::Tag::App")]
impl IamCtAppApi {
    /// Modify Current App
    #[oai(path = "/", method = "put")]
    async fn modify(&self, mut modify_req: Json<IamCaAppModifyReq>, cxt: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamCaAppServ::modify_app(&mut modify_req.0, &funs, &cxt.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Get Current App
    #[oai(path = "/", method = "get")]
    async fn get(&self, cxt: TardisContextExtractor) -> TardisApiResult<IamAppDetailResp> {
        let result = IamCaAppServ::get_app(&iam_constants::get_tardis_inst(), &cxt.0).await?;
        TardisResp::ok(result)
    }
}
