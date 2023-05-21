use std::time::Duration;

use crate::{
    api::{auth_crypto_api, auth_kernel_api, auth_mgr_api},
    auth_config::AuthConfig,
    auth_constants::DOMAIN_CODE,
    serv::{auth_crypto_serv, auth_res_serv},
};
use tardis::cache::{AsyncCommands, AsyncIter};
use tardis::{
    basic::result::TardisResult,
    log::{info, trace},
    tokio::time,
    web::web_server::TardisWebServer,
    TardisFuns,
};

pub async fn init(web_server: &TardisWebServer) -> TardisResult<()> {
    init_data().await?;
    auth_crypto_serv::init().await?;
    init_api(web_server).await
}

pub async fn crypto_init() -> TardisResult<()> {
    auth_crypto_serv::init().await
}

pub async fn init_data() -> TardisResult<()> {
    let cache_client = TardisFuns::cache_by_module_or_default(DOMAIN_CODE);
    let config = TardisFuns::cs_config::<AuthConfig>(DOMAIN_CODE);
    info!(
        "[Auth] Initializing full resource cache , interval [{}] secs fetch change resource cache.",
        config.cache_key_res_changed_timer_sec
    );
    auth_res_serv::init_res()?;
    let mut cache_cmd = cache_client.cmd().await?;
    let mut res_iter: AsyncIter<'_, (String, String)> = cache_cmd.hscan(&config.cache_key_res_info).await?;
    while let Some((f, v)) = res_iter.next_item().await {
        let f = f.split("##").collect::<Vec<_>>();
        let info = TardisFuns::json.str_to_json(&v)?;
        let auth = if let Some(auth) = info.get("auth") {
            if auth.is_null() {
                None
            } else {
                Some(TardisFuns::json.json_to_obj(auth.clone())?)
            }
        } else {
            None
        };
        let need_crypto_req = if let Some(need_crypto_req) = info.get("need_crypto_req") {
            need_crypto_req.as_bool().unwrap()
        } else {
            false
        };
        let need_crypto_resp = if let Some(need_crypto_resp) = info.get("need_crypto_resp") {
            need_crypto_resp.as_bool().unwrap()
        } else {
            false
        };
        let need_double_auth = if let Some(need_double_auth) = info.get("need_double_auth") {
            need_double_auth.as_bool().unwrap()
        } else {
            false
        };
        auth_res_serv::add_res(f[1], f[0], auth, need_crypto_req, need_crypto_resp, need_double_auth).unwrap();
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
                    if let Some(changed_value) = TardisFuns::cache_by_module_or_default(DOMAIN_CODE).hget(&config.cache_key_res_info, key).await.unwrap() {
                        let info = TardisFuns::json.str_to_json(&changed_value).unwrap();
                        let auth = if let Some(auth) = info.get("auth") {
                            if auth.is_null() {
                                None
                            } else {
                                Some(TardisFuns::json.json_to_obj(auth.clone()).unwrap())
                            }
                        } else {
                            None
                        };
                        let need_crypto_req = if let Some(need_crypto_req) = info.get("need_crypto_req") {
                            need_crypto_req.as_bool().unwrap()
                        } else {
                            false
                        };
                        let need_crypto_resp = if let Some(need_crypto_resp) = info.get("need_crypto_resp") {
                            need_crypto_resp.as_bool().unwrap()
                        } else {
                            false
                        };
                        let need_double_auth = if let Some(need_double_auth) = info.get("need_double_auth") {
                            need_double_auth.as_bool().unwrap()
                        } else {
                            false
                        };
                        auth_res_serv::add_res(f[1], f[0], auth, need_crypto_req, need_crypto_resp, need_double_auth).unwrap();
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
    web_server.add_module(DOMAIN_CODE, (auth_mgr_api::MgrApi, auth_crypto_api::CryptoApi, auth_kernel_api::AuthApi), None).await;
    Ok(())
}
