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

use super::{
    flow_model_serv::FlowModelServ,
    flow_rel_serv::{FlowRelKind, FlowRelServ},
};

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
        let key = if let Some(mut template_id) = FlowModelServ::find_rel_template_id(funs, ctx).await? {
            // 引用的模板，则向上获取根模板ID的配置
            while let Some(p_template_id) = FlowRelServ::find_to_simple_rels(&FlowRelKind::FlowTemplateTemplate, &template_id, None, None, funs, ctx).await?.pop().map(|r| r.rel_id)
            {
                template_id = p_template_id;
            }
            format!("__tag__:_:_:{}:{}_config", template_id, root_tag.to_ascii_lowercase())
        } else {
            // 复制的模板，则直接获取当前ctx下的配置
            let tenant_paths = rbum_scope_helper::get_path_item(1, &ctx.own_paths).unwrap_or_default();
            let app_paths = rbum_scope_helper::get_path_item(2, &ctx.own_paths).unwrap_or_default();
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
        let key = Self::get_root_config_key(None, root_tag, ctx);

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

    pub async fn add_or_modify_root_config(rel_template_id: String, target_template_id: Option<String>, root_tag: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        // 引用的模板，则向上获取根模板ID的配置
        let mut template_id = rel_template_id;
        while let Some(p_template_id) = FlowRelServ::find_to_simple_rels(&FlowRelKind::FlowTemplateTemplate, &template_id, None, None, funs, ctx).await?.pop().map(|r| r.rel_id)
        {
            template_id = p_template_id;
        }
        let source_key = format!("__tag__:_:_:{}:{}_config", template_id, root_tag.to_ascii_lowercase());

        let config = SpiKvClient::get_item(source_key, None, funs, ctx).await?;
        if let Some(config) = config {
            let config = TardisFuns::json.json_to_obj::<Vec<FlowRootConfigResp>>(config.value)?;
            let target_key = Self::get_root_config_key(target_template_id, root_tag, ctx);
            SpiKvClient::add_or_modify_item(&target_key, &config, None, None, None, funs, ctx).await
        } else {
            Ok(())
        }
    }

    pub fn get_root_config_by_tag(config: &[FlowRootConfigResp], tag: &str) -> TardisResult<Option<FlowReviewConfigLabelResp>> {
        if let Some(config) = config.iter().find(|conf| conf.code == *tag) {
            TardisFuns::json.str_to_obj::<FlowReviewConfigLabelResp>(&config.label).map_or(Ok(None), |o| Ok(Some(o)))
        } else {
            Ok(None)
        }
    }

    fn get_root_config_key(rel_template_id: Option<String>, root_tag: &str, ctx: &TardisContext) -> String {
        let tenant_paths = rbum_scope_helper::get_path_item(1, &ctx.own_paths).unwrap_or_default();
        let app_paths = rbum_scope_helper::get_path_item(2, &ctx.own_paths).unwrap_or_default();
        if let Some(rel_template_id) = rel_template_id {
            format!("__tag__:_:_:{}:{}_config", rel_template_id, root_tag.to_ascii_lowercase())
        } else {
            format!("__tag__:{}:{}:_:{}_config", tenant_paths, app_paths, root_tag.to_ascii_lowercase())
        }
    }
}
