use tardis::{
    async_trait::async_trait,
    basic::{error::TardisError, result::TardisResult},
    mail::mail_client::TardisMailSendReq,
};

use crate::dto::*;

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
    async fn send(&self, template: GenericTemplate<'_>, content: &ContentReplace, to: &str) -> TardisResult<()>;
}
fn bad_template(msg: impl AsRef<str>) -> TardisError {
    TardisError::conflict(msg.as_ref(), "409-reach-bad-template")
}
#[derive(Clone, Copy)]
pub struct UnimplementedChannel(pub ReachChannelKind);
macro_rules! const_inst {
    (
        $($variant:ident),*
    ) => {
        #[allow(non_upper_case_globals)]
        pub const fn get_const_ref(kind: ReachChannelKind) -> &'static Self {
            $(
                const $variant: UnimplementedChannel = Self(ReachChannelKind::$variant);
            )*
            match kind {
                $(
                    ReachChannelKind::$variant => &$variant,
                )*
            }
        }
    };
}
impl UnimplementedChannel {
    const_inst! {Sms, Email, Inbox, Wechat, DingTalk, Push, WebHook}
}

#[async_trait]
impl SendChannel for UnimplementedChannel {
    async fn send(&self, _template: GenericTemplate<'_>, _content: &ContentReplace, _to: &str) -> TardisResult<()> {
        let message = format!("trying to send through an unimplemented channel [{kind}]", kind = self.0);
        Err(TardisError::conflict(&message, "500-unimplemented-channel"))
    }
}

#[async_trait]
impl SendChannel for email::MailClient {
    async fn send(&self, template: GenericTemplate<'_>, content: &ContentReplace, to: &str) -> TardisResult<()> {
        self.inner
            .send(&TardisMailSendReq {
                subject: template.name.ok_or_else(|| bad_template("template missing field sms_from"))?.to_owned(),
                txt_body: content.render_final_content::<{ usize::MAX }>(template.content),
                html_body: None,
                to: vec![to.to_string()],
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
    async fn send(&self, template: GenericTemplate<'_>, content: &ContentReplace, to: &str) -> TardisResult<()> {
        let request = SendSmsRequest {
            from: template.sms_from.ok_or_else(|| TardisError::conflict("template missing field sms_from", "409-reach-bad-template"))?,
            status_callback: None,
            extend: None,
            to,
            template_id: template.sms_template_id.ok_or_else(|| TardisError::conflict("template missing field template_id", "409-reach-bad-template"))?,
            template_paras: content.render_final_content::<20>(template.content),
            signature: template.sms_signature,
        };
        self.send_sms(request).await?;
        Ok(())
    }
}
