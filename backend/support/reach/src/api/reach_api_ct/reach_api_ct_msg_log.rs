use crate::dto::*;
use crate::reach_consts::get_tardis_inst;
#[cfg(feature = "simple-client")]
use crate::reach_invoke::Client;
use crate::serv::*;
use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;
#[cfg(feature = "simple-client")]
use bios_sdk_invoke::simple_invoke_client;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::param::Query;
use tardis::web::web_resp::{TardisApiResult, TardisPage, TardisResp};
#[derive(Clone, Default)]
/// 消息记录-租户控制台
pub struct ReachMsgLogCtApi;
#[cfg_attr(feature = "simple-client", simple_invoke_client(Client<'_>))]
#[poem_openapi::OpenApi(prefix_path = "/ct/msg/log", tag = "bios_basic::ApiTag::App")]
impl ReachMsgLogCtApi {
    /// 获取所有消息记录数据
    #[oai(method = "get", path = "/all")]
    pub async fn find_msg_log(&self, reach_message_id: Query<String>, TardisContextExtractor(ctx): TardisContextExtractor) -> TardisApiResult<Vec<ReachMsgLogSummaryResp>> {
        let funs = get_tardis_inst();
        // filter
        let mut filter = ReachMsgLogFilterReq::default();
        filter.base_filter.basic.with_sub_own_paths = true;
        filter.rel_reach_message_id = Some(reach_message_id.0);
        let resp = ReachMessageLogServ::find_rbums(&filter, None, None, &funs, &ctx).await?;
        TardisResp::ok(resp)
    }

    /// 获取所有消息记录数据
    #[oai(method = "get", path = "/page")]
    pub async fn find_msg_log_paged(
        &self,
        page_number: Query<Option<u32>>,
        page_size: Query<Option<u32>>,
        reach_message_id: Query<Option<String>>,
        TardisContextExtractor(ctx): TardisContextExtractor,
    ) -> TardisApiResult<TardisPage<ReachMsgLogSummaryResp>> {
        let funs = get_tardis_inst();
        let page_number = page_number.unwrap_or(1);
        let page_size = page_size.unwrap_or(10);
        // filter
        let mut filter = ReachMsgLogFilterReq::default();
        filter.base_filter.basic.with_sub_own_paths = true;
        filter.rel_reach_message_id = reach_message_id.0;
        let page_resp = ReachMessageLogServ::paginate_rbums(&filter, page_number, page_size, Some(true), None, &funs, &ctx).await?;
        TardisResp::ok(page_resp)
    }
}
