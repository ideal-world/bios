use serde::Serialize;
use tardis::{
    basic::result::TardisResult,
    url::Url,
    web::reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION},
};

use crate::{model::*, SmsClient};

#[derive(Debug, Serialize, Default)]
#[serde(rename_all = "camelCase")]
/// Referance: https://support.huaweicloud.com/api-msgsms/sms_05_0002.html
pub struct SmsClientBatchDiffSendRequest<'r> {
    pub from: &'r str,
    pub status_callback: Option<&'r str>,
    pub sms_content: &'r [SmsContent<'r>],
    pub extend: Option<&'r str>,
}

impl<'r> SmsClientBatchDiffSendRequest<'r> {
    pub fn new(from: &'r str) -> Self {
        Self { from, ..Default::default() }
    }
}

impl SmsClient {
    pub async fn send_diff_sms(&self, mut request: SmsClientBatchDiffSendRequest<'_>) -> TardisResult<SmsResponse<Vec<SmsId>>> {
        const PATH: &str = "sms/batchSendDiffSms/v1";
        let mut headers = HeaderMap::new();
        request.status_callback = request.status_callback.or(self.status_callback.as_ref().map(Url::as_str));
        headers.insert(AUTHORIZATION, HeaderValue::from_static(Self::AUTH_WSSE_HEADER_VALUE));
        self.add_wsse_headers_to(&mut headers)?;
        let mut url = self.base_url.clone();
        url.set_path(PATH);
        let resp = self.inner.post(url).headers(headers).json(&request).send().await?.json().await?;
        Ok(resp)
    }
}
