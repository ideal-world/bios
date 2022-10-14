use bios_chat_basic::chat_constants;
use bios_chat_basic::dto::chat_message_dto::{ChatMessageAddReq, ChatMessageInfoResp};
use tardis::chrono::{DateTime, Utc};
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem::web::websocket::{Message, WebSocket};
use tardis::web::poem::web::Data;
use tardis::web::poem::{handler, IntoResponse};
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::param::Query;
use tardis::web::poem_openapi::payload::Json;
use tardis::web::web_resp::{TardisApiResult, TardisPage, TardisResp};

pub struct ChatCcMessageApi;

/// Common Console Message API
#[poem_openapi::OpenApi(prefix_path = "/cc/message", tag = "bios_basic::ApiTag::Common")]
impl ChatCcMessageApi {
    /// Add Message
    #[oai(path = "/", method = "post")]
    async fn add_message(&self, add_req: Json<ChatMessageAddReq>, ctx: TardisContextExtractor) -> TardisApiResult<String> {
        let mut funs = chat_constants::get_tardis_inst();
        funs.begin().await?;
        // TODO
        funs.commit().await?;
        TardisResp::ok("".to_string())
    }

    /// Find Message
    #[oai(path = "/", method = "get")]
    async fn paginate_message(
        &self,
        to_id: Query<String>,
        start_time: Query<DateTime<Utc>>,
        end_time: Query<DateTime<Utc>>,
        page_number: Query<u64>,
        page_size: Query<u64>,
        ctx: TardisContextExtractor,
    ) -> TardisApiResult<TardisPage<ChatMessageInfoResp>> {
        let funs = chat_constants::get_tardis_inst();
        // TODO
        TardisResp::ok(TardisPage {
            page_size: todo!(),
            page_number: todo!(),
            total_size: todo!(),
            records: todo!(),
        })
    }
}
