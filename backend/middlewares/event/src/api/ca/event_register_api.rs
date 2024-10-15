use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi;
use tardis::web::web_resp::{TardisApiResult, TardisResp};

use crate::dto::event_dto::EventRegisterResp;

use crate::serv::event_register_serv;
#[derive(Clone, Default, Debug)]
pub struct EventRegisterApi {
    register_serv: event_register_serv::EventRegisterServ,
}

/// Event Node Register API
///
/// 事件注册节点API
#[poem_openapi::OpenApi(prefix_path = "/ca/register")]
impl EventRegisterApi {
    /// Register event node
    ///
    /// 注册事件监听器
    #[oai(path = "/", method = "put")]
    async fn register(&self, ctx: TardisContextExtractor) -> TardisApiResult<EventRegisterResp> {
        let node_id = self.register_serv.register_ctx(&ctx.0).await?;
        let resp = EventRegisterResp { node_id: node_id.to_base64() };
        TardisResp::ok(resp)
    }
}
