use std::collections::HashMap;

use super::crypto_process;
use crate::mini_tardis::{self, basic::TardisResult, time};
use serde::{Deserialize, Serialize};

pub fn mix(method: &str, uri: &str, body: &str, mut headers: HashMap<String, String>) -> TardisResult<MixRequest> {
    let mix_body = MixRequestBody {
        method: method.to_string(),
        uri: uri.to_string(),
        body: body.to_string(),
        headers: headers.clone(),
        ts: time::now(),
    };
    let mix_body = mini_tardis::serde::obj_to_str(&mix_body)?;
    let resp = crypto_process::do_encrypt(&mix_body, true, true)?;
    headers.extend(resp.additional_headers);
    let resp = MixRequest {
        method: "POST".to_string(),
        uri: "apis".to_string(),
        body: resp.body,
        headers,
    };
    Ok(resp)
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct MixRequestBody {
    pub method: String,
    pub uri: String,
    pub body: String,
    pub headers: HashMap<String, String>,
    pub ts: f64,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct MixRequest {
    pub method: String,
    pub uri: String,
    pub body: String,
    pub headers: HashMap<String, String>,
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{
        constants::TARDIS_CRYPTO,
        initializer::{self, Config},
        mini_tardis::crypto::{
            self,
            sm::{TardisCryptoSm2, TardisCryptoSm4},
        },
        modules::global_api_process::MixRequest,
    };

    use super::mix;

    #[test]
    fn test_mix() {
        // Prepare
        let sm2 = TardisCryptoSm2 {};
        let mock_serv_pri_key = sm2.new_private_key().unwrap();
        let mock_serv_pub_key = sm2.new_public_key(&mock_serv_pri_key).unwrap();
        initializer::do_init(Config {
            strict_security_mode: false,
            pub_key: mock_serv_pub_key.serialize().unwrap(),
            double_auth_exp_sec: 1,
            apis: vec![],
        })
        .unwrap();

        let mock_req_body = "中台经过几年“滚雪球”的发展或是资本地运作，已是个“庞然大物”，是到了“减肥”，“减负”的时候。一言以避之：解构中台，让他融合到更大的IT能力共享架构中，把共享交给开放平台，把技术还给技术平台，让中台专注于领域服务及事件 。";
        let resp = mix("PUT", "im/ct/xxx", mock_req_body, HashMap::from([("Test-Key".to_string(), "xxx".to_string())])).unwrap();
        assert_eq!(resp.method, "POST");
        assert_eq!(resp.uri, "apis");
        assert_eq!(resp.headers.len(), 2);

        // Mock serv process
        // ------------------------------------------------
        // 1. Decrypt request key by service private key
        let key = resp.headers.get(TARDIS_CRYPTO).unwrap();
        let encrypt_body = resp.body;
        let key = mock_serv_pri_key.decrypt(&crypto::base64::decode(key).unwrap()).unwrap();
        let key = key.split(" ").collect::<Vec<&str>>();
        assert_eq!(key.len(), 3);
        let sm4_key = key[0];
        let sm4_iv = key[1];
        sm2.new_public_key_from_public_key(key[2]).unwrap();
        // 2. Decrypt request body by frontend sm4 key & iv
        let mix_request = serde_json::from_str::<MixRequest>(&TardisCryptoSm4 {}.decrypt_cbc(&encrypt_body, sm4_key, sm4_iv).unwrap()).unwrap();
        assert_eq!(mix_request.method, "PUT");
        assert_eq!(mix_request.uri, "im/ct/xxx");
        assert_eq!(mix_request.headers.len(), 1);
        assert_eq!(mix_request.headers["Test-Key"], "xxx");
        assert_eq!(mix_request.body, mock_req_body);
        // ------------------------------------------------
    }
}
