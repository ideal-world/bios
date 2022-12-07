use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem::web::Path;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::param::Query;
use tardis::web::web_resp::{TardisApiResult, TardisPage};
use crate::basic::dto::iam_account_dto::IamAccountBoneResp;

pub struct IamCcCertApi;

/// Common Console Cert API
#[poem_openapi::OpenApi(prefix_path = "/cc/cert", tag = "bios_basic::ApiTag::Common")]
impl IamCcCertApi {
    /// Find Accounts
    #[oai(path = "/:kind", method = "get")]
    async fn paginate(
        &self,
        kind: Path<String>,
        supplier: Query<supplier>,
        ctx: TardisContextExtractor,
    ) -> TardisApiResult<TardisPage<IamAccountBoneResp>> {

    }

    }
}