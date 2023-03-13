use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem::Request;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::param::{Path, Query};
use tardis::web::poem_openapi::payload::Json;
use tardis::web::web_resp::{TardisApiResult, TardisPage, TardisResp, Void};

use crate::rbum::dto::rbum_filer_dto::RbumBasicFilterReq;
use crate::rbum::serv::rbum_item_serv::RbumItemCrudOperation;
use crate::spi::dto::spi_bs_dto::{SpiBsAddReq, SpiBsDetailResp, SpiBsFilterReq, SpiBsModifyReq, SpiBsSummaryResp};
use crate::spi::serv::spi_bs_serv::SpiBsServ;
use crate::spi::spi_funs::SpiTardisFunInstExtractor;

pub struct SpiCiBsApi;

/// Interface Console Backend Service API
#[poem_openapi::OpenApi(prefix_path = "/ci/manage/bs", tag = "crate::ApiTag::Interface")]
impl SpiCiBsApi {
    /// Add Backend Service
    #[oai(path = "/", method = "post")]
    async fn add(&self, mut add_req: Json<SpiBsAddReq>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<String> {
        let mut funs = request.tardis_fun_inst();
        funs.begin().await?;
        let result = SpiBsServ::add_item(&mut add_req.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        TardisResp::ok(result)
    }

    /// Modify Backend Service
    #[oai(path = "/:id", method = "patch")]
    async fn modify(&self, id: Path<String>, mut modify_req: Json<SpiBsModifyReq>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        let mut funs = request.tardis_fun_inst();
        funs.begin().await?;
        SpiBsServ::modify_item(&id.0, &mut modify_req.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Get Backend Service
    #[oai(path = "/:id", method = "get")]
    async fn get(&self, id: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<SpiBsDetailResp> {
        let funs = request.tardis_fun_inst();
        let result = SpiBsServ::get_bs(&id.0, &funs, &ctx.0).await?;
        TardisResp::ok(result)
    }

    /// Find Backend Services
    #[oai(path = "/", method = "get")]
    async fn paginate(
        &self,
        id: Query<Option<String>>,
        name: Query<Option<String>>,
        page_number: Query<u32>,
        page_size: Query<u32>,
        desc_by_create: Query<Option<bool>>,
        desc_by_update: Query<Option<bool>>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<TardisPage<SpiBsSummaryResp>> {
        let funs = request.tardis_fun_inst();
        let result = SpiBsServ::paginate_items(
            &SpiBsFilterReq {
                basic: RbumBasicFilterReq {
                    ids: id.0.map(|id| vec![id]),
                    name: name.0,
                    ..Default::default()
                },
                domain_code: Some(funs.module_code().to_string()),
                ..Default::default()
            },
            page_number.0,
            page_size.0,
            desc_by_create.0,
            desc_by_update.0,
            &funs,
            &ctx.0,
        )
        .await?;
        TardisResp::ok(result)
    }

    /// Delete Backend Service
    #[oai(path = "/:id", method = "delete")]
    async fn delete(&self, id: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        let mut funs = request.tardis_fun_inst();
        funs.begin().await?;
        SpiBsServ::delete_item_with_all_rels(&id.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Add Backend Service Rel App/Tenant
    #[oai(path = "/:id/rel/:app_tenant_id", method = "put")]
    async fn add_rel(&self, id: Path<String>, app_tenant_id: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        let mut funs = request.tardis_fun_inst();
        funs.begin().await?;
        SpiBsServ::add_rel(&id.0, &app_tenant_id.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Delete Backend Service Rel App/Tenant
    #[oai(path = "/:id/rel/:app_tenant_id", method = "delete")]
    async fn delete_rel(&self, id: Path<String>, app_tenant_id: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        let mut funs = request.tardis_fun_inst();
        funs.begin().await?;
        SpiBsServ::delete_rel(&id.0, &app_tenant_id.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }
}
