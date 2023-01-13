use std::time::Duration;

use crate::{
    api::{auth_apisix_api, auth_mgr_api},
    auth_config::AuthConfig,
    auth_constants::DOMAIN_CODE,
    serv::auth_res_serv,
};
use redis::{AsyncCommands, AsyncIter};
use tardis::{
    basic::result::TardisResult,
    log::{info, trace},
    tokio::time,
    web::web_server::TardisWebServer,
    TardisFuns,
};

pub async fn init(web_server: &TardisWebServer) -> TardisResult<()> {
    init_data().await?;
    init_api(web_server).await
}

pub async fn init_data() -> TardisResult<()> {
    let cache_client = TardisFuns::cache_by_module_or_default(DOMAIN_CODE);
    let config = TardisFuns::cs_config::<AuthConfig>(DOMAIN_CODE);
    info!(
        "[Auth] Initializing full resource cache , interval [{}] secs fetch change resource cache.",
        config.cache_key_res_changed_timer_sec
    );
    let mut cache_cmd = cache_client.cmd().await?;
    let mut res_iter: AsyncIter<'_, (String, String)> = cache_cmd.hscan(&config.cache_key_res_info).await?;
    while let Some((f, v)) = res_iter.next_item().await {
        let f = f.split("##").collect::<Vec<_>>();
        auth_res_serv::add_res(f[1], f[0], &TardisFuns::json.str_to_obj(&v)?).unwrap();
    }
    tardis::tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(config.cache_key_res_changed_timer_sec as u64));
        loop {
            {
                trace!("[Auth] Fetch changed resource cache");
                let mut cache_cmd = cache_client.cmd().await.unwrap();
                let mut res_iter: AsyncIter<String> = cache_cmd.scan_match(&format!("{}*", config.cache_key_res_changed_info)).await.unwrap();
                while let Some(changed_key) = res_iter.next_item().await {
                    let key = changed_key.strip_prefix(&config.cache_key_res_changed_info).unwrap();
                    trace!("[Auth]Fetch changed key [{}]", key);
                    let f = key.split("##").collect::<Vec<_>>();
                    if let Some(changed_value) = TardisFuns::cache_by_module_or_default(DOMAIN_CODE).hget(&config.cache_key_res_info, &key).await.unwrap() {
                        auth_res_serv::add_res(f[1], f[0], &TardisFuns::json.str_to_obj(&changed_value).unwrap()).unwrap();
                    } else {
                        auth_res_serv::remove_res(f[1], f[0]).unwrap();
                    }
                }
            }
            interval.tick().await;
        }
    });
    Ok(())
}

pub async fn init_api(web_server: &TardisWebServer) -> TardisResult<()> {
    web_server.add_module(DOMAIN_CODE, (auth_mgr_api::MgrApi, auth_apisix_api::AuthApi, auth_apisix_api::FakeOPAApi)).await;
    Ok(())
}
