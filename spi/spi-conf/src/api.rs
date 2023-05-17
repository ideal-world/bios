use bios_basic::spi::api::spi_ci_bs_api::SpiCiBsApi;
use tardis::web::web_server::TardisWebServer;

mod ci;
use ci::*;

pub async fn init_api(web_server: &TardisWebServer) {
    web_server.add_module("spi-conf", (SpiCiBsApi, ConfCiConfigServiceApi, ConfCiNamespaceApi)).await;
}
