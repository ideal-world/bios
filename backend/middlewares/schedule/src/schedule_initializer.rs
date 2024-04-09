use crate::{api::ci::schedule_ci_job_api, schedule_config::ScheduleConfig, schedule_constants::DOMAIN_CODE, serv::schedule_job_serv};
use bios_basic::spi::{dto::spi_bs_dto::SpiBsCertResp, spi_constants, spi_funs::SpiBsInst, spi_initializer};
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    log::error,
    web::{web_server::TardisWebServer, ws_client::TardisWSClient},
    TardisFuns,
};

pub async fn init(web_server: &TardisWebServer) -> TardisResult<()> {
    let mut funs = TardisFuns::inst_with_db_conn(DOMAIN_CODE.to_string(), None);
    funs.begin().await?;
    let ctx = spi_initializer::init(DOMAIN_CODE, &funs).await?;
    schedule_job_serv::init(&funs, &ctx).await?;
    funs.commit().await?;
    init_api(web_server).await
}

async fn init_api(web_server: &TardisWebServer) -> TardisResult<()> {
    web_server.add_module(DOMAIN_CODE, schedule_ci_job_api::ScheduleCiJobApi).await;
    Ok(())
}

pub async fn init_fun(bs_cert: SpiBsCertResp, ctx: &TardisContext, mgr: bool) -> TardisResult<SpiBsInst> {
    match bs_cert.kind_code.as_str() {
        #[cfg(feature = "spi-pg")]
        spi_constants::SPI_PG_KIND_CODE => spi_initializer::common_pg::init(&bs_cert, ctx, mgr).await,
        _ => Err(bs_cert.bs_not_implemented())?,
    }
}

async fn init_ws_client() -> TardisWSClient {
    while !TardisFuns::web_server().is_running().await {
        tardis::tokio::task::yield_now().await
    }
    let funs = TardisFuns::inst_with_db_conn(DOMAIN_CODE.to_string(), None);
    let conf = funs.conf::<ScheduleConfig>();
    let mut event_conf = conf.event.clone();
    if event_conf.avatars.is_empty() {
        event_conf.avatars.push(format!("{}/{}", event_conf.topic_code, env!("CARGO_PKG_NAME")))
    }
    let default_avatar = event_conf.avatars[0].clone();
    set_default_avatar(default_avatar);
    let client = bios_sdk_invoke::clients::event_client::EventClient::new("http://127.0.0.1:8080/event", &funs);
    loop {
        let addr = loop {
            if let Ok(result) = client.register(&event_conf.clone().into()).await {
                break result.ws_addr;
            }
            tardis::tokio::time::sleep(std::time::Duration::from_secs(10)).await;
        };
        let ws_client = TardisFuns::ws_client(&addr, |_| async move { None }).await;
        match ws_client {
            Ok(ws_client) => {
                return ws_client;
            }
            Err(err) => {
                error!("[BIOS.Event] failed to connect to event server: {}", err);
                tardis::tokio::time::sleep(std::time::Duration::from_secs(10)).await;
            }
        }
    }
}
use std::sync::OnceLock;
tardis::tardis_static! {
    pub(crate) async ws_client: TardisWSClient = init_ws_client();
    pub(crate) async set default_avatar: String;
}
