use serde::Serialize;
use tardis::{
    basic::result::TardisResult,
    serde_json,
    web::reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION},
};

use crate::client::sms::{model::*, SmsClient};

#[derive(Debug, Serialize)]
pub struct SendSmsRequest<'r> {
    pub from: &'r str,
    pub status_callback: Option<&'r str>,
    pub extend: Option<&'r str>,
    pub to: &'r str,
    pub template_id: &'r str,
    pub template_paras: String,
    pub signature: Option<&'r str>,
}

impl<'r> SendSmsRequest<'r> {
    pub fn new(from: &'r str, sms_content: SmsContent<'r>) -> Self {
        let template_paras = serde_json::to_string(sms_content.template_paras).expect("string[] to rust String shouldn't fail");
        Self {
            from,
            status_callback: None,
            extend: None,
            to: sms_content.to,
            template_id: sms_content.template_id,
            template_paras,
            signature: sms_content.signature,
        }
    }
}

impl SmsClient {
    pub async fn send_sms(&self, request: SendSmsRequest<'_>) -> TardisResult<SmsResponse<Vec<SmsId>>> {
        const PATH: &str = "sms/batchSendSms/v1";
        let mut headers = HeaderMap::new();
        self.add_wsse_headers_to(&mut headers)?;
        let url = self.get_url(PATH);
        let resp = self.inner.post(url).form(&request).send().await?.json().await?;
        Ok(resp)
    }
}
