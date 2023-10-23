use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;
use std::collections::{HashMap, HashSet};
use tardis::log as tracing;

use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi::param::{Path, Query};

use tardis::web::poem_openapi;
use tardis::web::poem_openapi::payload::Json;
use tardis::web::web_resp::{TardisApiResult, TardisResp, Void};

use crate::dto::*;
use crate::reach_config::ReachConfig;
use crate::reach_consts::*;
use crate::reach_init::REACH_SEND_CHANNEL_MAP;
#[cfg(feature = "simple-client")]
use crate::reach_invoke::Client;
use crate::reach_send_channel::{GenericTemplate, SendChannelMap};
use crate::serv::*;

#[derive(Clone)]
/// 用户触达消息-公共控制台
pub struct ReachMessageCcApi {
    channel: &'static SendChannelMap,
}

impl Default for ReachMessageCcApi {
    fn default() -> Self {
        Self {
            channel: REACH_SEND_CHANNEL_MAP.get().expect("missing send channel map"),
        }
    }
}

#[cfg_attr(feature = "simple-client", bios_sdk_invoke::simple_invoke_client(Client<'_>))]
#[poem_openapi::OpenApi(prefix_path = "/cc/msg", tag = "bios_basic::ApiTag::App")]
impl ReachMessageCcApi {
    /// 根据模板id发送信息
    #[oai(method = "put", path = "/general/:to/template/:template_id")]
    #[tardis::log::instrument(skip_all, fields(module = "reach"))]
    pub async fn general_send(
        &self,
        to: Path<String>,
        template_id: Path<String>,
        ctx: TardisContextExtractor,
        replacement: Json<HashMap<String, String>>,
    ) -> TardisApiResult<Void> {
        let ctx = ctx.0;
        let funs = get_tardis_inst();
        let msg_template = ReachMessageTemplateServ::get_by_id(&template_id, &funs, &ctx).await?;
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
                .ok_or_else(|| funs.err().internal_error("reach_message", "vcode_send", "no reach vcode strategy was found", "500-reach-missing-v-code-strategy"))?
        };
        let msg_template = {
            let req = ReachMessageTemplateFilterReq {
                rel_reach_channel: Some(ReachChannelKind::Sms),
                rel_reach_verify_code_strategy_id: Some(v_code_strategy.id),
                ..Default::default()
            };
            ReachMessageTemplateServ::find_one_rbum(&req, &funs, &ctx)
                .await?
                .ok_or_else(|| funs.err().internal_error("reach_message", "vcode_send", "corresponded template not found", "500-reach-missing-message-template"))?
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
        self.channel.send(ReachChannelKind::Sms, GenericTemplate::pwd_template(config), &[("pwd", code.0)].into(), &[to.0].into()).await?;
        TardisResp::ok(VOID)
    }

    /// 邮箱发送
    #[oai(method = "put", path = "/mail/:mail")]
    pub async fn mail_pwd_send(&self, mail: Path<String>, message: Query<String>, subject: Query<String>) -> TardisApiResult<Void> {
        self.channel
            .send(
                ReachChannelKind::Email,
                GenericTemplate {
                    name: Some(subject.as_ref()),
                    content: &message,
                    ..Default::default()
                },
                &Default::default(),
                &HashSet::from([mail.0]),
            )
            .await?;
        TardisResp::ok(VOID)
    }
}
