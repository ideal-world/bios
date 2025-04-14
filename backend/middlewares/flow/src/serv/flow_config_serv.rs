use bios_basic::rbum::{helper::rbum_scope_helper, rbum_enumeration::RbumScopeLevelKind};
use bios_sdk_invoke::clients::spi_kv_client::{KvItemSummaryResp, SpiKvClient};
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    web::web_resp::TardisPage,
    TardisFuns, TardisFunsInst,
};

use crate::{
    dto::{flow_config_dto::{FlowConfigModifyReq, FlowReviewConfigLabelResp, FlowRootConfigResp}, flow_inst_dto::FlowInstFilterReq},
    flow_constants,
};

use super::{flow_inst_serv::FlowInstServ, flow_rel_serv::{FlowRelKind, FlowRelServ}};

pub struct FlowConfigServ;

impl FlowConfigServ {
    pub async fn modify_config(modify_req: &Vec<FlowConfigModifyReq>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        for req in modify_req {
            SpiKvClient::add_or_modify_item(
                &format!("{}:config:{}", flow_constants::DOMAIN_CODE, req.code.clone()),
                &req.value,
                None,
                None,
                Some(RbumScopeLevelKind::Root.to_int()),
                funs,
                ctx,
            )
            .await?;
        }
        Ok(())
    }

    pub async fn get_config(funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Option<TardisPage<KvItemSummaryResp>>> {
        let prefix = format!("{}:config:", flow_constants::DOMAIN_CODE);
        let mut result = SpiKvClient::match_items_by_key_prefix(prefix.clone(), None, 1, 100, Some(false), funs, ctx).await?;
        result.as_mut().map(|configs| configs.records.iter_mut().map(|config| config.key = config.key.replace(&prefix, "")).collect::<Vec<_>>());
        Ok(result)
    }

    // 获取父级配置 租户id:项目id:项目模板id:review_config
    pub async fn get_root_config(root_tag: &str, child_tag: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Option<FlowReviewConfigLabelResp>> {
        let tenant_paths = rbum_scope_helper::get_path_item(1, &ctx.own_paths).unwrap_or_default();
        let app_paths = rbum_scope_helper::get_path_item(2, &ctx.own_paths).unwrap_or_default();
        let key = if let Some(template_id) = FlowRelServ::find_from_simple_rels(&FlowRelKind::FlowAppTemplate, &app_paths, None, None, funs, ctx).await?.pop().map(|rel| rel.rel_id) {
            format!("__tag__:{}:_:{}:{}_config", tenant_paths, template_id, root_tag.to_ascii_lowercase())
        } else {
            format!("__tag__:{}:{}:_:{}_config", tenant_paths, app_paths, root_tag.to_ascii_lowercase())
        };
        let result = SpiKvClient::get_item(key, None, funs, ctx)
            .await?
            .ok_or_else(|| funs.err().not_found("flow_config", "get_root_config", "review config is not found", "404-flow-config-not-found"))?;
        let config = TardisFuns::json.json_to_obj::<Vec<FlowRootConfigResp>>(result.value)?;
        if let Some(config) = config.into_iter().find(|conf| conf.code == *child_tag) {
            TardisFuns::json.str_to_obj::<FlowReviewConfigLabelResp>(&config.label).map_or(Ok(None), |o| Ok(Some(o)))
        } else {
            Ok(None)
        }
    }
}
