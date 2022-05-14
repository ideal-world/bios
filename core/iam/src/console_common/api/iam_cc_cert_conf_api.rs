use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi::{param::Path, param::Query, OpenApi};
use tardis::web::web_resp::{TardisApiResult, TardisPage, TardisResp};

use bios_basic::rbum::dto::rbum_cert_conf_dto::{RbumCertConfDetailResp, RbumCertConfSummaryResp};
use bios_basic::rbum::helper::rbum_scope_helper::get_max_level_id_by_context;

use crate::basic::serv::iam_cert_serv::IamCertServ;
use crate::iam_constants;

pub struct IamCcCertConfApi;

/// Common Console Cert Config API
#[OpenApi(prefix_path = "/cc/cert-conf", tag = "crate::iam_enumeration::Tag::Common")]
impl IamCcCertConfApi {
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
        id: Query<Option<String>>,
        code: Query<Option<String>>,
        name: Query<Option<String>>,
        page_number: Query<u64>,
        page_size: Query<u64>,
        desc_by_create: Query<Option<bool>>,
        desc_by_update: Query<Option<bool>>,
        cxt: TardisContextExtractor,
    ) -> TardisApiResult<TardisPage<RbumCertConfSummaryResp>> {
        let funs = iam_constants::get_tardis_inst();
        let result = IamCertServ::paginate_cert_conf(
            id.0,
            code.0,
            name.0,
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
}
