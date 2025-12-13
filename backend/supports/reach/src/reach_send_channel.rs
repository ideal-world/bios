use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use tardis::{
    TardisFuns, async_trait::async_trait, basic::{error::TardisError, result::TardisResult}, mail::mail_client::TardisMailSendReq, serde_json, web::reqwest::{Client as ReqwestClient, Method}
};

use crate::{domain::message_template, dto::*, reach_config::ReachConfig};

#[derive(Default, Debug)]
pub struct GenericTemplate<'t> {
    pub name: Option<&'t str>,
    pub content: &'t str,
    pub sms_from: Option<&'t str>,
    pub sms_template_id: Option<&'t str>,
    pub sms_signature: Option<&'t str>,
    pub topic: Option<&'t str>,
}

impl<'t> GenericTemplate<'t> {
    pub fn pwd_template(config: &'t ReachConfig) -> Self {
        Self {
            name: None,
            content: r#"["{pwd}"]"#,
            sms_from: Some(&config.sms.sms_general_from),
            sms_template_id: Some(&config.sms.sms_pwd_template_id),
            sms_signature: config.sms.sms_general_signature.as_deref(),
            topic: None,
        }
    }
}

impl<'t> From<&'t message_template::Model> for GenericTemplate<'t> {
    fn from(value: &'t message_template::Model) -> Self {
        GenericTemplate {
            name: Some(&value.name),
            content: &value.content,
            sms_from: Some(&value.sms_from),
            sms_template_id: Some(&value.sms_template_id),
            sms_signature: Some(&value.sms_signature),
            topic: Some(&value.topic),
        }
    }
}

impl<'t> From<&'t ReachMessageTemplateDetailResp> for GenericTemplate<'t> {
    fn from(value: &'t ReachMessageTemplateDetailResp) -> Self {
        GenericTemplate {
            name: value.name.as_deref(),
            content: &value.content,
            sms_from: value.sms_from.as_deref(),
            sms_template_id: value.sms_template_id.as_deref(),
            sms_signature: value.sms_signature.as_deref(),
            topic: value.topic.as_deref(),
        }
    }
}

impl<'t> From<&'t ReachMessageTemplateSummaryResp> for GenericTemplate<'t> {
    fn from(value: &'t ReachMessageTemplateSummaryResp) -> Self {
        GenericTemplate {
            name: value.name.as_deref(),
            content: &value.content,
            sms_from: value.sms_from.as_deref(),
            sms_template_id: value.sms_template_id.as_deref(),
            sms_signature: value.sms_signature.as_deref(),
            topic: value.topic.as_deref(),
        }
    }
}

#[async_trait]
pub trait SendChannel: Send + Sync {
    fn kind(&self) -> ReachChannelKind;
    async fn send(&self, template: GenericTemplate<'_>, content: &ContentReplace, to: &HashSet<&str>) -> TardisResult<()>;
}
fn bad_template(msg: impl AsRef<str>) -> TardisError {
    TardisError::conflict(msg.as_ref(), "409-reach-bad-template")
}
#[derive(Clone, Copy, Debug)]
pub struct UnimplementedChannel(pub ReachChannelKind);

#[async_trait]
impl SendChannel for UnimplementedChannel {
    async fn send(&self, _template: GenericTemplate<'_>, _content: &ContentReplace, _to: &HashSet<&str>) -> TardisResult<()> {
        let message = format!("trying to send through an unimplemented channel [{kind}]", kind = self.0);
        Err(TardisError::conflict(&message, "500-unimplemented-channel"))
    }
    fn kind(&self) -> ReachChannelKind {
        self.0
    }
}

#[async_trait]
impl SendChannel for tardis::mail::mail_client::TardisMailClient {
    async fn send(&self, template: GenericTemplate<'_>, content: &ContentReplace, to: &HashSet<&str>) -> TardisResult<()> {
        self.send(
            &TardisMailSendReq::builder()
                .subject(template.name.ok_or_else(|| bad_template("template missing field sms_from"))?)
                .txt_body(content.render_final_content::<{ usize::MAX }>(template.content))
                .to(to.iter().map(|x| x.to_string()).collect::<Vec<_>>())
                .build(),
        )
        .await
    }
    fn kind(&self) -> ReachChannelKind {
        ReachChannelKind::Email
    }
}

/// WebHook 通道实现
/// 使用模板的 sms_from 字段存储 webhook URL
#[derive(Clone, Debug)]
pub struct WebHookChannel {
    client: Arc<ReqwestClient>,
}

impl WebHookChannel {
    pub fn new() -> TardisResult<Self> {
        Ok(Self {
            client: Arc::new(ReqwestClient::builder().build()?),
        })
    }
}

impl Default for WebHookChannel {
    fn default() -> Self {
        Self::new().expect("Failed to create WebHookChannel")
    }
}

#[async_trait]
impl SendChannel for WebHookChannel {
    async fn send(&self, template: GenericTemplate<'_>, content: &ContentReplace, _to: &HashSet<&str>) -> TardisResult<()> {
        // 从模板的 topic 字段获取 webhook URL
        let webhook_url = template.topic.ok_or_else(|| bad_template("template missing webhook URL (topic field)"))?;
        
        // 从 content 中获取 webhook_method，默认为 POST
        let method_str = content.get("webhook_method").map(|s| s.as_str()).unwrap_or("POST");
        let method = Method::from_bytes(method_str.as_bytes())
            .map_err(|e| TardisError::wrap(&format!("Invalid HTTP method '{}': {}", method_str, e), "400-invalid-webhook-method"))?;
        
        // 根据 webhook_method 发送请求到 webhook URL
        let mut request_builder = self.client.request(method.clone(), webhook_url);
        
        // 对于支持请求体的方法（POST, PUT, PATCH），添加请求体
        match method {
            Method::GET | Method::DELETE | Method::HEAD | Method::OPTIONS | Method::TRACE => {}
            _ => {
                // 从 content 中获取 webhook_content 作为请求体
                if let Some(webhook_content) = content.get("webhook_content") {
                    let json_value = TardisFuns::json.str_to_json(webhook_content)?;
                    request_builder = request_builder.json(&json_value);
                }
            }
        }
        
        let response = request_builder
            .send()
            .await
            .map_err(|e| TardisError::wrap(&format!("Failed to send webhook request: {}", e), "500-webhook-send-error"))?;
        
        // 检查响应状态
        if !response.status().is_success() {
            let status = response.status();
            let error_body = response.text().await.unwrap_or_default();
            return Err(TardisError::wrap(
                &format!("Webhook request failed with status {}: {}", status, error_body),
                "500-webhook-response-error",
            ));
        }
        
        Ok(())
    }
    
    fn kind(&self) -> ReachChannelKind {
        ReachChannelKind::WebHook
    }
}

/// 集成发送通道，每个`ReachChannelKind`对应一个实例
#[derive(Clone, Default)]
pub struct SendChannelMap {
    pub channels: HashMap<ReachChannelKind, Arc<dyn SendChannel + Send + Sync>>,
}

impl SendChannelMap {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn with_arc_channel<C>(mut self, channel: Arc<C>) -> Self
    where
        C: SendChannel + Send + Sync + 'static,
    {
        self.channels.insert(channel.kind(), channel);
        self
    }
    pub fn with_channel<C>(self, channel: C) -> Self
    where
        C: SendChannel + Send + Sync + 'static,
    {
        self.with_arc_channel(Arc::new(channel))
    }
    pub fn get_channel(&self, kind: ReachChannelKind) -> Arc<dyn SendChannel + Send + Sync> {
        self.channels.get(&kind).cloned().unwrap_or(Arc::new(UnimplementedChannel(kind)))
    }
    pub async fn send(&self, kind: ReachChannelKind, template: impl Into<GenericTemplate<'_>>, content: &ContentReplace, to: &HashSet<impl AsRef<str>>) -> TardisResult<()> {
        self.get_channel(kind).send(template.into(), content, &to.iter().map(|x| x.as_ref()).collect()).await
    }
}
