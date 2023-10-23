use std::sync::OnceLock;

use bios_basic::spi::api::spi_ci_bs_api::SpiCiBsApi;
use tardis::{
    basic::result::TardisResult,
    log,
    web::web_server::{TardisWebServer, WebServerGrpcModule, WebServerModule},
};

mod ci;
use ci::*;
mod nacos;

use crate::{conf_config::ConfConfig, conf_constants};
use nacos::*;
const NACOS_GRPC_SERVICE_DESCRIPTOR: &[u8] = include_bytes!("../proto/nacos_grpc_service.desc");
static GRPC_SERVER: OnceLock<TardisWebServer> = OnceLock::new();
static HTTP_SERVER: OnceLock<TardisWebServer> = OnceLock::new();
pub async fn init_api(web_server: &TardisWebServer) {
    web_server.add_module(conf_constants::DOMAIN_CODE, (SpiCiBsApi, ConfCiApi::default())).await;
    let mut nacos_module = WebServerModule::new(ConfNacosApi::default());
    nacos_module.options.set_uniform_error(false);
    web_server.add_module(&format!("{domain}-nacos", domain = conf_constants::DOMAIN_CODE), nacos_module).await;
}

pub async fn init_nacos_servers(cfg: &ConfConfig) -> TardisResult<()> {
    log::info!("[Spi.Conf] init nacos server");
    let http_server = TardisWebServer::init_simple(cfg.nacos_host.clone(), cfg.nacos_port)?;
    let mut nacos_module = WebServerModule::new(ConfNacosApi::default());
    nacos_module.options.set_uniform_error(false);

    let grpc_server = TardisWebServer::init_simple(cfg.nacos_host.clone(), cfg.nacos_grpc_port)?;

    grpc_server
        .add_grpc_route(
            WebServerGrpcModule::default()
                .with_grpc_service(BiRequestStreamGrpcServer::new(BiRequestStreamProtoImpl))
                .with_grpc_service(RequestGrpcServer::new(RequestProtoImpl))
                .with_descriptor(NACOS_GRPC_SERVICE_DESCRIPTOR.to_vec()),
        )
        .await;
    http_server.add_route(nacos_module).await;

    HTTP_SERVER.get_or_init(|| http_server).start().await?;
    GRPC_SERVER.get_or_init(|| grpc_server).start().await?;
    Ok(())
}
