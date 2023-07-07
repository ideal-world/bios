use bios_basic::rbum::dto::rbum_filer_dto::{RbumBasicFilterReq, RbumItemRelFilterReq};
use bios_basic::rbum::rbum_enumeration::RbumRelFromKind;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;
use bios_basic::spi::dto::spi_bs_dto::{SpiBsAddReq, SpiBsDetailResp, SpiBsFilterReq, SpiBsModifyReq, SpiBsSummaryResp};
use bios_basic::spi::serv::spi_bs_serv::SpiBsServ;
use bios_basic::spi::spi_constants;

use tardis::web::context_extractor::TardisContextExtractor;

use tardis::web::poem_openapi;
use tardis::web::poem_openapi::param::{Path, Query};
use tardis::web::poem_openapi::payload::Json;
use tardis::web::web_resp::{TardisApiResult, TardisPage, TardisResp, Void};

use crate::dto::plugin_bs_dto::{PluginBsAddReq, PluginBsInfoResp};
use crate::serv::plugin_bs_serv::PluginBsServ;
#[derive(Clone)]

pub struct PluginCiBsApi;

/// Interface Console Backend rel Service API
#[poem_openapi::OpenApi(prefix_path = "/ci/manage/bs", tag = "bios_basic::ApiTag::Interface")]
impl PluginCiBsApi {
    /// Add Backend Service
    #[oai(path = "/", method = "post")]
    async fn add(&self, mut add_req: Json<SpiBsAddReq>, ctx: TardisContextExtractor) -> TardisApiResult<String> {
        let mut funs = crate::get_tardis_inst();
        funs.begin().await?;
        let result = SpiBsServ::add_item(&mut add_req.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        TardisResp::ok(result)
    }

    /// Modify Backend Service
    #[oai(path = "/:id", method = "patch")]
    async fn modify(&self, id: Path<String>, mut modify_req: Json<SpiBsModifyReq>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = crate::get_tardis_inst();
        funs.begin().await?;
        SpiBsServ::modify_item(&id.0, &mut modify_req.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Get Backend Service
    #[oai(path = "/:id", method = "get")]
    async fn get(&self, id: Path<String>, ctx: TardisContextExtractor) -> TardisApiResult<SpiBsDetailResp> {
        let funs = crate::get_tardis_inst();
        let result = SpiBsServ::get_bs(&id.0, &funs, &ctx.0).await?;
        TardisResp::ok(result)
    }

    /// Find Backend Services
    #[oai(path = "/", method = "get")]
    async fn paginate(
        &self,
        id: Query<Option<String>>,
        name: Query<Option<String>>,
        kind_id: Query<Option<String>>,
        kind_code: Query<Option<String>>,
        app_tenant_id: Query<Option<String>>,
        page_number: Query<u32>,
        page_size: Query<u32>,
        desc_by_create: Query<Option<bool>>,
        desc_by_update: Query<Option<bool>>,
        ctx: TardisContextExtractor,
    ) -> TardisApiResult<TardisPage<SpiBsSummaryResp>> {
        let funs = crate::get_tardis_inst();
        let rel = app_tenant_id.0.map(|app_tenant_id| RbumItemRelFilterReq {
            rel_by_from: true,
            tag: Some(spi_constants::SPI_IDENT_REL_TAG.to_string()),
            from_rbum_kind: Some(RbumRelFromKind::Item),
            rel_item_id: Some(app_tenant_id),
            ..Default::default()
        });
        let result = SpiBsServ::paginate_items(
            &SpiBsFilterReq {
                basic: RbumBasicFilterReq {
                    ids: id.0.map(|id| vec![id]),
                    name: name.0,
                    ..Default::default()
                },
                kind_id: kind_id.0,
                kind_code: kind_code.0,
                domain_code: Some(funs.module_code().to_string()),
                rel,
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
    async fn delete(&self, id: Path<String>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = crate::get_tardis_inst();
        funs.begin().await?;
        SpiBsServ::delete_item_with_all_rels(&id.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Add Plugin Service Rel App/Tenant
    #[oai(path = "/:id/rel/:app_tenant_id", method = "put")]
    async fn add_plugin_rel_agg(&self, id: Path<String>, app_tenant_id: Path<String>, mut add_req: Json<PluginBsAddReq>, ctx: TardisContextExtractor) -> TardisApiResult<String> {
        let mut funs = crate::get_tardis_inst();
        funs.begin().await?;
        let result = PluginBsServ::add_or_modify_plugin_rel_agg(&id.0, &app_tenant_id.0, &mut add_req.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        TardisResp::ok(result)
    }

    /// Get Plugin Service Rel App/Tenant Support empty
    #[oai(path = "/:id/rel/:app_tenant_id/empty", method = "get")]
    async fn get_empty_bs_rel_agg(&self, id: Path<String>, app_tenant_id: Path<String>, ctx: TardisContextExtractor) -> TardisApiResult<PluginBsInfoResp> {
        let funs = crate::get_tardis_inst();
        let result = PluginBsServ::get_bs(&id.0, &app_tenant_id.0, &funs, &ctx.0).await?;
        TardisResp::ok(result)
    }

    /// Exist Plugin Service Rel App/Tenant Support empty
    #[oai(path = "/:id/rel/exist/:app_tenant_id/empty", method = "get")]
    async fn exist_empty_bs_rel_agg(&self, id: Path<String>, app_tenant_id: Path<String>, ctx: TardisContextExtractor) -> TardisApiResult<bool> {
        let funs = crate::get_tardis_inst();
        let result = PluginBsServ::exist_bs(&id.0, &app_tenant_id.0, &funs, &ctx.0).await?;
        TardisResp::ok(result)
    }

    /// Get Plugin Service Rel App/Tenant
    #[oai(path = "/:id/rel/:app_tenant_id", method = "get")]
    async fn get_bs_rel_agg(&self, id: Path<String>, app_tenant_id: Path<String>, ctx: TardisContextExtractor) -> TardisApiResult<PluginBsInfoResp> {
        let funs = crate::get_tardis_inst();
        let result = PluginBsServ::get_bs(&id.0, &app_tenant_id.0, &funs, &ctx.0).await?;
        TardisResp::ok(result)
    }

    /// Find Plugin Services rel App/Tenant
    #[oai(path = "/rel", method = "get")]
    async fn paginate_bs_rel_agg(
        &self,
        app_tenant_id: Query<String>,
        page_number: Query<u32>,
        page_size: Query<u32>,
        desc_by_create: Query<Option<bool>>,
        desc_by_update: Query<Option<bool>>,
        ctx: TardisContextExtractor,
    ) -> TardisApiResult<TardisPage<PluginBsInfoResp>> {
        let funs = crate::get_tardis_inst();
        let result = PluginBsServ::paginate_bs_rel_agg(&app_tenant_id.0, page_number.0, page_size.0, desc_by_create.0, desc_by_update.0, &funs, &ctx.0).await?;
        TardisResp::ok(result)
    }

    /// Delete Plugin Service Rel App/Tenant
    #[oai(path = "/:id/rel/:app_tenant_id", method = "delete")]
    async fn delete_rel(&self, id: Path<String>, app_tenant_id: Path<String>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = crate::get_tardis_inst();
        funs.begin().await?;
        PluginBsServ::delete_plugin_rel(&id.0, &app_tenant_id.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }
}
