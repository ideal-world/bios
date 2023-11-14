use bios_basic::{
    rbum::{
        dto::rbum_filer_dto::{RbumBasicFilterReq, RbumCertFilterReq, RbumItemRelFilterReq},
        rbum_enumeration::RbumRelFromKind,
        serv::{
            rbum_cert_serv::RbumCertServ,
            rbum_crud_serv::RbumCrudOperation,
            rbum_domain_serv::RbumDomainServ,
            rbum_item_serv::{RbumItemCrudOperation, RbumItemServ},
            rbum_kind_serv::RbumKindServ,
        },
    },
    spi::{
        dto::spi_bs_dto::{SpiBsAddReq, SpiBsFilterReq},
        serv::spi_bs_serv::SpiBsServ,
        spi_constants::{self, SPI_CERT_KIND, SPI_IDENT_REL_TAG},
    },
};
use poem::{web::RealIp, Request};
use tardis::{
    basic::dto::TardisContext,
    serde_json,
    web::{
        context_extractor::TardisContextExtractor,
        poem_openapi::{self, payload::Json},
        web_resp::{TardisApiResult, TardisResp},
    },
    TardisFuns,
};

use crate::{conf_config::ConfConfig, conf_constants::DOMAIN_CODE, serv::*};
use crate::{dto::conf_auth_dto::*, serv::placehodler::has_placeholder_auth};

#[derive(Default, Clone, Copy, Debug)]

pub struct ConfCiAuthApi;

#[poem_openapi::OpenApi(prefix_path = "/ci/auth", tag = "bios_basic::ApiTag::Interface")]
impl ConfCiAuthApi {
    #[oai(path = "/register", method = "post")]
    async fn register(&self, json: Json<RegisterRequest>, ctx: TardisContextExtractor) -> TardisApiResult<RegisterResponse> {
        let reg_req = json.0;
        let funs = crate::get_tardis_inst();
        let resp = register(reg_req, &funs, &ctx.0).await?;
        TardisResp::ok(resp)
    }
    #[oai(path = "/register", method = "put")]
    async fn change_password(&self, json: Json<ChangePasswordRequest>, ctx: TardisContextExtractor) -> TardisApiResult<RegisterResponse> {
        let reg_req = json.0;
        let funs = crate::get_tardis_inst();
        let resp = change_password(reg_req, &funs, &ctx.0).await?;
        TardisResp::ok(resp)
    }
    #[oai(path = "/register_bundle", method = "put")]
    async fn bind_and_register(&self, json: Json<RegisterBundleRequest>, ctx: TardisContextExtractor) -> TardisApiResult<RegisterResponse> {
        let req = json.0;
        let mut funs = crate::get_tardis_inst();
        let ctx = ctx.0;
        let source = if let Some(source) = req.backend_service {
            serde_json::from_value(source).unwrap_or_default()
        } else {
            BackendServiceSource::Default
        };
        funs.begin().await?;
        let bs_id = match source {
            BackendServiceSource::Id(id) => id,
            BackendServiceSource::Default => {
                let default_ctx = TardisContext::default();
                let rbum_domain = RbumDomainServ::find_one_rbum(
                    &RbumBasicFilterReq {
                        code: Some(DOMAIN_CODE.to_string()),
                        ..Default::default()
                    },
                    &funs,
                    &default_ctx,
                )
                .await?
                .ok_or_else(|| funs.err().not_found(&SpiBsServ::get_obj_name(), "register", "not found spi-conf domain", "404-spi-bs-not-exist"))?;
                let bs = RbumItemServ::find_one_rbum(
                    &RbumBasicFilterReq {
                        enabled: Some(true),
                        rbum_domain_id: Some(rbum_domain.id),
                        ..Default::default()
                    },
                    &funs,
                    &default_ctx,
                )
                .await?
                .ok_or_else(|| funs.err().not_found(&SpiBsServ::get_obj_name(), "register", "not found backend service", "404-spi-bs-not-exist"))?;
                bs.id
            }
            BackendServiceSource::New { name, conn_uri, kind_code } => {
                let default_ctx = TardisContext::default();
                let kind_code = kind_code.unwrap_or(spi_constants::SPI_PG_KIND_CODE.to_string());
                let kind_id = RbumKindServ::get_rbum_kind_id_by_code(&kind_code, &funs)
                    .await?
                    .ok_or_else(|| funs.err().not_found(&SpiBsServ::get_obj_name(), "register", "db spi kind not found", "404-spi-bs-not-exist"))?;
                SpiBsServ::add_item(
                    &mut SpiBsAddReq {
                        name: name.into(),
                        conn_uri,
                        ext: "{\"max_connections\":20,\"min_connections\":10}".to_string(),
                        private: false,
                        disabled: None,
                        ak: Default::default(),
                        sk: Default::default(),
                        kind_id: kind_id.into(),
                    },
                    &funs,
                    &default_ctx,
                )
                .await?
            }
        };
        SpiBsServ::add_rel(&bs_id, req.app_tenent_id.as_deref().unwrap_or(ctx.owner.as_str()), &funs, &ctx).await?;
        let resp = register(req.register_request, &funs, &ctx).await?;
        funs.commit().await?;
        TardisResp::ok(resp)
    }
    #[oai(path = "/placeholder", method = "get")]
    async fn placeholder(&self, RealIp(real_ip): RealIp) -> TardisApiResult<bool> {
        let funs = crate::get_tardis_inst();
        if let Some(ip_addr) = real_ip {
            TardisResp::ok(has_placeholder_auth(ip_addr, &funs))
        } else {
            TardisResp::ok(false)
        }
    }
}

// id+owner
