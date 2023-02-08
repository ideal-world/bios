use std::env;
use std::time::Duration;

use bios_basic::rbum::rbum_config::RbumConfig;
use bios_basic::rbum::serv::rbum_kind_serv::RbumKindServ;
use bios_basic::spi::dto::spi_bs_dto::SpiBsAddReq;
use bios_basic::spi::spi_constants;
use bios_basic::test::test_http_client::TestHttpClient;
use bios_spi_search::search_constants::DOMAIN_CODE;
use bios_spi_search::search_initializer;
use tardis::basic::dto::TardisContext;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::tokio::time::sleep;
use tardis::web::web_resp::Void;
use tardis::{testcontainers, tokio, TardisFuns};

#[tokio::test]
async fn test_event() -> TardisResult<()> {
     let docker = testcontainers::clients::Cli::default();
    let _x = init_rbum_test_container::init(&docker, None).await?;

    init_data().await?;

    Ok(())
}

async fn init_data() -> TardisResult<()> {
    

    Ok(())
}
