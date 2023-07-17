use std::collections::HashMap;

use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;

use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem::web::{Json, Path};

use tardis::web::poem_openapi;
use tardis::web::web_resp::{TardisApiResult, TardisResp, Void};
use tardis::TardisFuns;

use crate::client::email::MailClient;
use crate::client::sms::SmsClient;
use crate::client::{email, sms, SendChannel, UnimplementedChannel};
use crate::config::ReachConfig;
use crate::consts::get_tardis_inst;
use crate::consts::DOMAIN_CODE;
use crate::dto::*;
use crate::serv::*;

#[derive(Clone)]
/// 用户触达消息-公共控制台
pub struct ReachMessageCcApi {
    sms_client: sms::SmsClient,
    email_client: email::MailClient,
}

impl Default for ReachMessageCcApi {
    fn default() -> Self {
        let config = TardisFuns::cs_config::<ReachConfig>(DOMAIN_CODE);
        let sms_config = &config.sms;
        let base_url = sms_config.base_url.parse().expect("invalid sms base url");
        let callback_url = sms_config.status_call_back.as_ref().map(|x| x.parse().expect("invalid sms status_call_back url"));
        Self {
            sms_client: SmsClient::new(base_url, &sms_config.app_key, &sms_config.app_secret, callback_url),
            email_client: MailClient::new(),
        }
    }
}

impl ReachMessageCcApi {
    pub fn get_channel(&self, kind: ReachChannelKind) -> &(dyn SendChannel + Send + Sync) {
        match kind {
            ReachChannelKind::Sms => &self.sms_client,
            ReachChannelKind::Email => &self.email_client,
            _ => UnimplementedChannel::get_const_ref(kind),
        }
    }
}
#[poem_openapi::OpenApi(prefix_path = "/cc/msg")]
impl ReachMessageCcApi {
    /// 根据模板id发送信息
    #[oai(method = "put", path = "/general/:to/:msg_template_id")]
    pub async fn general_send(
        &self,
        to: Path<String>,
        msg_template_id: Path<String>,
        replacement: Json<HashMap<String, String>>,
        TardisContextExtractor(ctx): TardisContextExtractor,
    ) -> TardisApiResult<Void> {
        let funs = get_tardis_inst();
        let msg_template = ReachMessageTemplateServ::get_by_id(&msg_template_id, &funs, &ctx).await?;
        self.get_channel(msg_template.rel_reach_channel).send((&msg_template).into(), &replacement.0.into(), &to).await?;
        TardisResp::ok(VOID)
    }

    /// 验证码发送
    #[oai(method = "put", path = "/general/:to/:code")]
    pub async fn vcode_send(&self, to: Path<String>, code: Path<String>, TardisContextExtractor(ctx): TardisContextExtractor) -> TardisApiResult<Void> {
        let funs = get_tardis_inst();
        let v_code_strategy = {
            let req = ReachVCodeStrategyFilterReq::default();
            VcodeStrategeServ::find_one_rbum(&req, &funs, &ctx)
                .await?
                .ok_or_else(|| funs.err().internal_error("reach_message", "vcode_send", "msg", "500-reach-missing-v-code-strategy"))?
        };
        let msg_template = {
            let req = ReachMessageTemplateFilterReq {
                rel_reach_channel: Some(ReachChannelKind::Sms),
                rel_reach_verify_code_strategy_id: Some(v_code_strategy.id),
                ..Default::default()
            };
            ReachMessageTemplateServ::find_one_rbum(&req, &funs, &ctx)
                .await?
                .ok_or_else(|| funs.err().internal_error("reach_message", "vcode_send", "msg", "500-reach-missing-message-template"))?
        };
        let content_replace = ([("code", code.0)]).into();
        self.get_channel(msg_template.rel_reach_channel).send((&msg_template).into(), &content_replace, &to).await?;
        TardisResp::ok(VOID)
    }

    /// 密码发送
    #[oai(method = "put", path = "/general/:to/:code")]
    pub async fn pwd_send(&self, to: Path<String>, code: Path<String>) -> TardisApiResult<Void> {
        let funs = get_tardis_inst();
        let config = funs.conf::<ReachConfig>();
        let sms_cfg = &config.sms;
        self.sms_client
            .send_sms(sms::SendSmsRequest {
                from: &sms_cfg.sms_general_from,
                status_callback: sms_cfg.status_call_back.as_deref(),
                extend: None,
                to: to.as_str(),
                template_id: &sms_cfg.sms_pwd_template_id,
                template_paras: format!("[{pwd}]", pwd = code.0),
                signature: sms_cfg.sms_general_signature.as_deref(),
            })
            .await?;
        TardisResp::ok(VOID)
    }
}
