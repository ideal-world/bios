use bios_basic::rbum::rbum_enumeration::RbumScopeLevelKind;
use bios_sdk_invoke::clients::spi_kv_client::{KvItemSummaryResp, SpiKvClient};
use tardis::{
    basic::{dto::TardisContext, result::TardisResult}, web::web_resp::TardisPage, TardisFuns, TardisFunsInst
};

use crate::{dto::flow_config_dto::{FlowConfigModifyReq, FlowReviewConfigLabelResp, FlowRootConfigResp}, flow_constants};

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

    // 获取父级配置
    pub async fn get_root_config(tag: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Option<FlowReviewConfigLabelResp>> {
        let result = SpiKvClient::get_item(format!("__tag__:{}:{}", ctx.own_paths, flow_constants::ROOT_CONFIG_KV), None, funs, ctx).await?
        .ok_or_else(|| funs.err().not_found("flow_config", "get_review_config", "review config is not found", "404-flow-config-not-found"))?;
        let config = TardisFuns::json.json_to_obj::<Vec<FlowRootConfigResp>>(result.value)?;
        Ok(config.into_iter().find(|conf| conf.code == tag.to_string()).map(|conf| conf.label))
    }
}
