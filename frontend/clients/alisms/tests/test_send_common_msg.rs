use std::env;

use bios_client_alisms::SendSmsRequest;
use serde::Deserialize;
use tardis::{basic::result::TardisResult, tokio};
use url::Url;

#[derive(Debug, Deserialize)]
pub struct CustomSmsConfig {
    pub base_url: String,
    pub app_key: String,
    pub app_secret: String,
    pub template_code: String,
    pub phone_numbers: String,
    pub template_param: String,
    pub sign_name: String,
}

#[tokio::test(flavor = "multi_thread")]
async fn send_common_msg() -> TardisResult<()> {
    env::set_var("RUST_LOG", "trace");
    let CustomSmsConfig {
        base_url,
        app_key,
        app_secret,
        template_code,
        phone_numbers,
        template_param,
        sign_name,
    } = {
        let toml_file = std::env::var("CUSTOM_SMS_CONFIG").unwrap_or("./tests/config/custom_sms.toml".to_string());
        let content = std::fs::read_to_string(toml_file).expect("fail to open config file");
        toml::from_str::<CustomSmsConfig>(&content).expect("invalid config")
    };
    let client = bios_client_alisms::SmsClient::new(base_url, app_key, app_secret);

    let req = SendSmsRequest {
        phone_numbers: &phone_numbers,
        template_code: &template_code,
        template_param: Some(template_param),
        sign_name: &sign_name,
    };
    client.send_sms(req).await?;
    Ok(())
}
