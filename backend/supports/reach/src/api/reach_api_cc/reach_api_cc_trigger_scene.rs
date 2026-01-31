use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;

use tardis::web::context_extractor::TardisContextExtractor;

use tardis::web::poem_openapi;
use tardis::web::poem_openapi::param::Query;
use tardis::web::web_resp::{TardisApiResult, TardisResp};

use crate::dto::*;
use crate::reach_constants::get_tardis_inst;
#[cfg(feature = "simple-client")]
use crate::reach_invoke::Client;
use crate::serv::*;

#[derive(Clone, Default)]
pub struct ReachTriggerSceneCcApi;

/// Common Console Reach Trigger Scene API
/// 通用控制台触达触发场景API
#[cfg_attr(feature = "simple-client", bios_sdk_invoke::simple_invoke_client(Client<'_>))]
#[poem_openapi::OpenApi(prefix_path = "/cc/trigger/scene", tag = "bios_basic::ApiTag::Common")]
impl ReachTriggerSceneCcApi {
    /// Find trigger scene
    /// 用户触达触发场景-公告`控制台
    #[oai(method = "get", path = "/")]
    pub async fn find_trigger_scene(&self, TardisContextExtractor(ctx): TardisContextExtractor) -> TardisApiResult<Vec<ReachTriggerSceneSummaryResp>> {
        let funs = get_tardis_inst();
        let mut filter = ReachTriggerSceneFilterReq::default();
        filter.base_filter.basic.with_sub_own_paths = true;
        filter.base_filter.basic.own_paths = Some(String::default());
        filter.sort_asc = Some(true);
        let resp = ReachTriggerSceneService::find_rbums(&filter, None, None, &funs, &ctx).await?;
        TardisResp::ok(resp)
    }

    /// Find trigger scene by code
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

    /// Find all trigger scenes with global instances
    /// 获取所有场景及其全局配置实例
    #[oai(method = "get", path = "/with_global_instances")]
    pub async fn find_trigger_scenes_with_global_instances(&self, kind: Query<Option<ReachTriggerSceneKind>>, TardisContextExtractor(ctx): TardisContextExtractor) -> TardisApiResult<Vec<ReachTriggerSceneWithGlobalInstancesResp>> {
        let funs = get_tardis_inst();
        // 获取所有场景
        let mut filter = ReachTriggerSceneFilterReq::default();
        filter.base_filter.basic.with_sub_own_paths = true;
        filter.base_filter.basic.own_paths = Some(String::default());
        filter.sort_asc = Some(true);
        filter.kinds = if let Some(kind) = kind.0 {
            Some(vec![kind, ReachTriggerSceneKind::All])
        } else {
            Some(vec![ReachTriggerSceneKind::All])
        };
        let scenes = ReachTriggerSceneService::find_rbums(&filter, None, None, &funs, &ctx).await?;
        
        // 为每个场景获取其 global_instances
        let mut result = Vec::new();
        for scene in scenes {
            let mut global_filter = ReachTriggerGlobalConfigFilterReq::default();
            global_filter.base_filter.basic.with_sub_own_paths = true;
            global_filter.base_filter.basic.own_paths = Some(String::default());
            global_filter.rel_reach_trigger_scene_id = Some(scene.id.clone());
            let global_instances = ReachTriggerGlobalConfigService::find_rbums(&global_filter, None, None, &funs, &ctx).await?;
            
            result.push(ReachTriggerSceneWithGlobalInstancesResp {
                scene,
                global_instances,
            });
        }
        
        TardisResp::ok(result)
    }
}
