use std::{
    env,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

use bios_basic::{
    rbum::serv::rbum_kind_serv::RbumKindServ,
    spi::{dto::spi_bs_dto::SpiBsAddReq, spi_constants, spi_initializer},
    test::test_http_client::TestHttpClient,
};
use bios_mw_schedule::{schedule_constants::DOMAIN_CODE, schedule_initializer, serv::schedule_job_serv::OwnedScheduleTaskServ};
use bios_spi_kv::kv_initializer;
use bios_spi_log::log_initializer;
use tardis::{
    basic::{dto::TardisContext, field::TrimString, result::TardisResult},
    web::{
        poem_openapi,
        web_resp::{TardisApiResult, TardisResp, Void},
    },
    TardisFuns,
};
#[allow(dead_code)]
pub struct TestEnv {
    pub counter: Arc<AtomicUsize>,
}

#[derive(Default, Clone)]
pub struct CallbackApi {
    counter: Arc<AtomicUsize>,
}

#[poem_openapi::OpenApi(prefix_path = "/callback")]
impl CallbackApi {
    #[oai(path = "/inc", method = "get")]
    pub async fn inc(&self) -> TardisApiResult<Void> {
        let counter = self.counter.fetch_add(1, Ordering::SeqCst) + 1;
        tardis::log::info!("callback: inc to {counter}");
        TardisResp::ok(Void {})
    }
}
#[allow(dead_code)]
pub async fn init_tardis() -> TardisResult<()> {
    TardisFuns::init(Some("tests/config")).await?;
    // rbum_initializer::init("", RbumConfig::default()).await?;
    let web_server = TardisFuns::web_server();
    // cache_initializer::init(web_server).await?;
    bios_basic::rbum::rbum_initializer::init("bios-spi", bios_basic::rbum::rbum_config::RbumConfig::default()).await?;
    log_initializer::init(&web_server).await?;
    kv_initializer::init(&web_server).await?;
    schedule_initializer::init(&web_server).await?;
    Ok(())
}
#[allow(dead_code)]
pub async fn mock_webserver() -> TardisResult<Arc<AtomicUsize>> {
    println!("mock logger started");
    let cb_api = CallbackApi::default();
    let cb_counter = Arc::clone(&cb_api.counter);
    TardisFuns::web_server().add_route(cb_api).await.start().await?;
    Ok(cb_counter)
}
#[allow(dead_code)]
pub async fn init_task_serve_group(size: usize) -> TardisResult<Vec<Arc<OwnedScheduleTaskServ>>> {
    let mut funs = TardisFuns::inst_with_db_conn(DOMAIN_CODE.to_string(), None);
    let mut collector = vec![];
    funs.begin().await?;
    let ctx = spi_initializer::init(DOMAIN_CODE, &funs).await?;
    for _ in 0..size {
        collector.push(OwnedScheduleTaskServ::init(&funs, &ctx).await?);
    }
    funs.commit().await?;
    Ok(collector)
}
#[allow(dead_code)]
pub async fn init_client() -> TardisResult<TestHttpClient> {
    let mut client = TestHttpClient::new(format!("https://localhost:8080/{}", DOMAIN_CODE));
    client.set_auth(&TardisContext {
        own_paths: "t1/app001".to_string(),
        ak: "".to_string(),
        roles: vec![],
        groups: vec![],
        owner: "app001".to_string(),
        ..Default::default()
    })?;
    Ok(client)
}

#[allow(dead_code)]
pub async fn init_spi(code: &str) -> TardisResult<()> {
    let mut client = TestHttpClient::new(format!("https://localhost:8080/{}", code));
    let funs = TardisFuns::inst_with_db_conn(DOMAIN_CODE.to_string(), None);

    let kind_id = RbumKindServ::get_rbum_kind_id_by_code(spi_constants::SPI_PG_KIND_CODE, &funs).await?.unwrap();
    let ctx = TardisContext {
        own_paths: "".to_string(),
        ak: "".to_string(),
        roles: vec![],
        groups: vec![],
        owner: "".to_string(),
        ..Default::default()
    };
    client.set_auth(&ctx)?;

    let bs_id: String = client
        .post(
            "/ci/manage/bs",
            &SpiBsAddReq {
                name: TrimString("test-spi".to_string()),
                kind_id: TrimString(kind_id),
                conn_uri: env::var("TARDIS_FW.DB.URL").unwrap(),
                ak: TrimString("".to_string()),
                sk: TrimString("".to_string()),
                ext: "{\"max_connections\":20,\"min_connections\":10}".to_string(),
                private: false,
                disabled: None,
            },
        )
        .await;

    let _: Void = client.put(&format!("/ci/manage/bs/{}/rel/app001", bs_id), &Void {}).await;

    Ok(())
}
