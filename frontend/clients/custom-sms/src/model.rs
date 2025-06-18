use crate::consts::*;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, time::Duration};
use tardis::chrono::{DateTime, Utc};
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SendCommonMessageRequest {
    /// 发送应用ID
    pub app_id: String,
    /// 发送子应用ID
    pub sub_app_id: Option<String>,
    /// 接收方手机号，多个手机号以逗号分隔，建议相同内容，相同发送方，不同接收人的一一条消息的方式发送，一条消息接收方个数不要超过500个
    pub recv_id: String,
    /// 消息内容
    pub content: String,
    /// 0表示不回复,1表示需要回执,2表示需回复，3表示需要回执＋需回复，4表示需回复（消息队列回复，用户回复消息会经短信平台后发送到指定的消息队列，业务系统接收队列消息）
    pub ack: String,
    /// 回复手机号码(ack为2、3时必填)
    pub reply: Option<String>,
    /// 消息优先级，值为(0到9)数值越大，则优先级越高
    pub priority: Option<String>,
    /// 消息发送重试次数
    pub retry: Option<String>,
    /// 部门号
    pub area_no: Option<String>,
    /// 指定消息的发送时间。format: yyyyMMddHHmmss
    pub send_date: Option<String>,
    /// 消息的有效时长，单位为分钟。如果在有效时间内消息由于网关等原因没有发送出去则丢弃
    pub valid_time: Option<String>,
    /// 短信模版，可以限制短信的有效时间，以及短信内容。短信模版在短信平台这边维护
    pub template_id: Option<String>,
    /// 消息编码，取值范围为1，2，3，4, 建议发给用户的短信都使用3.数据短信使用2
    pub msg_type: String,
    /// 状态报告，0表示不需要（默认），1表示需要
    pub need_report: Option<&'static str>,
    /// 应用唯一序列号或应用流水号（35位长度），当需要状态报告时为必填、需要消息唯一性校验或需要回复到业务系统时必填
    pub app_serial_no: Option<String>,
    /// 拓展字段
    #[serde(flatten)]
    pub ext: HashMap<&'static str, String>,
}

impl SendCommonMessageRequest {
    pub fn build() -> SendCommonMessageBuilder {
        SendCommonMessageBuilder::default()
    }
}

/// 0表示不回复,1表示需要回执,2表示需回复，3表示需要回执＋需回复，4表示需回复（消息队列回复，用户回复消息会经短信平台后发送到指定的消息队列，业务系统接收队列消息）
#[non_exhaustive]
#[repr(u8)]
#[derive(Debug, Clone, Copy, Default)]
pub enum AckType {
    #[default]
    NoReply = 0,
    NeedReceipt = 1,
    NeedReply = 2,
    NeedReceiptAndReply = 3,
    NeedMqReply = 4,
}

impl AckType {
    pub fn into_u8(self) -> u8 {
        self as u8
    }
}

#[non_exhaustive]
#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum MsgType {
    User = 2,
    Data = 3,
}

impl MsgType {
    #[inline]
    pub fn into_u8(self) -> u8 {
        self as u8
    }
}
#[derive(Debug, Default)]
pub struct SendCommonMessageBuilder {
    /// 发送应用ID
    pub app_id: Option<String>,
    /// 发送子应用ID
    pub sub_app_id: Option<String>,
    /// 接收方手机号，多个手机号以逗号分隔，建议相同内容，相同发送方，不同接收人的一一条消息的方式发送，一条消息接收方个数不要超过500个
    pub recv_id: Vec<String>,
    /// 消息内容
    pub content: Option<String>,
    /// 0表示不回复,1表示需要回执,2表示需回复，3表示需要回执＋需回复，4表示需回复（消息队列回复，用户回复消息会经短信平台后发送到指定的消息队列，业务系统接收队列消息）
    pub ack: Option<AckType>,
    /// 回复手机号码(ack为2、3时必填)
    pub reply: Option<String>,
    /// 消息优先级，值为(0到9)数值越大，则优先级越高
    pub priority: Option<u8>,
    /// 消息发送重试次数
    pub retry: Option<u32>,
    /// 部门号
    pub area_no: Option<String>,
    /// 指定消息的发送时间。format: yyyyMMddHHmmss
    pub send_date: Option<DateTime<Utc>>,
    /// 消息的有效时长，单位为分钟。如果在有效时间内消息由于网关等原因没有发送出去则丢弃
    pub valid_time: Option<Duration>,
    /// 短信模版，可以限制短信的有效时间，以及短信内容。短信模版在短信平台这边维护
    pub template_id: Option<String>,
    /// 消息编码，取值范围为1，2，3，4, 建议发给用户的短信都使用3.数据短信使用2
    pub msg_type: Option<MsgType>,
    /// 状态报告，0表示不需要（默认），1表示需要
    pub need_report: Option<bool>,
    /// 应用唯一序列号或应用流水号（35位长度），当需要状态报告时为必填、需要消息唯一性校验或需要回复到业务系统时必填
    pub app_serial_no: Option<String>,
    /// 拓展字段
    pub ext: HashMap<&'static str, String>,
}

macro_rules! builder_field {
    ($field: ident: $Type: ty) => {
        pub fn $field(mut self, $field: $Type) -> Self {
            self.$field = Some($field);
            self
        }
    };
    ($field: ident: $Type: ty | Iter) => {
        pub fn $field<T: Into<$Type>>(mut self, $field: impl IntoIterator<Item = T>) -> Self {
            self.$field.extend($field.into_iter().map(Into::into));
            self
        }
    };
    ($field: ident: $Type: ty | Into) => {
        pub fn $field(mut self, $field: impl Into<$Type>) -> Self {
            self.$field = Some($field.into());
            self
        }
    };
    {$($field: ident: $Type: ty $(| $Mode:tt)?;)*} => {
        $(
            builder_field!{$field:$Type$(| $Mode)?}
        )*
    }
}

impl SendCommonMessageBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    builder_field! {
        app_id: String | Into;
        recv_id: String | Iter;
        content: String | Into;
        ack: AckType;
        reply: String;
        priority: u8;
        retry: u32;
        area_no: String | Into;
        send_date: DateTime<Utc>;
        valid_time: Duration;
        template_id: String | Into;
        msg_type: MsgType;
        need_report: bool;
        app_serial_no: String | Into;
    }

    pub fn build(self) -> Result<SendCommonMessageRequest, &'static str> {
        // check recv_id number
        if self.recv_id.len() >= MAX_RECV_ID_LIMIT {
            return Err("to much recv_ids");
        }
        // check content
        if self.content.as_ref().is_some_and(String::is_empty) {
            return Err("content cannot be empty");
        }
        // check ack
        if matches!(self.ack, Some(AckType::NeedReply) | Some(AckType::NeedReceiptAndReply)) && self.reply.is_none() {
            return Err("missing reply");
        }
        // check ser no
        if self.app_serial_no.as_ref().is_some_and(|n| n.len() != 35) {
            return Err("invalid app_serial_no");
        }
        Ok(SendCommonMessageRequest {
            app_id: self.app_id.ok_or("missing app_id")?,
            sub_app_id: self.sub_app_id,
            recv_id: self.recv_id.join(","),
            content: self.content.ok_or("missing content")?,
            ack: self.ack.unwrap_or_default().into_u8().to_string(),
            reply: self.reply,
            priority: self.priority.map(|p| p.min(9).to_string()),
            retry: self.retry.map(|r| r.to_string()),
            area_no: self.area_no,
            send_date: self.send_date.map(|d| d.format("%Y%M%d%H%m%s").to_string()),
            valid_time: self.valid_time.map(|v| (v.as_secs() % 60).to_string()),
            template_id: self.template_id,
            msg_type: self.msg_type.ok_or("missing msg_type")?.into_u8().to_string(),
            need_report: self.need_report.map(|b| if b { "1" } else { "0" }),
            app_serial_no: self.app_serial_no,
            ext: self.ext,
        })
    }
}

#[derive(Debug, Deserialize)]
pub struct CustomSmsResponse<T> {
    pub code: i32,
    pub msg: Option<String>,
    pub data: Option<T>,
}

#[derive(Debug, Deserialize)]
pub struct SendCommonMessageResponse {
    pub code: String,
    pub desc: Option<String>,
}

impl SendCommonMessageResponse {
    #[inline]
    pub fn is_ok(&self) -> bool {
        self.code == SUCCESS_CODE
    }
}
