use bios_basic::rbum::helper::rbum_scope_helper::check_without_owner_and_unsafe_fill_ctx;
use tardis::log as tracing;

use tardis::web::context_extractor::TardisContextExtractor;

use tardis::web::poem::Request;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::payload::Json;
use tardis::web::web_resp::{TardisApiResult, TardisResp, Void};
use std::collections::HashSet;

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
    pub async fn message_send(&self, body: Json<ReachMsgSendReq>, request: &Request, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut ctx = ctx.0;
        let funs = get_tardis_inst();
        let body = body.0;
        check_without_owner_and_unsafe_fill_ctx(request, &funs, &mut ctx)?;
        message_send(body, &funs, &ctx).await?;
        TardisResp::ok(VOID)
    }

    /// Batch send messages by template id
    /// 根据模板id批量发送信息
    #[oai(method = "put", path = "/batch/send")]
    #[tardis::log::instrument(skip_all, fields(module = "reach"))]
    pub async fn message_batch_send(&self, body: Json<Vec<ReachMsgSendReq>>, request: &Request, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut ctx = ctx.0;
        let funs = get_tardis_inst();
        let mut body = body.0;
        check_without_owner_and_unsafe_fill_ctx(request, &funs, &mut ctx)?;
        
        deduplicate_send_requests(&mut body);
        
        for send_req in body {
            message_send(send_req, &funs, &ctx).await?;
        }
        TardisResp::ok(VOID)
    }
}
