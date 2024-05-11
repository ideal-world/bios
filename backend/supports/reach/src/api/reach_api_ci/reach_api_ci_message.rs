use tardis::log as tracing;

use tardis::web::context_extractor::TardisContextExtractor;

use tardis::web::poem_openapi;
use tardis::web::poem_openapi::payload::Json;
use tardis::web::web_resp::{TardisApiResult, TardisResp, Void};

use crate::dto::*;
use crate::reach_constants::*;
#[cfg(feature = "simple-client")]
use crate::reach_invoke::Client;
use crate::serv::*;

#[derive(Debug, Default, Clone)]
pub struct ReachMessageCiApi;

/// Interface Console Reach Message API
/// 接口控制台触达消息API
#[cfg_attr(feature = "simple-client", bios_sdk_invoke::simple_invoke_client(Client<'_>))]
#[poem_openapi::OpenApi(prefix_path = "/ci/message", tag = "bios_basic::ApiTag::Interface")]
impl ReachMessageCiApi {
    /// Send message by template id
    /// 根据模板id发送信息
    #[oai(method = "put", path = "/send")]
    #[tardis::log::instrument(skip_all, fields(module = "reach"))]
    pub async fn message_send(&self, body: Json<ReachMsgSendReq>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let ctx = ctx.0;
        let funs = get_tardis_inst();
        let body = body.0;
        message_send(body, &funs, &ctx).await?;
        TardisResp::ok(VOID)
    }
}
