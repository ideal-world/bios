
use bios_basic::rbum::{rbum_config::RbumConfig, rbum_initializer};
use bios_spi_conf::{conf_initializer, conf_constants::DOMAIN_CODE};
use tardis::{basic::result::TardisResult, TardisFuns, tokio::{self, task::JoinHandle}};

#[allow(dead_code)]
pub async fn init_tardis() -> TardisResult<()> {
    TardisFuns::init(Some("tests/config")).await?;
    bios_basic::rbum::rbum_initializer::init(DOMAIN_CODE, RbumConfig::default()).await?;

    let web_server = TardisFuns::web_server();
    rbum_initializer::init("bios-spi", RbumConfig::default()).await?;
    conf_initializer::init(web_server).await?;
    Ok(())
}

#[allow(dead_code)]
pub fn start_web_server() -> JoinHandle<TardisResult<()>> {
    let task = TardisFuns::web_server().start();
    tokio::spawn(task)
}