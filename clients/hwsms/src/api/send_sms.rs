use serde::Serialize;
use tardis::{basic::result::TardisResult, serde_json, url::Url, web::reqwest::header::HeaderMap};

use crate::{model::*, SmsClient};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SendSmsRequest<'r> {
    pub from: &'r str,
    pub status_callback: Option<&'r str>,
    pub extend: Option<&'r str>,
    pub to: &'r str,
    pub template_id: &'r str,
    pub template_paras: Option<String>,
    pub signature: Option<&'r str>,
}

impl<'r> SendSmsRequest<'r> {
    pub fn new(from: &'r str, sms_content: SmsContent<'r>) -> Self {
        let template_paras = if !sms_content.template_paras.is_empty() {
            Some(serde_json::to_string(&sms_content.template_paras).expect("string[] to rust String shouldn't fail"))
        } else {
            None
        };
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
    pub async fn send_sms(&self, mut request: SendSmsRequest<'_>) -> TardisResult<SmsResponse<Vec<SmsId>>> {
        tardis::log::trace!("send sms request: {:?}", request);
        const PATH: &str = "sms/batchSendSms/v1";
        request.status_callback = request.status_callback.or(self.status_callback.as_ref().map(Url::as_str));
        let mut headers = HeaderMap::new();
        self.add_wsse_headers_to(&mut headers)?;
        let url = self.get_url(PATH);
        let builder = self.inner.post(url).headers(headers).form(&request);
        let resp = builder.send().await?.json().await?;
        tardis::log::trace!("send sms response: {:?}", resp);
        Ok(resp)
    }
}
