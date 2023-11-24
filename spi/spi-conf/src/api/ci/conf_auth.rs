use bios_basic::{
    rbum::{
        dto::rbum_filer_dto::RbumBasicFilterReq,
        serv::{
            rbum_crud_serv::RbumCrudOperation,
            rbum_domain_serv::RbumDomainServ,
            rbum_item_serv::{RbumItemCrudOperation, RbumItemServ},
            rbum_kind_serv::RbumKindServ,
        },
    },
    spi::{
        dto::spi_bs_dto::SpiBsAddReq,
        serv::spi_bs_serv::SpiBsServ,
        spi_constants::{self},
    },
};
use poem::web::RealIp;
use tardis::{
    serde_json,
    web::{
        context_extractor::TardisContextExtractor,
        poem_openapi::{self, payload::Json},
        reqwest::Url,
        web_resp::{TardisApiResult, TardisResp},
    },
};

use crate::{conf_constants::DOMAIN_CODE, serv::*};
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
    async fn register_bundle(&self, json: Json<RegisterBundleRequest>, ctx: TardisContextExtractor) -> TardisApiResult<RegisterResponse> {
        let req = json.0;
        let mut funs = crate::get_tardis_inst();
        let mut ctx = ctx.0;
        let source = if let Some(source) = req.backend_service {
            serde_json::from_value(source).unwrap_or_default()
        } else {
            BackendServiceSource::Default
        };
        funs.begin().await?;
        let default_ctx = ctx.clone();
        let bs_id = match source {
            BackendServiceSource::Id(id) => id,
            BackendServiceSource::Default => {
                // let default_ctx = TardisContext::default();
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
            BackendServiceSource::New { name, kind_code } => {
                // #TODO
                // this should be determined by url, but now we only support spi-pg
                let kind_code = kind_code.unwrap_or(spi_constants::SPI_PG_KIND_CODE.to_string());
                let kind_id = RbumKindServ::get_rbum_kind_id_by_code(&kind_code, &funs)
                    .await?
                    .ok_or_else(|| funs.err().not_found(&SpiBsServ::get_obj_name(), "register", "db spi kind not found", "404-spi-bs-not-exist"))?;
                let conn_uri = tardis::TardisFuns::fw_config().db().default.url.clone();
                let mut req = SpiBsAddReq {
                    name: name.unwrap_or(format!("spi-conf-{}", tardis::crypto::crypto_key::TardisCryptoKey.rand_8_hex())).into(),
                    conn_uri: conn_uri.to_string(),
                    ext: "{\"max_connections\":20,\"min_connections\":10}".to_string(),
                    private: false,
                    disabled: None,
                    ak: "".into(),
                    sk: "".into(),
                    kind_id: kind_id.into(),
                };
                if let Ok(conn_uri) = Url::parse(&conn_uri) {
                    req.ak = conn_uri.username().into();
                    req.sk = conn_uri.password().unwrap_or("").into();
                }
                SpiBsServ::add_item(&mut req, &funs, &default_ctx).await?
            }
        };
        let app_tenant_id = req.app_tenant_id.as_deref().unwrap_or(ctx.owner.as_str());
        SpiBsServ::add_rel(&bs_id, app_tenant_id, &funs, &ctx).await?;
        ctx.owner = app_tenant_id.to_string();
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
