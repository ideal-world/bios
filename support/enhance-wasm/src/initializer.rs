use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{
    constants::{self, SessionConfig, StableConfig},
    mini_tardis::{basic::TardisResult, crypto, error, http, log},
    modules::{
        crypto_process,
        resource_process::{self, ResContainerNode},
    },
};

pub(crate) async fn init(service_url: &str, serv_config: Option<ServConfig>) -> TardisResult<bool> {
    let service_url = if service_url.ends_with('/') {
        service_url.to_string()
    } else {
        format!("{service_url}/")
    };
    let serv_config = if let Some(serv_config) = serv_config {
        log::log("[BIOS] Init by spec config.");
        serv_config
    } else {
        log::log(&format!("[BIOS] Init by url: {service_url}."));
        http::request::<ServConfig>("GET", &format!("{service_url}auth/apis"), None, HashMap::new()).await?.unwrap()
    };
    do_init(&service_url, &serv_config)?;
    Ok(serv_config.strict_security_mode)
}

pub(crate) fn do_init(service_url: &str, serv_config: &ServConfig) -> TardisResult<()> {
    init_config(service_url, serv_config)?;
    init_behavior(serv_config.strict_security_mode, service_url)?;
    Ok(())
}

fn init_behavior(strict_security_mode: bool, _service_url: &str) -> TardisResult<()> {
    error::set_hide_error_detail(strict_security_mode);
    #[cfg(target_arch = "wasm32")]
    {
        js_sys::Reflect::set(
            &wasm_bindgen::JsValue::from(web_sys::window().unwrap()),
            &wasm_bindgen::JsValue::from(constants::BIOS_SERV_URL_CONFIG),
            &wasm_bindgen::JsValue::from(_service_url),
        )?;
        if constants::get_strict_security_mode()? {
            crate::mini_tardis::channel::init(
                constants::BIOS_SESSION_CONFIG,
                |_| {
                    let config_container = crate::constants::SESSION_CONFIG.read().unwrap();
                    if let Some(config) = config_container.as_ref() {
                        crate::mini_tardis::channel::send(crate::constants::BIOS_SESSION_CONFIG, config).unwrap();
                    }
                },
                |session_config| {
                    let session_config = crate::mini_tardis::serde::jsvalue_to_obj::<crate::constants::SessionConfig>(session_config).unwrap();
                    crate::constants::init_session_config(session_config).unwrap();
                },
            )?;
            if let Ok(Some(storage)) = web_sys::window().unwrap().session_storage() {
                if let Ok(Some(session_config)) = storage.get(constants::BIOS_SESSION_CONFIG) {
                    let session_config = crypto_process::simple_decrypt(&session_config)?;
                    let session_config = crate::mini_tardis::serde::str_to_obj::<crate::constants::SessionConfig>(&session_config)?;
                    return crate::constants::init_session_config(session_config);
                }
            }
        } else {
            if let Ok(Some(storage)) = web_sys::window().unwrap().local_storage() {
                if let Ok(Some(session_config)) = storage.get(constants::BIOS_SESSION_CONFIG) {
                    let session_config = crypto_process::simple_decrypt(&session_config)?;
                    let session_config = crate::mini_tardis::serde::str_to_obj::<crate::constants::SessionConfig>(&session_config)?;
                    return crate::constants::init_session_config(session_config);
                }
            }
        }
    }
    crate::constants::init_session_config(crate::constants::SessionConfig {
        token: None,
        double_auth_last_time: 0.0,
    })
}

pub(crate) fn change_behavior(_session_config: &SessionConfig, _only_storage: bool) -> TardisResult<()> {
    #[cfg(target_arch = "wasm32")]
    {
        if constants::get_strict_security_mode()? {
            if let Ok(Some(storage)) = web_sys::window().unwrap().session_storage() {
                storage.set(
                    constants::BIOS_SESSION_CONFIG,
                    &crypto_process::simple_encrypt(&crate::mini_tardis::serde::obj_to_str(_session_config)?)?,
                )?;
            }
            if !_only_storage {
                crate::mini_tardis::channel::send(constants::BIOS_SESSION_CONFIG, _session_config)?;
            }
        } else {
            if let Ok(Some(storage)) = web_sys::window().unwrap().local_storage() {
                storage.set(
                    constants::BIOS_SESSION_CONFIG,
                    &crypto_process::simple_encrypt(&crate::mini_tardis::serde::obj_to_str(_session_config)?)?,
                )?;
            }
        }
    }
    Ok(())
}

// In some cases, some modules cannot get the initialized object data,
// so the base data is written to window and read in the corresponding module to achieve data sharing.
pub(crate) fn init_simple_sm_config_by_window() -> TardisResult<Option<(String, String)>> {
    let service_url = js_sys::Reflect::get(
        &wasm_bindgen::JsValue::from(web_sys::window().unwrap()),
        &wasm_bindgen::JsValue::from(constants::BIOS_SERV_URL_CONFIG),
    )?;
    if let Some(service_url) = service_url.as_string() {
        let seed = crypto_process::init_fd_sm4_key(&service_url)?;
        constants::init_simple_sm_config(seed.clone())?;
        Ok(Some(seed))
    } else {
        Ok(None)
    }
}

fn init_config(service_url: &str, serv_config: &ServConfig) -> TardisResult<()> {
    constants::init_simple_sm_config(crypto_process::init_fd_sm4_key(service_url)?)?;
    let mut res_container = ResContainerNode::new();
    for api in &serv_config.apis {
        resource_process::add_res(&mut res_container, &api.action, &api.uri, api.need_crypto_req, api.need_crypto_resp, api.need_double_auth)?;
    }
    let fd_sm2_keys = crypto_process::init_fd_sm2_keys()?;
    let config = StableConfig {
        res_container,
        double_auth_exp_sec: serv_config.double_auth_exp_sec,
        serv_pub_key: crypto::sm::TardisCryptoSm2PublicKey::from_public_key_str(&serv_config.pub_key)?,
        fd_sm2_pub_key: fd_sm2_keys.0,
        fd_sm2_pri_key: fd_sm2_keys.1,
        login_req_method: serv_config.login_req_method.to_lowercase(),
        login_req_paths: serv_config.login_req_paths.iter().map(|i| if i.starts_with('/') { i.clone() } else { format!("/{}", i) }).collect::<Vec<String>>(),
        logout_req_method: serv_config.logout_req_method.to_lowercase(),
        logout_req_path: if serv_config.logout_req_path.starts_with('/') {
            serv_config.logout_req_path.clone()
        } else {
            format!("/{}", &serv_config.logout_req_path)
        },
        double_auth_req_method: serv_config.double_auth_req_method.to_lowercase(),
        double_auth_req_path: if serv_config.double_auth_req_path.starts_with('/') {
            serv_config.double_auth_req_path.clone()
        } else {
            format!("/{}", &serv_config.double_auth_req_path)
        },
    };
    constants::init_stable_config(serv_config.strict_security_mode, config)?;
    Ok(())
}

#[derive(Serialize, Deserialize, Default)]
pub(crate) struct ServConfig {
    pub strict_security_mode: bool,
    pub pub_key: String,
    pub double_auth_exp_sec: u32,
    pub apis: Vec<Api>,
    pub login_req_method: String,
    pub login_req_paths: Vec<String>,
    pub logout_req_method: String,
    pub logout_req_path: String,
    pub double_auth_req_method: String,
    pub double_auth_req_path: String,
}

#[derive(Serialize, Deserialize, Default)]
pub(crate) struct Api {
    pub action: String,
    pub uri: String,
    pub need_crypto_req: bool,
    pub need_crypto_resp: bool,
    pub need_double_auth: bool,
}
