use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::param::{Path, Query};
use tardis::web::poem_openapi::payload::Json;
use tardis::web::web_resp::{TardisApiResult, TardisResp, Void};

use crate::dto::*;
use crate::reach_consts::get_tardis_inst;
use crate::serv::*;

#[cfg(feature = "simple-client")]
use crate::reach_invoke::Client;
#[derive(Clone, Default)]
/// 用户触达触发实例配置-租户控制台
pub struct ReachVcodeStrategyCtApi;

#[cfg_attr(feature = "simple-client", bios_sdk_invoke::simple_invoke_client(Client<'_>))]
#[poem_openapi::OpenApi(prefix_path = "/ct/vcode/strategy", tag = "bios_basic::ApiTag::App")]
impl ReachVcodeStrategyCtApi {
    /// 添加vcode策略
    #[oai(method = "post", path = "/")]
    pub async fn add_vcode_strategy(&self, json_body: Json<ReachVCodeStrategyAddReq>, TardisContextExtractor(ctx): TardisContextExtractor) -> TardisApiResult<String> {
        let funs = get_tardis_inst();
        let mut add_req = json_body.0;
        let id = VcodeStrategeServ::add_rbum(&mut add_req, &funs, &ctx).await?;
        TardisResp::ok(id)
    }

    /// 修改vcode策略
    #[oai(method = "put", path = "/:id")]
    pub async fn modify_vcode_strategy(
        &self,
        id: Path<String>,
        json_body: Json<ReachVCodeStrategyModifyReq>,
        TardisContextExtractor(ctx): TardisContextExtractor,
    ) -> TardisApiResult<Void> {
        let funs = get_tardis_inst();
        let mut mod_req = json_body.0;
        VcodeStrategeServ::modify_rbum(&id, &mut mod_req, &funs, &ctx).await?;
        TardisResp::ok(Void)
    }

    /// 修改vcode策略
    #[oai(method = "get", path = "/")]
    pub async fn get_vcode_strategy(
        &self,
        reach_set_id: Query<Option<String>>,
        TardisContextExtractor(ctx): TardisContextExtractor,
    ) -> TardisApiResult<Option<ReachVCodeStrategySummaryResp>> {
        let funs = get_tardis_inst();
        let filter = ReachVCodeStrategyFilterReq {
            base_filter: Default::default(),
            rel_reach_set_id: reach_set_id.0,
        };
        let resp = VcodeStrategeServ::find_one_rbum(&filter, &funs, &ctx).await?;
        TardisResp::ok(resp)
    }
}
