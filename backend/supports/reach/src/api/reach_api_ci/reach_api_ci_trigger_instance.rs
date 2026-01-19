use bios_basic::rbum::dto::rbum_filer_dto::{RbumBasicFilterReq, RbumItemBasicFilterReq};
use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;

use tardis::web::context_extractor::TardisContextExtractor;

use tardis::web::poem_openapi;
use tardis::web::poem_openapi::param::Query;
use tardis::web::poem_openapi::payload::Json;
use tardis::web::web_resp::{TardisApiResult, TardisResp, Void};

use crate::dto::*;
use crate::reach_constants::get_tardis_inst;
use crate::serv::*;

#[cfg(feature = "simple-client")]
use crate::reach_invoke::Client;
#[derive(Clone, Default)]
pub struct ReachTriggerInstanceConfigCiApi;

/// Interface Console Reach Trigger Instance Config API
/// 接口控制台触达触发实例配置API
#[cfg_attr(feature = "simple-client", bios_sdk_invoke::simple_invoke_client(Client<'_>))]
#[poem_openapi::OpenApi(prefix_path = "/ci/trigger/instance/config", tag = "bios_basic::ApiTag::Interface")]
impl ReachTriggerInstanceConfigCiApi {
    /// Find all user reach trigger instance config data
    /// 根据类型获取所有用户触达触发实例配置数据
    #[oai(method = "get", path = "/")]
    pub async fn find_trigger_instance_config(
        &self,
        rel_item_id: Query<String>,
        channel: Query<ReachChannelKind>,
        scene_code: Query<Option<String>>,
        TardisContextExtractor(ctx): TardisContextExtractor,
    ) -> TardisApiResult<Vec<ReachTriggerInstanceConfigSummaryResp>> {
        let funs = get_tardis_inst();
        // filter
        let mut filter = ReachTriggerInstanceConfigFilterReq::default();
        filter.base_filter.basic.with_sub_own_paths = true;
        filter.base_filter.basic.own_paths = Some(String::default());
        filter.rel_item_id = Some(rel_item_id.0);
        filter.rel_reach_channel = Some(channel.0);
        if let Some(scene_code) = scene_code.0 {
            // 根据场景编码获取场景id
            let scene = ReachTriggerSceneService::find_one_rbum(&ReachTriggerSceneFilterReq {
                base_filter: RbumItemBasicFilterReq {
                    basic: RbumBasicFilterReq {
                        with_sub_own_paths: true,
                        own_paths: Some(String::default()),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                code: Some(scene_code),
                ..Default::default()
            }, &funs, &ctx).await?;
            if let Some(scene) = scene {
                filter.rel_reach_trigger_scene_id = Some(scene.id);
            }
        }
        let resp = ReachTriggerInstanceConfigService::find_rbums(&filter, None, None, &funs, &ctx).await?;
        TardisResp::ok(resp)
    }
}
