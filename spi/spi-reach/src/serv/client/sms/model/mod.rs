use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SmsResponse<T> {
    pub code: String,
    pub description: String,
    pub result: Option<T>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SmsId {
    pub sms_msg_id: String,
    pub from: String,
    pub origin_to: String,
    pub status: String,
    pub create_time: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
/// see: [SmsClientBatchDiffSendRequest]
///
/// reference: https://support.huaweicloud.com/api-msgsms/sms_05_0002.html#ZH-CN_TOPIC_0000001430352905__table4039578
pub struct SmsContent<'r> {
    pub to: &'r str,
    pub template_id: &'r str,
    pub template_paras: &'r [&'r str],
    pub signature: Option<&'r str>,
}