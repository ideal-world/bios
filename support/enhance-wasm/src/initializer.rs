use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{
    constants::{self, Config, CONFIG},
    mini_tardis::{basic::TardisResult, crypto, http, log},
    modules::{
        crypto_process,
        resource_process::{self, ResContainerNode},
    },
};

pub(crate) async fn init(service_url: &str, serv_config: Option<ServConfig>) -> TardisResult<bool> {
    constants::init()?;

    if !constants::is_conf_unset()? {
        return Ok(CONFIG.read().unwrap().as_ref().unwrap().strict_security_mode);
    }

    let service_url = if service_url.ends_with("/") {
        service_url.to_string()
    } else {
        format!("{service_url}/")
    };
    let serv_config = if let Some(serv_config) = serv_config {
        log::log(&format!("[BIOS] Init by spec config."));
        serv_config
    } else {
        log::log(&format!("[BIOS] Init by url: {service_url}."));
        http::request::<ServConfig>("GET", &format!("{service_url}auth/apis"), None, HashMap::new()).await?.unwrap()
    };
    do_init(&service_url, &serv_config)
}

pub(crate) fn do_init(service_url: &str, serv_config: &ServConfig) -> TardisResult<bool> {
    let mut res_container = ResContainerNode::new();
    for api in &serv_config.apis {
        resource_process::add_res(&mut res_container, &api.action, &api.uri, api.need_crypto_req, api.need_crypto_resp, api.need_double_auth)?;
    }
    let fd_sm2_keys = crypto_process::init_fd_sm2_keys()?;
    let config = Config {
        strict_security_mode: serv_config.strict_security_mode,
        serv_url: service_url.to_string(),
        token: None,
        double_auth_last_time: 0.0,
        double_auth_exp_sec: serv_config.double_auth_exp_sec,
        res_container: res_container,
        serv_pub_key: crypto::sm::TardisCryptoSm2PublicKey::from_public_key_str(&serv_config.pub_key)?,
        fd_sm2_pub_key: fd_sm2_keys.0,
        fd_sm2_pri_key: fd_sm2_keys.1,
        fd_sm4_key: crypto_process::init_fd_sm4_key()?,
        login_req_method: serv_config.login_req_method.to_lowercase(),
        login_req_paths: serv_config.login_req_paths.iter().map(|i| if i.starts_with("/") { i.clone() } else { format!("/{}", i) }).collect::<Vec<String>>(),
        logout_req_method: serv_config.logout_req_method.to_lowercase(),
        logout_req_path: if serv_config.logout_req_path.starts_with("/") {
            serv_config.logout_req_path.clone()
        } else {
            format!("/{}", &serv_config.logout_req_path)
        },
        double_auth_req_method: serv_config.double_auth_req_method.to_lowercase(),
        double_auth_req_path: if serv_config.double_auth_req_path.starts_with("/") {
            serv_config.double_auth_req_path.clone()
        } else {
            format!("/{}", &serv_config.double_auth_req_path)
        },
    };
    let strict_security_mode = config.strict_security_mode;
    constants::set_config(config)?;
    Ok(strict_security_mode)
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
