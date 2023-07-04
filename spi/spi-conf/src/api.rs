use bios_basic::spi::api::spi_ci_bs_api::SpiCiBsApi;
use tardis::web::web_server::{TardisWebServer, WebServerModule};

mod ci;
use ci::*;
mod nacos;

use crate::conf_constants;
use nacos::*;

pub type ConfApi = (ConfCiApi, ConfNacosApi);

pub async fn init_api(web_server: &TardisWebServer) {
    web_server.add_module(conf_constants::DOMAIN_CODE, (SpiCiBsApi, ConfCiApi::default())).await;
    let mut nacos_module = WebServerModule::new(ConfNacosApi::default());
    nacos_module.options.set_uniform_error(false);
    web_server.add_module(&format!("{domain}-nacos", domain = conf_constants::DOMAIN_CODE), nacos_module).await;
}
