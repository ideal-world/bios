use std::collections::{HashMap, HashSet};

use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;

use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem::web::{Json, Path, Query};

use tardis::web::poem_openapi;
use tardis::web::web_resp::{TardisApiResult, TardisResp, Void};

use crate::client::{sms, SendChannelAll, GenericTemplate};
use crate::config::ReachConfig;
use crate::consts::*;
use crate::dto::*;
use crate::serv::*;

#[derive(Clone, Default)]
/// 用户触达消息-公共控制台
pub struct ReachMessageCcApi {
    channel: SendChannelAll,
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
        self.channel.send(msg_template.rel_reach_channel, &msg_template, &replacement.0.into(), &HashSet::from([to.0])).await?;
        TardisResp::ok(VOID)
    }

    /// 验证码发送
    #[oai(method = "put", path = "/vcode/:to/:code")]
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
        self.channel.send(msg_template.rel_reach_channel, &msg_template, &content_replace, &HashSet::from([to.0])).await?;
        TardisResp::ok(VOID)
    }

    /// 密码发送
    #[oai(method = "put", path = "/pwd/:to/:code")]
    pub async fn pwd_send(&self, to: Path<String>, code: Path<String>) -> TardisApiResult<Void> {
        let funs = get_tardis_inst();
        let config = funs.conf::<ReachConfig>();
        let sms_cfg = &config.sms;
        self.channel
            .sms_client
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

    /// 邮箱发送
    #[oai(method = "put", path = "/mail/:mail")]
    pub async fn mail_pwd_send(&self, mail: Path<String>, message: Query<String>, subject: Query<String>) -> TardisApiResult<Void> {
        self.channel.send(ReachChannelKind::Email, GenericTemplate {
            name: Some(subject.as_ref()),
            content: &message,
            ..Default::default()
        }, &Default::default(), &HashSet::from([mail.0])).await?;
        TardisResp::ok(VOID)
    }
}
