use bios_basic::rbum::dto::rbum_filer_dto::RbumBasicFilterReq;
use serde::{Deserialize, Serialize};
use tardis::{
    basic::{error::TardisError, result::TardisResult},
    chrono::{DateTime, Utc},
    db::sea_orm,
    web::poem_openapi,
};

use crate::{client::sms::SendSmsRequest, dto::*};

#[derive(Debug, poem_openapi::Object, Deserialize)]
pub struct ReachMessageTemplateAddReq {
    pub own_paths: String,
    pub owner: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
    /// 用户触达等级类型
    pub scope_level: Option<i16>,
    /// 编码
    #[oai(validator(max_length = "255"))]
    pub code: Option<String>,
    /// 名称
    #[oai(validator(max_length = "255"))]
    pub name: Option<String>,
    /// 说明
    #[oai(validator(max_length = "2000"), default)]
    pub note: String,
    /// 图标
    #[oai(validator(max_length = "255"), default)]
    pub icon: String,
    /// 排序
    #[oai(default)]
    pub sort: i32,
    /// 是否禁用
    #[oai(default)]
    pub disabled: bool,
    /// 参数
    #[oai(default)]
    pub variables: String,
    /// 用户触达等级类型
    pub level_kind: ReachLevelKind,
    /// 主题
    #[oai(validator(max_length = "255"))]
    pub topic: String,
    /// 内容
    #[oai(validator(max_length = "2000"))]
    pub content: String,
    /// 确认超时时间
    pub timeout_sec: i32,
    /// 确认超时策略
    pub timeout_strategy: ReachTimeoutStrategyKind,
    /// 关联的触达通道
    pub rel_reach_channel: ReachChannelKind,
    /// 模板类型
    pub kind: ReachTemplateKind,
    /// 用户触达验证码策略Id
    #[oai(validator(max_length = "255"))]
    pub rel_reach_verify_code_strategy_id: String,
    /// 第三方插件-模板Id
    #[oai(validator(max_length = "255"))]
    #[oai(default)]
    pub sms_template_id: String,
    /// 第三方插件-签名
    #[oai(validator(max_length = "255"))]
    #[oai(default)]
    pub sms_signature: String,
    /// 第三方插件-短信发送方的号码
    #[oai(validator(max_length = "255"))]
    #[oai(default)]
    pub sms_from: String,
}

#[derive(Debug, poem_openapi::Object, Deserialize)]
pub struct ReachMessageTemplateModifyReq {
    /// 用户触达等级类型
    pub scope_level: Option<i16>,
    /// 编码
    #[oai(validator(max_length = "255"))]
    pub code: Option<String>,
    /// 名称
    #[oai(validator(max_length = "255"))]
    pub name: Option<String>,
    /// 说明
    #[oai(validator(max_length = "2000"), default)]
    pub note: Option<String>,
    /// 图标
    #[oai(validator(max_length = "255"), default)]
    pub icon: Option<String>,
    /// 排序
    #[oai(default)]
    pub sort: Option<i32>,
    /// 是否禁用
    #[oai(default)]
    pub disabled: Option<bool>,
    /// 参数
    #[oai(default)]
    pub variables: Option<String>,
    /// 用户触达等级类型
    pub level_kind: Option<ReachLevelKind>,
    /// 主题
    #[oai(validator(max_length = "255"))]
    pub topic: Option<String>,
    /// 内容
    #[oai(validator(max_length = "2000"))]
    pub content: Option<String>,
    /// 确认超时时间
    pub timeout_sec: Option<i32>,
    /// 确认超时策略
    pub timeout_strategy: Option<ReachTimeoutStrategyKind>,
    /// 关联的触达通道
    pub rel_reach_channel: Option<ReachChannelKind>,
    /// 模板类型
    pub kind: Option<ReachTemplateKind>,
    /// 用户触达验证码策略Id
    #[oai(validator(max_length = "255"))]
    pub rel_reach_verify_code_strategy_id: Option<String>,
    /// 第三方插件-模板Id
    #[oai(validator(max_length = "255"))]
    pub sms_template_id: Option<String>,
    /// 第三方插件-签名
    #[oai(validator(max_length = "255"))]
    pub sms_signature: Option<String>,
    /// 第三方插件-短信发送方的号码
    #[oai(validator(max_length = "255"))]
    pub sms_from: Option<String>,
}

#[derive(Debug, poem_openapi::Object, Default)]
pub struct ReachMessageTemplateFilterReq {
    #[oai(flatten)]
    pub base_filter: RbumBasicFilterReq,
    pub rel_reach_channel: Option<ReachChannelKind>,
    pub level_kind: Option<ReachLevelKind>,
    pub kind: Option<ReachTemplateKind>,
    pub rel_reach_verify_code_strategy_id: Option<String>,
}

#[derive(Debug, poem_openapi::Object, Serialize, sea_orm::FromQueryResult)]
pub struct ReachMessageTemplateSummaryResp {
    pub own_paths: String,
    pub owner: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
    pub scope_level: Option<i16>,
    /// 编码
    #[oai(validator(max_length = "255"))]
    pub code: Option<String>,
    /// 名称
    #[oai(validator(max_length = "255"))]
    pub name: Option<String>,
    /// 说明
    #[oai(validator(max_length = "2000"), default)]
    pub note: String,
    /// 图标
    #[oai(validator(max_length = "255"), default)]
    pub icon: String,
    /// 排序
    #[oai(default)]
    pub sort: i32,
    /// 是否禁用
    #[oai(default)]
    pub disabled: bool,
    /// 参数
    #[oai(default)]
    pub variables: String,
    /// 用户触达等级类型
    pub level_kind: ReachLevelKind,
    /// 主题
    #[oai(validator(max_length = "255"))]
    pub topic: Option<String>,
    /// 内容
    #[oai(validator(max_length = "2000"))]
    pub content: String,
    /// 确认超时时间
    pub timeout_sec: i32,
    /// 确认超时策略
    pub timeout_strategy: ReachTimeoutStrategyKind,
    /// 关联的触达通道
    pub rel_reach_channel: ReachChannelKind,
    /// 模板类型
    pub kind: ReachTemplateKind,
    /// 用户触达验证码策略Id
    #[oai(validator(max_length = "255"))]
    pub rel_reach_verify_code_strategy_id: String,
    /// 第三方插件-模板Id
    #[oai(validator(max_length = "255"))]
    pub sms_template_id: Option<String>,
    /// 第三方插件-签名
    #[oai(validator(max_length = "255"))]
    pub sms_signature: Option<String>,
    /// 第三方插件-短信发送方的号码
    #[oai(validator(max_length = "255"))]
    pub sms_from: Option<String>,
}

#[derive(Debug, poem_openapi::Object, Serialize, sea_orm::FromQueryResult)]
pub struct ReachMessageTemplateDetailResp {
    pub own_paths: String,
    pub owner: String,
    pub owner_name: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
    pub scope_level: Option<i16>,
    /// 编码
    #[oai(validator(max_length = "255"))]
    pub code: Option<String>,
    /// 名称
    #[oai(validator(max_length = "255"))]
    pub name: Option<String>,
    /// 说明
    #[oai(validator(max_length = "2000"), default)]
    pub note: String,
    /// 图标
    #[oai(validator(max_length = "255"), default)]
    pub icon: String,
    /// 排序
    #[oai(default)]
    pub sort: i32,
    /// 是否禁用
    #[oai(default)]
    pub disabled: bool,
    /// 参数
    #[oai(default)]
    pub variables: String,
    /// 用户触达等级类型
    pub level_kind: ReachLevelKind,
    /// 主题
    #[oai(validator(max_length = "255"))]
    pub topic: Option<String>,
    /// 内容
    #[oai(validator(max_length = "2000"))]
    pub content: String,
    /// 确认超时时间
    pub timeout_sec: i32,
    /// 确认超时策略
    pub timeout_strategy: ReachTimeoutStrategyKind,
    /// 关联的触达通道
    pub rel_reach_channel: ReachChannelKind,
    /// 模板类型
    pub kind: ReachTemplateKind,
    /// 用户触达验证码策略Id
    #[oai(validator(max_length = "255"))]
    pub rel_reach_verify_code_strategy_id: String,
    /// 第三方插件-模板Id
    #[oai(validator(max_length = "255"))]
    pub sms_template_id: Option<String>,
    /// 第三方插件-签名
    #[oai(validator(max_length = "255"))]
    pub sms_signature: Option<String>,
    /// 第三方插件-短信发送方的号码
    #[oai(validator(max_length = "255"))]
    pub sms_from: Option<String>,
}

impl ReachMessageTemplateDetailResp {
    pub fn create_send_sms_request<'a>(&'a self, to: &'a str, content: &'a ContentReplace) -> TardisResult<SendSmsRequest<'a>> {
        Ok(SendSmsRequest {
            from: self.sms_from.as_deref().ok_or_else(|| TardisError::conflict("template missing field sms_from", "409-reach-bad-template"))?,
            status_callback: None,
            extend: None,
            to,
            template_id: self.sms_template_id.as_deref().ok_or_else(|| TardisError::conflict("template missing field template_id", "409-reach-bad-template"))?,
            template_paras: content.render_final_content::<20>(&self.content),
            signature: self.sms_signature.as_deref(),
        })
    }
}
