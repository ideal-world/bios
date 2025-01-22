use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi;
use tardis::web::web_resp::{TardisApiResult, TardisResp};

use crate::dto::event_dto::EventRegisterResp;

use crate::serv::event_register_serv;
#[derive(Clone)]
pub struct EventRegisterApi {
    pub(crate) register_serv: event_register_serv::EventRegisterServ,
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
        let resp = self.register_serv.register_ctx(&ctx.0).await?;
        tardis::tracing::debug!(?resp, "register event node");
        TardisResp::ok(resp)
    }
}
