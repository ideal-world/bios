//! # Huawei Cloud Platform Sms Client
//! reference: https://support.huaweicloud.com/msgsms/index.html

use tardis::{
    basic::result::TardisResult,
    chrono::{Utc, SecondsFormat},
    rand::random,
    url::Url,
    web::reqwest::{
        header::{HeaderMap, HeaderValue, AUTHORIZATION},
        Client,
    }, crypto::rust_crypto::sha2::Sha256,
};
mod ext;
mod api;
pub use api::*;
mod model;
pub use model::*;
#[derive(Clone, Debug)]
pub struct SmsClient {
    pub(crate) inner: Client,
    pub app_key: String,
    pub app_secret: String,
    pub base_url: Url,
    pub status_callback: Option<Url>,
}

impl SmsClient {
    const AUTH_WSSE_HEADER_VALUE: &'static str = r#"WSSE realm="SDP",profile="UsernameToken",type="Appkey""#;
    fn add_wsse_headers_to(&self, headers: &mut HeaderMap) -> TardisResult<()> {
        use tardis::crypto::{crypto_base64::TardisCryptoBase64, crypto_digest::TardisCryptoDigest};
        const WSSE_HEADER_NAME: &str = "X-WSSE";
        const BASE64: TardisCryptoBase64 = TardisCryptoBase64;
        const DIGEST: TardisCryptoDigest = TardisCryptoDigest;
        let username = &self.app_key;
        // actually iso-8601
        let created = Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true);
        // and random 1~128bit number
        let nonce = format!("{:X}", random::<u128>());
        let digest_raw = format!("{}{}{}", nonce, created, &self.app_secret);
        let password_digest = BASE64.encode_raw(DIGEST.digest_raw(digest_raw.as_bytes(), Sha256::new())?);
        let wsse_header = format!(r#"UsernameToken Username="{username}",PasswordDigest="{password_digest}",Nonce="{nonce}",Created="{created}""#);
        let wsse_header = HeaderValue::from_str(&wsse_header).expect("Fail to build sms header, maybe there are unexpected char in app_key.");
        headers.insert(AUTHORIZATION, HeaderValue::from_static(Self::AUTH_WSSE_HEADER_VALUE));
        headers.insert(WSSE_HEADER_NAME, wsse_header);
        Ok(())
    }
    fn get_url(&self, path: &str) -> Url {
        let mut new_url = self.base_url.clone();
        let origin_path = new_url.path();
        let new_path = [origin_path.trim_end_matches('/'), path.trim_start_matches('/')].join("/");
        new_url.set_path(&new_path);
        new_url
    }
    pub fn new(base_url: Url, app_key: impl Into<String>, app_secret: impl Into<String>, status_callback: Option<Url>) -> Self {
        let app_key: String = app_key.into();
        let app_secret: String = app_secret.into();

        SmsClient {
            inner: Default::default(),
            base_url,
            app_key,
            app_secret,
            status_callback,
        }
    }
}
