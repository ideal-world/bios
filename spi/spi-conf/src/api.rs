use std::sync::OnceLock;

use bios_basic::spi::api::spi_ci_bs_api::SpiCiBsApi;
use tardis::{
    basic::result::TardisResult,
    web::web_server::{TardisWebServer, WebServerGrpcModule, WebServerModule},
};

mod ci;
use ci::*;
mod nacos;

use crate::{conf_config::ConfConfig, conf_constants};
use nacos::*;
const NACOS_GRPC_SERVICE_DESCRIPTOR: &[u8] = include_bytes!("../proto/nacos_grpc_service.desc");
static GRPC_SERVER: OnceLock<TardisWebServer> = OnceLock::new();
pub async fn init_api(web_server: &TardisWebServer) {
    web_server.add_module(conf_constants::DOMAIN_CODE, (SpiCiBsApi, ConfCiApi::default())).await;
    let mut nacos_module = WebServerModule::new(ConfNacosApi::default());
    nacos_module.options.set_uniform_error(false);
    web_server.add_module(&format!("{domain}-nacos", domain = conf_constants::DOMAIN_CODE), nacos_module).await;
    web_server
        .add_grpc_route(WebServerGrpcModule::default().with_grpc_service(RequestGrpcServer::new(RequestProtoImpl)).with_descriptor(NACOS_GRPC_SERVICE_DESCRIPTOR.to_vec()))
        .await;
}

pub async fn init_grpc_server(cfg: &ConfConfig) -> TardisResult<()> {
    let grpc_server = TardisWebServer::init_simple(&cfg.grpc_host.to_string(), cfg.grpc_port)?;
    grpc_server
        .add_grpc_route(WebServerGrpcModule::default().with_grpc_service(RequestGrpcServer::new(RequestProtoImpl)).with_descriptor(NACOS_GRPC_SERVICE_DESCRIPTOR.to_vec()))
        .await;
    GRPC_SERVER.get_or_init(|| grpc_server).start().await?;
    Ok(())
}
