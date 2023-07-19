use bios_sdk_invoke::clients::spi_kv_client::{KvItemSummaryResp, SpiKvClient};
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    web::web_resp::TardisPage,
    TardisFunsInst,
};

use crate::{dto::flow_config_dto::FlowConfigEditReq, flow_constants};

pub struct FlowConfigServ;

impl FlowConfigServ {
    pub async fn edit_config(edit_req: &Vec<FlowConfigEditReq>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        for req in edit_req {
            SpiKvClient::add_or_modify_item(&format!("{}:{}", flow_constants::DOMAIN_CODE, req.code.clone()), &req.value, None, funs, ctx).await?;
        }
        Ok(())
    }

    pub async fn get_config(funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Option<TardisPage<KvItemSummaryResp>>> {
        SpiKvClient::match_items_by_key_prefix(flow_constants::DOMAIN_CODE.to_string(), None, 1, 100, funs, ctx).await
    }
}
