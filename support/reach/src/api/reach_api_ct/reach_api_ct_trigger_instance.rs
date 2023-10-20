use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;

use tardis::web::context_extractor::TardisContextExtractor;

use tardis::web::poem_openapi;
use tardis::web::poem_openapi::param::Query;
use tardis::web::poem_openapi::payload::Json;
use tardis::web::web_resp::{TardisApiResult, TardisResp, Void};

use crate::reach_consts::get_tardis_inst;
use crate::dto::*;
use crate::serv::*;

#[cfg(feature = "simple-client")]
use crate::reach_invoke::Client;
#[derive(Clone, Default)]
/// 用户触达触发实例配置-租户控制台
pub struct ReachTriggerInstanceConfigCtApi;

#[cfg_attr(feature = "simple-client", bios_sdk_invoke::simple_invoke_client(Client<'_>))]
#[poem_openapi::OpenApi(prefix_path = "/ct/msg/instance/config", tag = "bios_basic::ApiTag::App")]
impl ReachTriggerInstanceConfigCtApi {
    /// 根据类型获取所有用户触达触发实例配置数据
    #[oai(method = "get", path = "/")]
    pub async fn find_trigger_instance_config(
        &self,
        rel_item_id: Query<String>,
        TardisContextExtractor(ctx): TardisContextExtractor,
    ) -> TardisApiResult<Vec<ReachTriggerInstanceConfigSummaryResp>> {
        let funs = get_tardis_inst();
        // filter
        let mut filter = ReachTriggerInstanceConfigFilterReq::default();
        filter.base_filter.basic.with_sub_own_paths = true;
        filter.rel_item_id = Some(rel_item_id.0);
        let resp = ReachTriggerInstanceConfigService::find_rbums(&filter, None, None, &funs, &ctx).await?;
        TardisResp::ok(resp)
    }

    /// 保存用户触达触发实例配置
    #[oai(method = "put", path = "/")]
    pub async fn add_or_modify_instance_config(
        &self,
        json_body: Json<ReachTriggerInstanceConfigAddOrModifyAggReq>,
        TardisContextExtractor(ctx): TardisContextExtractor,
    ) -> TardisApiResult<Void> {
        let funs = get_tardis_inst();
        ReachTriggerInstanceConfigService::add_or_modify_instance_config(json_body.0, &funs, &ctx).await?;
        TardisResp::ok(VOID)
    }
}
