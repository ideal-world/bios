use bios_client_custom_sms::{AckType, MsgType, SendCommonMessageRequest};
use serde::Deserialize;
use tardis::{basic::result::TardisResult, tokio};
use url::Url;

#[derive(Debug, Deserialize)]
pub struct CustomSmsConfig {
    pub base_url: Url,
    pub app_id: String,
    pub app_pwd: String,
    pub area_no: Option<String>,
    pub revicer: String,
    pub content: String,
}

#[tokio::test(flavor = "multi_thread")]
async fn send_common_msg() -> TardisResult<()> {
    let CustomSmsConfig {
        base_url,
        app_id,
        area_no,
        app_pwd,
        revicer,
        content,
    } = {
        let toml_file = std::env::var("CUSTOM_SMS_CONFIG").unwrap_or("./tests/config/custom_sms.toml".to_string());
        let content = std::fs::read_to_string(toml_file).expect("fail to open config file");
        toml::from_str::<CustomSmsConfig>(&content).expect("invalid config")
    };
    let client = bios_client_custom_sms::Client::init(base_url, app_id.parse().expect("not valid header value"), app_pwd.parse().expect("not valid header value"))?;

    let mut req = SendCommonMessageRequest::build()
        .need_report(false)
        .ack(AckType::NoReply)
        .msg_type(MsgType::User)
        .app_id(&app_id)
        .recv_id(Some(revicer))
        .content(content)
        .build()
        .expect("build request");
    req.area_no = area_no;
    client.send_message(&req).await?;
    Ok(())
}
