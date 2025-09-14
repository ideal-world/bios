use std::{
    collections::HashSet,
    sync::{Arc, OnceLock},
};

use bios_reach::{
    dto::{ContentReplace, ReachChannelKind},
    reach_config::ReachConfig,
    reach_constants::MODULE_CODE,
    reach_send_channel::{GenericTemplate, SendChannel},
};
use tardis::{
    async_trait::async_trait,
    basic::{error::TardisError, result::TardisResult},
    serde_json, TardisFuns,
};

use crate::{SendSmsRequest, SmsClient, SmsContent};

#[async_trait]
impl SendChannel for crate::SmsClient {
    fn kind(&self) -> ReachChannelKind {
        ReachChannelKind::Sms
    }
    async fn send(&self, template: GenericTemplate<'_>, content: &ContentReplace, to: &HashSet<&str>) -> TardisResult<()> {
        tardis::log::trace!("send sms {content}");
        let sms_content = SmsContent {
            template_code: template.sms_template_id.ok_or_else(|| TardisError::conflict("template missing field template_id", "409-reach-bad-template"))?,
            template_param: (*content).clone(),
            sign_name: template.sms_signature.unwrap_or_default(),
        };
        let from = template.sms_from.ok_or_else(|| TardisError::conflict("template missing field sms_from", "409-reach-bad-template"))?;
        let request = SendSmsRequest::new(from, sms_content);
        let resp = self.send_sms(request).await?;
        if resp.is_error() {
            use std::fmt::Write;
            let mut error_buffer = String::new();
            writeln!(&mut error_buffer, "send sms error [{code}]: {message}.", code = resp.code, message = resp.message).expect("write to string shouldn't fail");
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
                SmsClient::new(base_url, &sms_config.app_key, &sms_config.app_secret).into()
            })
            .clone()
    }
}
