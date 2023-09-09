use crate::{
    constants::{SESSION_CONFIG, STABLE_CONFIG},
    initializer,
    mini_tardis::{basic::TardisResult, time},
};

use super::resource_process;

pub fn set_latest_authed() -> TardisResult<()> {
    let double_auth_exp_sec = {
        let config = STABLE_CONFIG.read().unwrap();
        config.as_ref().unwrap().double_auth_exp_sec
    };
    let mut config_container = SESSION_CONFIG.write().unwrap();
    let session_config = config_container.as_mut().unwrap();
    session_config.double_auth_last_time = time::now() + (double_auth_exp_sec * 1000) as f64;
    initializer::change_behavior(session_config, false)
}

pub fn remove_latest_authed() -> TardisResult<()> {
    let mut config_container = SESSION_CONFIG.write().unwrap();
    let session_config = config_container.as_mut().unwrap();
    session_config.double_auth_last_time = 0.0;
    initializer::change_behavior(session_config, false)
}

pub fn need_auth(method: &str, uri: &str) -> TardisResult<bool> {
    let method = method.to_lowercase();
    let config = SESSION_CONFIG.read().unwrap();
    if config.as_ref().unwrap().double_auth_last_time > time::now() {
        Ok(false)
    } else {
        let config = STABLE_CONFIG.read().unwrap();
        let config = config.as_ref().unwrap();
        if let Some(info) = resource_process::match_res(&config.res_container, &method, uri)?.first() {
            Ok(info.need_double_auth)
        } else {
            Ok(false)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{thread::sleep, time::Duration};

    use crate::{
        initializer::{self, Api, ServConfig},
        mini_tardis::crypto::sm::TardisCryptoSm2,
        modules::double_auth_process::set_latest_authed,
    };

    use super::need_auth;

    #[test]
    fn test_double_auth() {
        // Prepare
        let sm2 = TardisCryptoSm2 {};
        let mock_serv_pri_key = sm2.new_private_key().unwrap();
        let mock_serv_pub_key = sm2.new_public_key(&mock_serv_pri_key).unwrap();
        initializer::do_init(
            "",
            &ServConfig {
                strict_security_mode: false,
                pub_key: mock_serv_pub_key.serialize().unwrap(),
                double_auth_exp_sec: 1,
                apis: vec![Api {
                    action: "get".to_string(),
                    uri: "iam/ct/all/**".to_string(),
                    need_double_auth: true,
                    need_crypto_req: false,
                    need_crypto_resp: false,
                }],
                login_req_method: "".to_string(),
                login_req_paths: vec![],
                logout_req_method: "".to_string(),
                logout_req_path: "".to_string(),
                double_auth_req_method: "".to_string(),
                double_auth_req_path: "".to_string(),
                exclude_encrypt_decrypt_path: "".to_string(),
            },
        )
        .unwrap();

        assert!(need_auth("GET", "iam/ct/all/ddd").unwrap());
        set_latest_authed().unwrap();
        assert!(!need_auth("GET", "iam/ct/all/ddd").unwrap());
        sleep(Duration::from_secs(2));
        assert!(need_auth("GET", "iam/ct/all/ddd").unwrap());
        assert!(!need_auth("GET", "iam/cc/ddd").unwrap());
        assert!(!need_auth("GET", "iam/cc/ddd?test1=1&test2=2").unwrap());
    }
}
