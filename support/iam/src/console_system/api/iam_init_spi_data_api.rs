use crate::basic::dto::iam_filer_dto::IamAppFilterReq;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;
use tardis::basic::result::TardisResult;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::payload::Json;
use tardis::web::web_resp::{TardisApiResult, TardisResp, Void};

use crate::basic::dto::iam_tenant_dto::IamTenantSummaryResp;
use crate::basic::serv::iam_app_serv::IamAppServ;
use crate::iam_constants;

pub struct IamInitSpiDataApi;

/// System Console Tenant API
#[poem_openapi::OpenApi(prefix_path = "/cs/init/data", tag = "bios_basic::ApiTag::System")]
impl IamInitSpiDataApi {
    /// Do Init Data
    #[oai(path = "/", method = "post")]
    async fn init_spi_data(&self, _ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        Self::do_init_spi_data(&funs).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }
    pub async fn do_init_spi_data() -> TardisResult<()> {
        // #[cfg(feature = "spi_kv_features")]
        {
            IamAppServ::paginate_items(&IamAppFilterReq {}, 1, 100, None, None, funs, ctx).await?;
        }
        Ok(())
    }
}
