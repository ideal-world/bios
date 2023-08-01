use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem::web::Path;
use simple_invoke_client_macro::simple_invoke_client;
use tardis::web::poem_openapi;
use tardis::web::web_resp::{TardisApiResult, TardisResp};

use crate::consts::get_tardis_inst;
use crate::dto::*;
use crate::serv::*;

#[derive(Clone, Default)]
/// 消息记录-租户控制台
pub struct ReachMsgLogCtApi;
#[simple_invoke_client(Client, "/ct/msg/log")]
#[poem_openapi::OpenApi(prefix_path = "/ct/msg/log")]
impl ReachMsgLogCtApi {
    /// 获取所有消息记录数据
    #[oai(method = "get", path = "/:reach_message_id")]
    pub async fn find_msg_log(&self, reach_message_id: Path<String>, TardisContextExtractor(ctx): TardisContextExtractor) -> TardisApiResult<Vec<ReachMsgLogSummaryResp>> {
        let funs = get_tardis_inst();
        // filter
        let mut filter = ReachMsgLogFilterReq::default();
        filter.base_filter.basic.with_sub_own_paths = true;
        filter.rel_reach_message_id = Some(reach_message_id.0);
        let resp = ReachMessageLogServ::find_rbums(&filter, None, None, &funs, &ctx).await?;
        TardisResp::ok(resp)
    }
}
