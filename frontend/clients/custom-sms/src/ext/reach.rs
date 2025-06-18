use std::sync::{Arc, OnceLock};

use bios_reach::dto::ReachChannelKind;
use bios_reach::reach_config::ReachConfig;
use bios_reach::reach_send_channel::{GenericTemplate, SendChannel};
use tardis::basic::{error::TardisError, result::TardisResult};
use tardis::TardisFuns;

use crate::client::Client;
use crate::model::*;
#[derive(Debug, Clone)]
pub struct CustomSmsReachChannel {
    pub inner: Client,
    pub default_from: String,
    pub app_id: String,
}

impl Default for CustomSmsReachChannel {
    fn default() -> Self {
        let inst = bios_reach::reach_constants::get_tardis_inst();
        let config = inst.conf::<ReachConfig>();
        Self::from(config.as_ref())
    }
}

impl CustomSmsReachChannel {
    pub fn from_reach_config() -> Arc<Self> {
        static SMS_CLIENT: OnceLock<Arc<CustomSmsReachChannel>> = OnceLock::new();
        SMS_CLIENT
            .get_or_init(|| {
                // this would block thread but it's ok
                let config = TardisFuns::cs_config::<ReachConfig>(bios_reach::reach_constants::MODULE_CODE);
                let client = CustomSmsReachChannel::from(config.as_ref());
                Arc::new(client)
            })
            .clone()
    }
}

impl From<&ReachConfig> for CustomSmsReachChannel {
    fn from(config: &ReachConfig) -> Self {
        let url = config.sms.base_url.parse().expect("reach base url shall be valid");
        let app_id = config.sms.app_key.parse().expect("reach app id shall be valid");
        let app_pwd = config.sms.app_secret.parse().expect("reach app pwd shall be valid");
        let client = Client::init(url, app_id, app_pwd).expect("reach client shall be init");
        Self {
            inner: client,
            default_from: config.sms.sms_general_from.clone(),
            app_id: config.sms.app_key.clone(),
        }
    }
}

#[tardis::async_trait::async_trait]
impl SendChannel for CustomSmsReachChannel {
    fn kind(&self) -> ReachChannelKind {
        ReachChannelKind::Sms
    }
    async fn send(&self, template: GenericTemplate<'_>, content: &bios_reach::dto::ContentReplace, to: &std::collections::HashSet<&str>) -> TardisResult<()> {
        let content = content.render_final_content::<20>(template.content);
        let req = crate::model::SendCommonMessageBuilder::default()
            .app_id(&self.app_id)
            .recv_id(to.iter().map(|s| s.to_string()))
            .content(content)
            .ack(AckType::NoReply)
            .msg_type(MsgType::User)
            .build()
            .expect("invalid sms request");
        let resp = self.inner.send_message(&req).await?;
        if let Some(resp) = resp.data {
            if !resp.is_ok() {
                return Err(TardisError::internal_error(&resp.desc.unwrap_or_default(), "500-reach-send-failed"));
            }
        } else {
            return Err(TardisError::internal_error(
                &format!("custom-smsapi error, code={}, message={}", &resp.code, &resp.msg.unwrap_or_default()),
                "500-reach-send-failed",
            ));
        }
        Ok(())
    }
}
