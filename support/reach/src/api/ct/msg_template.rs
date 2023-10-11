use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;

use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi::param::{Path, Query};
use tardis::web::poem_openapi::payload::Json;

use tardis::web::poem_openapi;
use tardis::web::web_resp::{TardisApiResult, TardisPage, TardisResp};

use crate::consts::get_tardis_inst;
use crate::dto::*;
use crate::serv::*;

#[cfg(feature = "simple-client")]
use crate::invoke::Client;

use super::map_notfound_to_false;
#[derive(Clone, Default)]
/// 用户触达消息消息模板-租户控制台
pub struct ReachMessageTemplateCtApi;
#[cfg_attr(feature = "simple-client", bios_sdk_invoke::simple_invoke_client(Client<'_>))]
#[poem_openapi::OpenApi(prefix_path = "/ct/msg/template", tag = "bios_basic::ApiTag::App")]
impl ReachMessageTemplateCtApi {
    /// 获取所有用户触达消息消息模板数据分页
    #[oai(method = "get", path = "/page")]
    pub async fn paginate_msg_template(
        &self,
        page_number: Query<Option<u32>>,
        page_size: Query<Option<u32>>,
        rel_reach_channel: Query<Option<String>>,
        TardisContextExtractor(ctx): TardisContextExtractor,
    ) -> TardisApiResult<TardisPage<ReachMessageTemplateSummaryResp>> {
        let page_number = page_number.unwrap_or(1);
        let page_size = page_size.unwrap_or(10);
        let rel_reach_channel = rel_reach_channel.0.map(|x| x.parse::<ReachChannelKind>()).transpose()?;
        let funs = get_tardis_inst();
        // filter
        let mut filter = ReachMessageTemplateFilterReq::default();
        filter.base_filter.with_sub_own_paths = true;
        filter.rel_reach_channel = rel_reach_channel;
        let page_resp = ReachMessageTemplateServ::paginate_rbums(&filter, page_number, page_size, None, None, &funs, &ctx).await?;
        TardisResp::ok(page_resp)
    }

    /// 获取所有用户触达消息消息模板数据
    #[oai(method = "get", path = "/")]
    pub async fn find_msg_template(
        &self,
        rel_reach_channel: Query<Option<String>>,
        TardisContextExtractor(ctx): TardisContextExtractor,
    ) -> TardisApiResult<Vec<ReachMessageTemplateSummaryResp>> {
        let funs = get_tardis_inst();
        // filter
        let mut filter = ReachMessageTemplateFilterReq::default();
        let rel_reach_channel = rel_reach_channel.0.map(|x| x.parse::<ReachChannelKind>()).transpose()?;
        filter.base_filter.with_sub_own_paths = true;
        filter.rel_reach_channel = rel_reach_channel;
        let resp = ReachMessageTemplateServ::find_rbums(&filter, None, None, &funs, &ctx).await?;
        TardisResp::ok(resp)
    }

    /// 根据Id获取用户触达消息消息模板数据
    #[oai(method = "get", path = "/:id")]
    pub async fn get_msg_template_by_id(&self, id: Path<String>, TardisContextExtractor(ctx): TardisContextExtractor) -> TardisApiResult<ReachMessageTemplateDetailResp> {
        let funs = get_tardis_inst();
        // filter
        let mut filter = ReachMessageTemplateFilterReq::default();
        filter.base_filter.with_sub_own_paths = true;
        let resp = ReachMessageTemplateServ::get_rbum(&id, &filter, &funs, &ctx).await?;
        TardisResp::ok(resp)
    }

    /// 添加用户触达消息消息模板
    #[oai(method = "post", path = "/")]
    pub async fn add_msg_template(&self, mut agg_req: Json<ReachMessageTemplateAddReq>, TardisContextExtractor(ctx): TardisContextExtractor) -> TardisApiResult<String> {
        let funs = get_tardis_inst();
        let id = ReachMessageTemplateServ::add_rbum(&mut agg_req, &funs, &ctx).await?;
        TardisResp::ok(id)
    }

    /// 编辑用户触达消息消息模板
    #[oai(method = "put", path = "/:id")]
    pub async fn modify_msg_template(
        &self,
        id: Path<String>,
        mut mod_req: Json<ReachMessageTemplateModifyReq>,
        TardisContextExtractor(ctx): TardisContextExtractor,
    ) -> TardisApiResult<String> {
        let funs = get_tardis_inst();
        ReachMessageTemplateServ::modify_rbum(&id, &mut mod_req, &funs, &ctx).await?;
        TardisResp::ok(id.0)
    }

    /// 删除用户触达消息消息模板
    #[oai(method = "delete", path = "/:id")]
    pub async fn delete_msg_template(&self, id: Path<String>, TardisContextExtractor(ctx): TardisContextExtractor) -> TardisApiResult<bool> {
        let funs = get_tardis_inst();
        let ok = ReachMessageTemplateServ::delete_rbum(&id, &funs, &ctx).await.map_or_else(map_notfound_to_false, |count| Ok(count != 0))?;
        TardisResp::ok(ok)
    }
}
