use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::param::Query;
use tardis::web::web_resp::{TardisApiResult, TardisResp};

use crate::event_constants::get_tardis_inst;
use crate::serv::event_message_serv::EventMessageServ;
#[derive(Clone)]
pub struct EventMessageApi;

/// Event Topic API
///
/// 事件主题API
#[poem_openapi::OpenApi(prefix_path = "/ci/message")]
impl EventMessageApi {
    /// Add Event Definition
    ///
    /// 添加事件主题
    #[oai(path = "/clear_archived", method = "delete")]
    async fn clear_archived(&self, topic_code: Query<Option<String>>, _ctx: TardisContextExtractor) -> TardisApiResult<u32> {
        let funs = get_tardis_inst();
        let count = EventMessageServ.clear_archived(topic_code.0.as_deref(), &funs).await?;
        TardisResp::ok(count)
    }
}
