use std::collections::HashMap;

use bios_basic::rbum::serv::rbum_crud_serv::{RbumCrudOperation, RbumCrudQueryPackage};
use tardis::basic::dto::TardisContext;
use tardis::log::trace;
use tardis::serde_json::Value;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem::web::{Json, Path};
use tardis::web::poem::Request;
use tardis::web::poem_openapi;
use tardis::web::web_resp::{TardisApiResult, TardisResp};

use crate::client::sms::SmsContent;
use crate::client::{email, sms, SendChannel, UnimplementedChannel};
use crate::consts::get_tardis_inst;
use crate::dto::*;
use crate::serv::*;
#[derive(Clone)]
/// 用户触达消息-公共控制台
pub struct ReachMessageCcApi {
    sms_client: sms::SmsClient,
    email_client: email::MailClient,
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
    ) -> TardisApiResult<String> {
        let funs = get_tardis_inst();
        let msg_template = ReachMessageTemplateServ::get_by_id(&msg_template_id, &funs, &ctx).await?;
        self.get_channel(msg_template.rel_reach_channel).send(&msg_template, &replacement.0.into(), &to).await?;
        TardisResp::ok(String::new())
    }

    /// 验证码发送
    #[oai(method = "put", path = "/general/:to/:code")]
    pub async fn vcode_send(
        &self,
        to: Path<String>,
        code: Path<String>,
        TardisContextExtractor(ctx): TardisContextExtractor,
    ) -> TardisApiResult<String> {
        let funs = get_tardis_inst();
        let msg_template: ReachMessageTemplateDetailResp = todo!();
        let content_replace = ([("code", code.0)]).into();
        self.get_channel(msg_template.rel_reach_channel).send(&msg_template, &content_replace, &to).await?;
        TardisResp::ok(String::new())
    }
}
