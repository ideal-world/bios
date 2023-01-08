use std::time::Duration;

use crate::{
    api::{auth_apisix_api, auth_mgr_api},
    auth_config::AuthConfig,
    auth_constants::DOMAIN_CODE,
    serv::auth_res_serv,
};
use redis::{AsyncCommands, AsyncIter};
use tardis::{basic::result::TardisResult, tokio::time, web::web_server::TardisWebServer, TardisFuns};

pub async fn init(web_server: &TardisWebServer) -> TardisResult<()> {
    init_data().await?;
    init_api(web_server).await
}

async fn init_data() -> TardisResult<()> {
    let cache_client = TardisFuns::cache_by_module_or_default(DOMAIN_CODE);
    let config = TardisFuns::cs_config::<AuthConfig>(DOMAIN_CODE);
    let mut cache_cmd = cache_client.cmd().await;
    let mut res_iter: AsyncIter<'_, (String, String)> = cache_cmd.hscan(&config.cache_key_res_info).await?;
    while let Some((f, v)) = res_iter.next_item().await {
        let f = f.split("##").collect::<Vec<_>>();
        auth_res_serv::add_res(f[0], f[1], &TardisFuns::json.str_to_obj(&v)?).unwrap();
    }
    tardis::tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(config.cache_key_res_changed_timer_sec as u64));
        loop {
            {
                let mut cache_cmd = cache_client.cmd().await;
                let mut res_iter: AsyncIter<String> = cache_cmd.scan_match(&config.cache_key_res_changed_info).await.unwrap();
                while let Some(changed_key) = res_iter.next_item().await {
                    let f = changed_key.split("##").collect::<Vec<_>>();
                    if let Some(changed_value) = cache_client.hget(&config.cache_key_res_changed_info, &changed_key).await.unwrap() {
                        auth_res_serv::add_res(f[0], f[1], &TardisFuns::json.str_to_obj(&changed_value).unwrap()).unwrap();
                    } else {
                        auth_res_serv::remove_res(f[0], f[1]).unwrap();
                    }
                }
            }
            interval.tick().await;
        }
    });
    Ok(())
}

async fn init_api(web_server: &TardisWebServer) -> TardisResult<()> {
    web_server.add_module(DOMAIN_CODE, (auth_mgr_api::MgrApi, auth_apisix_api::AuthApi)).await;
    Ok(())
}
