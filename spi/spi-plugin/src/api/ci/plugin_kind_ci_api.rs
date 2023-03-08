use bios_basic::rbum::dto::rbum_filer_dto::RbumBasicFilterReq;
use bios_basic::rbum::dto::rbum_kind_dto::{RbumKindAddReq, RbumKindModifyReq, RbumKindSummaryResp};
use bios_basic::rbum::rbum_enumeration::RbumScopeLevelKind;
use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;
use bios_basic::rbum::serv::rbum_kind_serv::RbumKindServ;
use bios_basic::spi::spi_funs::SpiTardisFunInstExtractor;
use tardis::basic::field::TrimString;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem::Request;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::param::Query;
use tardis::web::web_resp::{TardisApiResult, TardisPage, TardisResp};

pub struct PluginKindApi;

/// Plugin kind API
#[poem_openapi::OpenApi(prefix_path = "/ci/spi/plugin/kind", tag = "bios_basic::ApiTag::Interface")]
impl PluginKindApi {

    /// find Plugin kind
    #[oai(path = "/", method = "get")]
    async fn find_page(
        &self,
        code: Query<Option<String>>,
        page_number: Query<u32>,
        page_size: Query<u32>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<TardisPage<RbumKindSummaryResp>> {
        let funs = request.tardis_fun_inst();
        let result = RbumKindServ::paginate_rbums(
            &mut RbumBasicFilterReq {
                code: code.0,
                
                ..Default::default()
            },
            page_number.0,
            page_size.0,
            None,
            None,
            &funs,
            &ctx.0,
        )
        .await?;
        TardisResp::ok(result)
    }
}
