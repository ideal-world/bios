use std::{
    collections::HashSet,
    sync::{Arc, OnceLock},
};

use bios_reach::{
    reach_send_channel::{GenericTemplate, SendChannel},
    reach_config::ReachConfig,
    reach_consts::MODULE_CODE,
    dto::{ContentReplace, ReachChannelKind},
};
use tardis::{
    async_trait::async_trait,
    basic::{error::TardisError, result::TardisResult},
    TardisFuns,
};

use crate::{SendSmsRequest, SmsClient, SmsContent};

#[async_trait]
impl SendChannel for crate::SmsClient {
    fn kind(&self) -> ReachChannelKind {
        ReachChannelKind::Sms
    }
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

impl crate::SmsClient {
    pub fn from_reach_config() -> Arc<Self> {
        static SMS_CLIENT: OnceLock<Arc<SmsClient>> = OnceLock::new();
        SMS_CLIENT
            .get_or_init(|| {
                // this would block thread but it's ok
                let config = TardisFuns::cs_config::<ReachConfig>(MODULE_CODE);
                let sms_config = &config.sms;
                let base_url = sms_config.base_url.parse().expect("invalid sms base url");
                let callback_url = sms_config.status_call_back.as_ref().map(|x| x.parse().expect("invalid sms status_call_back url"));
                SmsClient::new(base_url, &sms_config.app_key, &sms_config.app_secret, callback_url).into()
            })
            .clone()
    }
}
