use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;

use tardis::web::context_extractor::TardisContextExtractor;

use tardis::web::poem_openapi;
use tardis::web::poem_openapi::param::{Path, Query};
use tardis::web::poem_openapi::payload::Json;
use tardis::web::web_resp::{TardisApiResult, TardisPage, TardisResp};

use crate::dto::*;
use crate::reach_consts::get_tardis_inst;
#[cfg(feature = "simple-client")]
use crate::reach_invoke::Client;
use crate::serv::*;
#[derive(Clone, Default)]
/// 用户触达消息-公共控制台
pub struct ReachMessageCtApi;

#[cfg_attr(feature = "simple-client", bios_sdk_invoke::simple_invoke_client(Client<'_>))]
#[poem_openapi::OpenApi(prefix_path = "/ct/msg", tag = "bios_basic::ApiTag::App")]
impl ReachMessageCtApi {
    /// 获取所有用户触达消息数据分页
    #[oai(method = "get", path = "/page")]
    pub async fn paginate_msg(
        &self,
        page_number: Query<Option<u32>>,
        page_size: Query<Option<u32>>,
        TardisContextExtractor(ctx): TardisContextExtractor,
    ) -> TardisApiResult<TardisPage<ReachMessageSummaryResp>> {
        let page_number = page_number.unwrap_or(1);
        let page_size = page_size.unwrap_or(10);
        let funs = get_tardis_inst();
        // filter
        let mut filter = ReachMessageFilterReq::default();
        filter.rbum_item_basic_filter_req.basic.with_sub_own_paths = true;
        let page_resp = ReachMessageServ::paginate_rbums(&filter, page_number, page_size, Some(true), None, &funs, &ctx).await?;
        TardisResp::ok(page_resp)
    }

    /// 获取所有用户触达消息数据
    #[oai(method = "get", path = "/")]
    pub async fn find_msg(&self, TardisContextExtractor(ctx): TardisContextExtractor) -> TardisApiResult<Vec<ReachMessageSummaryResp>> {
        let mut filter = ReachMessageFilterReq::default();
        filter.rbum_item_basic_filter_req.basic.with_sub_own_paths = true;
        let funs = get_tardis_inst();
        let resp = ReachMessageServ::find_rbums(&filter, None, None, &funs, &ctx).await?;
        TardisResp::ok(resp)
    }

    /// 根据Id获取用户触达消息数据
    #[oai(method = "get", path = "/:id")]
    pub async fn get_msg_by_id(&self, id: Path<String>, TardisContextExtractor(ctx): TardisContextExtractor) -> TardisApiResult<ReachMessageDetailResp> {
        let mut filter = ReachMessageFilterReq::default();
        filter.rbum_item_basic_filter_req.basic.with_sub_own_paths = true;
        let funs = get_tardis_inst();
        let resp = ReachMessageServ::get_rbum(&id, &filter, &funs, &ctx).await?;
        TardisResp::ok(resp)
    }

    /// 重新发送消息（仅仅适用于状态为Fail的消息）
    #[oai(method = "put", path = "/resend/:id")]
    pub async fn resend(&self, id: Path<String>, TardisContextExtractor(_ctx): TardisContextExtractor) -> TardisApiResult<bool> {
        let funs = get_tardis_inst();
        // let resp = ReachMessageServ::get_rbum(&id, &Default::default(), &funs, &ctx).await?;
        let success = ReachMessageServ::resend(&id, &funs).await?;
        TardisResp::ok(success)
    }

    /// 添加消息
    #[oai(method = "post", path = "/")]
    pub async fn add_message(&self, mut add_req: Json<ReachMessageAddReq>, TardisContextExtractor(ctx): TardisContextExtractor) -> TardisApiResult<String> {
        let funs = get_tardis_inst();
        let id = ReachMessageServ::add_rbum(&mut add_req.0, &funs, &ctx).await?;
        TardisResp::ok(id)
    }
}
