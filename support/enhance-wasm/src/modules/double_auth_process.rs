use crate::{
    constants::{self, CONFIG},
    mini_tardis::{basic::TardisResult, time},
};

use super::resource_process;

pub fn set_latest_authed() -> TardisResult<()> {
    let double_auth_exp_sec = {
        let config = CONFIG.read().unwrap();
        config.as_ref().unwrap().double_auth_exp_sec
    };
    constants::conf_by_double_auth_last_time(time::now() + (double_auth_exp_sec * 1000) as f64)
}

pub fn need_auth(method: &str, uri: &str) -> TardisResult<bool> {
    let method = method.to_lowercase();
    let config = CONFIG.read().unwrap();
    let config = config.as_ref().unwrap();
    if config.double_auth_last_time > time::now() {
        Ok(false)
    } else {
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
        constants,
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
        constants::remove_config().unwrap();
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
            },
        )
        .unwrap();

        assert!(need_auth("GET", "iam/ct/all/ddd").unwrap());
        set_latest_authed().unwrap();
        assert!(!need_auth("GET", "iam/ct/all/ddd").unwrap());
        sleep(Duration::from_secs(2));
        assert!(need_auth("GET", "iam/ct/all/ddd").unwrap());
        assert!(!need_auth("GET", "iam/cc/ddd").unwrap());
    }
}
