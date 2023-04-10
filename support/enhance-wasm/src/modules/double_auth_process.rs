use crate::{
    constants::DOUBLE_AUTH_CACHE_EXP_SEC,
    mini_tardis::{basic::TardisResult, time},
};

use super::resource_process;

pub fn set_latest_authed() -> TardisResult<()> {
    let mut double_auth_exp_sec = DOUBLE_AUTH_CACHE_EXP_SEC.write().unwrap();
    *double_auth_exp_sec = (time::now() + (double_auth_exp_sec.1 * 1000) as f64, double_auth_exp_sec.1);
    Ok(())
}

pub fn need_auth(method: &str, uri: &str) -> TardisResult<bool> {
    if DOUBLE_AUTH_CACHE_EXP_SEC.read().unwrap().0 > time::now() {
        Ok(false)
    } else {
        if let Some(info) = resource_process::match_res(method, uri)?.first() {
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
        initializer::{self, Api, Config},
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
        initializer::do_init(Config {
            strict_security_mode: false,
            pub_key: mock_serv_pub_key.serialize().unwrap(),
            double_auth_exp_sec: 1,
            apis: vec![Api {
                action: "get".to_string(),
                uri: "im/ct/all/**".to_string(),
                need_double_auth: true,
                need_crypto_req: false,
                need_crypto_resp: false,
            }],
        })
        .unwrap();

        assert!(need_auth("GET", "im/ct/all/ddd").unwrap());
        set_latest_authed().unwrap();
        assert!(!need_auth("GET", "im/ct/all/ddd").unwrap());
        sleep(Duration::from_secs(2));
        assert!(need_auth("GET", "im/ct/all/ddd").unwrap());
        assert!(!need_auth("GET", "im/cc/ddd").unwrap());
    }
}
