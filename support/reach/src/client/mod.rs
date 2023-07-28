use std::{collections::HashSet, sync::Arc};

use tardis::{
    async_trait::async_trait,
    basic::{error::TardisError, result::TardisResult},
    mail::mail_client::TardisMailSendReq,
};

use crate::{consts::*, domain::message_template, dto::*};

use self::sms::SendSmsRequest;

pub mod email;
pub mod sms;

pub struct GenericTemplate<'t> {
    pub name: Option<&'t str>,
    pub content: &'t str,
    pub sms_from: Option<&'t str>,
    pub sms_template_id: Option<&'t str>,
    pub sms_signature: Option<&'t str>,
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
    async fn send(&self, template: GenericTemplate<'_>, content: &ContentReplace, to: &HashSet<&str>) -> TardisResult<()> {
        let request = SendSmsRequest {
            from: template.sms_from.ok_or_else(|| TardisError::conflict("template missing field sms_from", "409-reach-bad-template"))?,
            status_callback: None,
            extend: None,
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
            template_paras: content.render_final_content::<20>(template.content),
            signature: template.sms_signature,
        };
        self.send_sms(request).await?;
        Ok(())
    }
}

/// 集成发送通道
#[derive(Clone)]
pub struct SendChannelAll {
    pub sms_client: Arc<sms::SmsClient>,
    pub mail_client: email::MailClient,
}

impl Default for SendChannelAll {
    fn default() -> Self {
        Self {
            sms_client: get_sms_client(),
            mail_client: get_mail_client(),
        }
    }
}

impl SendChannelAll {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn get_channel(&self, kind: ReachChannelKind) -> Arc<dyn SendChannel + Send + Sync> {
        match kind {
            ReachChannelKind::Sms => self.sms_client.clone(),
            ReachChannelKind::Email => Arc::new(self.mail_client),
            _ => Arc::new(UnimplementedChannel(kind)),
        }
    }
    pub async fn send(&self, kind: ReachChannelKind, template: impl Into<GenericTemplate<'_>>, content: &ContentReplace, to: &HashSet<impl AsRef<str>>) -> TardisResult<()> {
        self.get_channel(kind).send(template.into(), content, &to.iter().map(|x|x.as_ref()).collect()).await
    }
}