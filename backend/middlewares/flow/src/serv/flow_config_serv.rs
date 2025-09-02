use bios_basic::rbum::{helper::rbum_scope_helper, rbum_enumeration::RbumScopeLevelKind};
use bios_sdk_invoke::clients::spi_kv_client::{KvItemSummaryResp, SpiKvClient};
use itertools::Itertools;
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    web::web_resp::TardisPage,
    TardisFuns, TardisFunsInst,
};

use crate::{
    dto::flow_config_dto::{FlowConfigModifyReq, FlowReviewConfigLabelResp, FlowRootConfigResp},
    flow_constants,
};

use super::{flow_model_serv::FlowModelServ, flow_rel_serv::{FlowRelKind, FlowRelServ}};

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
    pub async fn get_root_config(root_tag: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Vec<FlowRootConfigResp>> {
        let tenant_paths = rbum_scope_helper::get_path_item(1, &ctx.own_paths).unwrap_or_default();
        let app_paths = rbum_scope_helper::get_path_item(2, &ctx.own_paths).unwrap_or_default();
        let key = if let Some(template_id) = FlowModelServ::find_rel_template_id(funs, ctx).await?
        {
            format!("__tag__:_:_:{}:{}_config", template_id, root_tag.to_ascii_lowercase())
        } else {
            format!("__tag__:{}:{}:_:{}_config", tenant_paths, app_paths, root_tag.to_ascii_lowercase())
        };
        let result = SpiKvClient::get_item(key, None, funs, ctx)
            .await?
            .ok_or_else(|| funs.err().not_found("flow_config", "get_root_config", "review config is not found", "404-flow-config-not-found"))?;
        TardisFuns::json.json_to_obj::<Vec<FlowRootConfigResp>>(result.value)
    }

    pub async fn modify_root_config_by_tag(
        root_tag: &str,
        child_tag: &str,
        original_state: &str,
        original_state_name: &str,
        new_state: &str,
        new_state_name: &str,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<()> {
        let tenant_paths = rbum_scope_helper::get_path_item(1, &ctx.own_paths).unwrap_or_default();
        let app_paths = rbum_scope_helper::get_path_item(2, &ctx.own_paths).unwrap_or_default();
        let key = if let Some(template_id) = FlowModelServ::find_rel_template_id(funs, ctx).await?
        {
            format!("__tag__:_:_:{}:{}_config", template_id, root_tag.to_ascii_lowercase())
        } else {
            format!("__tag__:{}:{}:_:{}_config", tenant_paths, app_paths, root_tag.to_ascii_lowercase())
        };

        if let Ok(root_config) = Self::get_root_config(root_tag, funs, ctx).await {
            if let Some(mut child_config) = Self::get_root_config_by_tag(&root_config, child_tag)? {
                if child_config.origin_status.contains(&original_state.to_string()) {
                    child_config.origin_status = child_config.origin_status.into_iter().filter(|state_id| state_id != original_state).collect_vec();
                    child_config.origin_status.push(new_state.to_string());
                }
                let mut origin_status_names = child_config.origin_status_name.split(',').map(|s| s.to_string()).collect_vec();
                if origin_status_names.contains(&original_state_name.to_string()) {
                    origin_status_names = origin_status_names.into_iter().filter(|state_id| state_id != original_state_name).collect_vec();
                    origin_status_names.push(new_state_name.to_string());
                    child_config.origin_status_name = origin_status_names.join(",").to_string();
                }
                if child_config.pass_status == *original_state {
                    child_config.pass_status = new_state.to_string();
                }
                if child_config.unpass_status == *original_state {
                    child_config.unpass_status = new_state.to_string();
                }
                let mut new_root_config = root_config.into_iter().filter(|child_config| child_config.code != *child_tag).collect_vec();
                new_root_config.push(FlowRootConfigResp {
                    url: None,
                    icon: "".to_string(),
                    color: "".to_string(),
                    code: child_tag.to_string(),
                    label: TardisFuns::json.obj_to_string(&child_config)?,
                    service: None,
                });
                SpiKvClient::add_or_modify_item(&key, &new_root_config, None, None, None, funs, ctx).await?;
            }
        }

        Ok(())
    }

    pub fn get_root_config_by_tag(config: &[FlowRootConfigResp], tag: &str) -> TardisResult<Option<FlowReviewConfigLabelResp>> {
        if let Some(config) = config.iter().find(|conf| conf.code == *tag) {
            TardisFuns::json.str_to_obj::<FlowReviewConfigLabelResp>(&config.label).map_or(Ok(None), |o| Ok(Some(o)))
        } else {
            Ok(None)
        }
    }
}
