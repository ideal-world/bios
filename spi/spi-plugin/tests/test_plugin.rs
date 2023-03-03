use std::env;
use std::time::Duration;

use bios_basic::rbum::rbum_config::RbumConfig;
use bios_basic::rbum::serv::rbum_kind_serv::RbumKindServ;
use bios_basic::spi::dto::spi_bs_dto::SpiBsAddReq;
use bios_basic::spi::spi_initializer;
use bios_basic::test::init_rbum_test_container;
use bios_basic::test::test_http_client::TestHttpClient;
use bios_spi_plugin::dto::plugin_api_dto::PluginApiAddOrModifyReq;
use bios_spi_plugin::plugin_constants::DOMAIN_CODE;
use bios_spi_plugin::plugin_enumeration::PluginApiMethodKind;
use bios_spi_plugin::plugin_initializer;
use tardis::basic::dto::TardisContext;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::tokio::time::sleep;
use tardis::web::web_resp::Void;
use tardis::{testcontainers, tokio, TardisFuns};
mod test_plugin_exec;

#[tokio::test]
async fn test_plugin() -> TardisResult<()> {
    env::set_var("RUST_LOG", "debug,tardis=trace,bios_mw_event=trace,test_event=trace,sqlx::query=off");
    let docker = testcontainers::clients::Cli::default();
    let _x = init_rbum_test_container::init(&docker, None).await?;
    init_data().await?;

    Ok(())
}

async fn init_data() -> TardisResult<()> {
    // Initialize RBUM
    bios_basic::rbum::rbum_initializer::init(DOMAIN_CODE, RbumConfig::default()).await?;

    let web_server = TardisFuns::web_server();
    // Initialize SPI plugin
    plugin_initializer::init(web_server).await.unwrap();

    tokio::spawn(async move {
        web_server.start().await.unwrap();
    });

    sleep(Duration::from_millis(500)).await;

    let funs = TardisFuns::inst_with_db_conn(DOMAIN_CODE.to_string(), None);
    let ctx = TardisContext {
        own_paths: "".to_string(),
        ak: "".to_string(),
        roles: vec![],
        groups: vec![],
        owner: "".to_string(),
        ..Default::default()
    };
    let _ = spi_initializer::add_kind(&"gitlib", &funs, &ctx).await?;
    let kind_id = RbumKindServ::get_rbum_kind_id_by_code("gitlib", &funs).await?.unwrap();
    let base_url = format!("https://localhost:8080/{}", DOMAIN_CODE);
    let mut client = TestHttpClient::new(base_url.clone());

    client.set_auth(&ctx)?;

    let bs_id: String = client
        .post(
            "/ci/manage/bs",
            &SpiBsAddReq {
                name: TrimString("test-spi".to_string()),
                kind_id: TrimString(kind_id.clone()),
                conn_uri: base_url.to_string(),
                ak: TrimString("minioadmin".to_string()),
                sk: TrimString("minioadmin".to_string()),
                ext: format!(r#"{{"region":"us-east-1"}}"#),
                private: false,
                disabled: None,
            },
        )
        .await;

    let _: Void = client.put(&format!("/ci/manage/bs/{}/rel/app001", bs_id), &Void {}).await;
    let _: String = client
        .post(
            &format!("/ci/spi/plugin/api"),
            &PluginApiAddOrModifyReq {
                code: TrimString("test-api".to_string()),
                name: TrimString("test-api".to_string()),
                kind_id: TrimString(kind_id),
                callback: "".to_string(),
                content_type: "".to_string(),
                timeout: 0,
                ext: "".to_string(),
                http_method: PluginApiMethodKind::GET,
                kind: "".to_string(),
                path_and_query: "ci/spi/plugin/test/exec".to_string(),
                save_message: true,
            },
        )
        .await;
    test_plugin_exec::test(&mut client).await?;

    Ok(())
}
