use std::cell::OnceCell;

use tardis::{
    async_trait::async_trait,
    basic::{error::TardisError, result::TardisResult},
    mail::mail_client::TardisMailSendReq,
};

use crate::dto::*;

use self::{email::MailClient, sms::SendSmsRequest};

pub mod email;
pub mod sms;
#[async_trait]
pub trait SendChannel {
    async fn send(&self, template: &ReachMessageTemplateDetailResp, content: &ContentReplace, to: &str) -> TardisResult<()>;
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
    const_inst! {Sms,Email,Inbox,Wechat,DingTalk,Push,WebHook}
}
#[async_trait]
impl SendChannel for UnimplementedChannel {
    async fn send(&self, template: &ReachMessageTemplateDetailResp, content: &ContentReplace, to: &str) -> TardisResult<()> {
        let message = format!("trying to send through an unimplemented channel [{kind}]", kind = self.0);
        Err(TardisError::conflict(&message, "500-unimplemented-channel"))
    }
}

#[async_trait]
impl SendChannel for email::MailClient {
    async fn send(&self, template: &ReachMessageTemplateDetailResp, content: &ContentReplace, to: &str) -> TardisResult<()> {
        self.inner
            .send(&TardisMailSendReq {
                subject: template.name.clone().ok_or_else(|| bad_template("template missing field sms_from"))?,
                txt_body: content.render_final_content::<{ usize::MAX }>(&template.content),
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
    async fn send(&self, template: &ReachMessageTemplateDetailResp, content: &ContentReplace, to: &str) -> TardisResult<()> {
        let request = SendSmsRequest {
            from: template.sms_from.as_deref().ok_or_else(|| TardisError::conflict("template missing field sms_from", "409-reach-bad-template"))?,
            status_callback: None,
            extend: None,
            to,
            template_id: template.sms_template_id.as_deref().ok_or_else(|| TardisError::conflict("template missing field template_id", "409-reach-bad-template"))?,
            template_paras: content.render_final_content::<20>(&template.content),
            signature: template.sms_signature.as_deref(),
        };
        self.send_sms(request).await?;
        Ok(())
    }
}
