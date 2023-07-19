use bios_sdk_invoke::clients::spi_kv_client::{KvItemSummaryResp, SpiKvClient};
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    web::web_resp::TardisPage,
    TardisFunsInst,
};

use crate::{dto::flow_config_dto::FlowConfigModifyReq, flow_constants};

pub struct FlowConfigServ;

impl FlowConfigServ {
    pub async fn modify_config(modify_req: &Vec<FlowConfigModifyReq>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        for req in modify_req {
            SpiKvClient::add_or_modify_item(&format!("{}:config:{}", flow_constants::DOMAIN_CODE, req.code.clone()), &req.value, None, funs, ctx).await?;
        }
        Ok(())
    }

    pub async fn get_config(funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Option<TardisPage<KvItemSummaryResp>>> {
        SpiKvClient::match_items_by_key_prefix(format!("{}:config:", flow_constants::DOMAIN_CODE), None, 1, 100, funs, ctx).await
    }
}
