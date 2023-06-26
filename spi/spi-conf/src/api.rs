use bios_basic::spi::api::spi_ci_bs_api::SpiCiBsApi;
use tardis::web::web_server::TardisWebServer;

mod ci;
use ci::*;
mod nacos;

use crate::conf_constants;
use nacos::*;

pub type ConfApi = (ConfCiApi, ConfNacosApi);

pub async fn init_api(web_server: &TardisWebServer) {
    web_server.add_module(conf_constants::DOMAIN_CODE, (SpiCiBsApi, ConfApi::default()), None).await;
}
