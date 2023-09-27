use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use tardis::{
    async_trait::async_trait,
    basic::{error::TardisError, result::TardisResult},
    mail::mail_client::TardisMailSendReq,
    {log as tracing, log::instrument},
};

use crate::{client::sms::SmsContent, config::ReachConfig, domain::message_template, dto::*};

use self::sms::SendSmsRequest;

pub mod email;
pub mod sms;

#[derive(Default, Debug)]
pub struct GenericTemplate<'t> {
    pub name: Option<&'t str>,
    pub content: &'t str,
    pub sms_from: Option<&'t str>,
    pub sms_template_id: Option<&'t str>,
    pub sms_signature: Option<&'t str>,
}

impl<'t> GenericTemplate<'t> {
    pub fn pwd_template(config: &'t ReachConfig) -> Self {
        Self {
            name: None,
            content: "{pwd}",
            sms_from: Some(&config.sms.sms_general_from),
            sms_template_id: Some(&config.sms.sms_pwd_template_id),
            sms_signature: config.sms.sms_general_signature.as_deref(),
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
        }
    }
}

#[async_trait]
pub trait SendChannel {
    async fn send(&self, template: GenericTemplate<'_>, content: &ContentReplace, to: &HashSet<&str>) -> TardisResult<()>;
}
fn bad_template(msg: impl AsRef<str>) -> TardisError {
    TardisError::conflict(msg.as_ref(), "409-reach-bad-template")
}
#[derive(Clone, Copy)]
pub struct UnimplementedChannel(pub ReachChannelKind);

#[async_trait]
impl SendChannel for UnimplementedChannel {
    async fn send(&self, _template: GenericTemplate<'_>, _content: &ContentReplace, _to: &HashSet<&str>) -> TardisResult<()> {
        let message = format!("trying to send through an unimplemented channel [{kind}]", kind = self.0);
        Err(TardisError::conflict(&message, "500-unimplemented-channel"))
    }
}

#[async_trait]
impl SendChannel for email::MailClient {
    async fn send(&self, template: GenericTemplate<'_>, content: &ContentReplace, to: &HashSet<&str>) -> TardisResult<()> {
        self.inner
            .send(&TardisMailSendReq {
                subject: template.name.ok_or_else(|| bad_template("template missing field sms_from"))?.to_owned(),
                txt_body: content.render_final_content::<{ usize::MAX }>(template.content),
                html_body: None,
                to: to.iter().map(|x| x.to_string()).collect(),
                reply_to: None,
                cc: None,
                bcc: None,
                from: None,
            })
            .await
    }
}

#[async_trait]
impl SendChannel for sms::SmsClient {
    #[instrument(skip(self), fields(module = "reach"))]
    async fn send(&self, template: GenericTemplate<'_>, content: &ContentReplace, to: &HashSet<&str>) -> TardisResult<()> {
        let content = content.render_final_content::<20>(template.content);
        tardis::log::trace!("send sms {content}");
        let sms_content = SmsContent {
            to: &to.iter().fold(
                // 11 digits + 1 comma, it's an estimate
                String::with_capacity(to.len() * 12),
                |mut acc, x| {
                    if !acc.is_empty() {
                        acc.push(',');
                    }
                    acc.push_str(x);
                    acc
                },
            ),
            template_id: template.sms_template_id.ok_or_else(|| TardisError::conflict("template missing field template_id", "409-reach-bad-template"))?,
            template_paras: vec![&content],
            signature: template.sms_signature,
        };
        let from = template.sms_from.ok_or_else(|| TardisError::conflict("template missing field sms_from", "409-reach-bad-template"))?;
        let request = SendSmsRequest::new(from, sms_content);
        let resp = self.send_sms(request).await?;
        if resp.is_error() {
            use std::fmt::Write;
            let mut error_buffer = String::new();
            writeln!(&mut error_buffer, "send sms error [{code}]: {desc}.", code = resp.code, desc = resp.description).expect("write to string shouldn't fail");
            if let Some(ids) = resp.result {
                writeln!(&mut error_buffer, "Detail: ").expect("write to string shouldn't fail");
                for detail in ids {
                    writeln!(&mut error_buffer, "{:?}", detail).expect("write to string shouldn't fail");
                }
            }
            return Err(TardisError::conflict(&error_buffer, "409-reach-sms-error"));
        }
        Ok(())
    }
}

/// 集成发送通道
#[derive(Clone, Default)]
pub struct SendChannelMap {
    pub channels: HashMap<ReachChannelKind, Arc<dyn SendChannel + Send + Sync>>,
}

impl SendChannelMap {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn with_channel(mut self, kind: ReachChannelKind, channel: Arc<dyn SendChannel + Send + Sync>) -> Self {
        self.channels.insert(kind, channel);
        self
    }
    pub fn get_channel(&self, kind: ReachChannelKind) -> Arc<dyn SendChannel + Send + Sync> {
        self.channels.get(&kind).cloned().unwrap_or(Arc::new(UnimplementedChannel(kind)))
    }
    pub async fn send(&self, kind: ReachChannelKind, template: impl Into<GenericTemplate<'_>>, content: &ContentReplace, to: &HashSet<impl AsRef<str>>) -> TardisResult<()> {
        self.get_channel(kind).send(template.into(), content, &to.iter().map(|x| x.as_ref()).collect()).await
    }
}
