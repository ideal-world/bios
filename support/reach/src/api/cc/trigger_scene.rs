use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;

use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem::web::Query;

use tardis::web::poem_openapi;
use tardis::web::web_resp::{TardisApiResult, TardisResp};

use crate::consts::get_tardis_inst;

use crate::dto::*;
use crate::serv::*;

#[derive(Clone, Default)]
pub struct ReachTriggerSceneCcApi;

#[poem_openapi::OpenApi(prefix_path = "/cc/trigger/scene")]
impl ReachTriggerSceneCcApi {
    /// 用户触达触发场景-公告`控制台
    #[oai(method = "get", path = "/")]
    pub async fn find_trigger_scene(&self, TardisContextExtractor(ctx): TardisContextExtractor) -> TardisApiResult<Vec<ReachTriggerSceneSummaryResp>> {
        let funs = get_tardis_inst();
        let mut filter = ReachTriggerSceneFilterReq::default();
        filter.base_filter.basic.with_sub_own_paths = true;
        filter.base_filter.basic.own_paths = Some(String::default());
        let resp = ReachTriggerSceneService::find_rbums(&filter, None, None, &funs, &ctx).await?;
        TardisResp::ok(resp)
    }

    /// 根据code获取相关用户触达触发场景
    #[oai(method = "get", path = "/code")]
    pub async fn find_trigger_scene_by_code(&self, code: Query<String>, TardisContextExtractor(ctx): TardisContextExtractor) -> TardisApiResult<Vec<ReachTriggerSceneSummaryResp>> {
        let funs = get_tardis_inst();
        let mut filter = ReachTriggerSceneFilterReq::default();
        filter.base_filter.basic.with_sub_own_paths = true;
        filter.base_filter.basic.own_paths = Some(String::default());
        filter.code = Some(code.0);
        let resp = ReachTriggerSceneService::find_rbums(&filter, None, None, &funs, &ctx).await?;
        TardisResp::ok(resp)
    }
}
