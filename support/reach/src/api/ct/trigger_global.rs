use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;

use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem::web::Json;

use tardis::web::poem_openapi;
use tardis::web::web_resp::{TardisApiResult, TardisResp, Void};

use crate::consts::get_tardis_inst;
use crate::dto::*;
use crate::serv::*;

#[derive(Clone, Default)]
/// 用户触达触发全局配置-租户控制台
pub struct ReachTriggerGlobalConfigCtApi;

#[poem_openapi::OpenApi(prefix_path = "/ct/msg/global/config")]
impl ReachTriggerGlobalConfigCtApi {
    /// 获取所有用户触达触发全局配置数据
    #[oai(method = "get", path = "/")]
    pub async fn find_trigger_global_config(&self, TardisContextExtractor(ctx): TardisContextExtractor) -> TardisApiResult<Vec<ReachTriggerGlobalConfigSummaryResp>> {
        let funs = get_tardis_inst();
        // filter
        let mut filter = ReachTriggerGlobalConfigFilterReq::default();
        filter.base_filter.basic.with_sub_own_paths = true;
        let resp = ReachTriggerGlobalConfigService::find_rbums(&filter, None, None, &funs, &ctx).await?;
        TardisResp::ok(resp)
    }

    /// 保存用户触达触发实例配置
    #[oai(method = "put", path = "/")]
    pub async fn add_or_modify_global_config(
        &self,
        json_body: Json<ReachTriggerGlobalConfigAddOrModifyAggReq>,
        TardisContextExtractor(ctx): TardisContextExtractor,
    ) -> TardisApiResult<Void> {
        let funs = get_tardis_inst();
        ReachTriggerGlobalConfigService::add_or_modify_global_config(json_body.0, &funs, &ctx).await?;
        TardisResp::ok(VOID)
    }
}
