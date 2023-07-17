use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;

use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem::web::{Json, Path, Query};

use tardis::web::poem_openapi;
use tardis::web::web_resp::{TardisApiResult, TardisPage, TardisResp};

use crate::consts::get_tardis_inst;
use crate::dto::*;
use crate::serv::*;

#[derive(Clone, Default)]
/// 用户触达消息-公共控制台
pub struct ReachMessageCtApi;

#[poem_openapi::OpenApi(prefix_path = "/ct/msg")]
impl ReachMessageCtApi {
    /// 获取所有用户触达消息数据分页
    #[oai(method = "get", path = "/page")]
    pub async fn paginate_msg_log(
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
    pub async fn find_msg_log(&self, TardisContextExtractor(ctx): TardisContextExtractor) -> TardisApiResult<Vec<ReachMessageSummaryResp>> {
        let mut filter = ReachMessageFilterReq::default();
        filter.rbum_item_basic_filter_req.basic.with_sub_own_paths = true;
        let funs = get_tardis_inst();
        let resp = ReachMessageServ::find_rbums(&filter, None, None, &funs, &ctx).await?;
        TardisResp::ok(resp)
    }

    /// 根据Id获取用户触达消息数据
    #[oai(method = "get", path = "/:id")]
    pub async fn get_msg_signature_by_id(&self, id: Path<String>, TardisContextExtractor(ctx): TardisContextExtractor) -> TardisApiResult<ReachMessageDetailResp> {
        let mut filter = ReachMessageFilterReq::default();
        filter.rbum_item_basic_filter_req.basic.with_sub_own_paths = true;
        let funs = get_tardis_inst();
        let resp = ReachMessageServ::get_rbum(&id, &filter, &funs, &ctx).await?;
        TardisResp::ok(resp)
    }

    /// 根据Id获取用户触达消息数据
    #[oai(method = "put", path = "/resend/:id")]
    pub async fn resend(&self, id: Path<String>, TardisContextExtractor(ctx): TardisContextExtractor) -> TardisApiResult<String> {
        let funs = get_tardis_inst();
        ReachMessageServ::resend(&id, &funs, &ctx).await?;
        TardisResp::ok(id.to_string())
    }

    /// 根据Id获取用户触达消息数据
    #[oai(method = "post", path = "/")]
    pub async fn add_message(&self, mut add_req: Json<ReachMessageAddReq>, TardisContextExtractor(ctx): TardisContextExtractor) -> TardisApiResult<String> {
        let funs = get_tardis_inst();
        let id = ReachMessageServ::add_rbum(&mut add_req.0, &funs, &ctx).await?;
        TardisResp::ok(id)
    }
}
