use bios_basic::rbum::helper::rbum_scope_helper;
use bios_sdk_invoke::clients::{reach_client::{ReachClient, ReachMsgSendReq}, spi_kv_client::SpiKvClient};
use itertools::Itertools;
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    TardisFunsInst,
};

pub struct FlowReachClient;

impl FlowReachClient {
    async fn send_message(req: &ReachMsgSendReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        ReachClient::send_message(req, funs, ctx).await
    }

    pub async fn send_approve_start_message() -> TardisResult<()> {
        Ok(())
    }
}
