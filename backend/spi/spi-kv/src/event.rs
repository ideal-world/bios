use tardis::{basic::dto::TardisContext, log::{self as tracing, instrument}, TardisFuns};
use bios_sdk_invoke::clients::{event_client::{BiosEventCenter, EventCenter}, spi_kv_client::{KvItemAddOrModifyReq, KvItemDeleteReq}};
use tardis::basic::result::TardisResult;
use crate::{get_tardis_inst, serv};
#[instrument]
async fn handle_kv_add_event(req: KvItemAddOrModifyReq, ctx: TardisContext) -> TardisResult<()> {
    let funs = get_tardis_inst();
    serv::kv_item_serv::add_or_modify_item(&mut req.into(), &funs, &ctx).await?;
    Ok(())
}

#[instrument]
async fn handle_kv_delete_event(req: KvItemDeleteReq, ctx: TardisContext) -> TardisResult<()> {
    let funs = get_tardis_inst();
    serv::kv_item_serv::delete_item(req.key.trim().to_string(), &funs, &ctx).await?;
    Ok(())
}

pub fn register_kv_events() {
    if let Some(bios_event_center) = TardisFuns::store().get_singleton::<BiosEventCenter>() {
        bios_event_center.subscribe(handle_kv_add_event);
        bios_event_center.subscribe(handle_kv_delete_event);
    }
}
