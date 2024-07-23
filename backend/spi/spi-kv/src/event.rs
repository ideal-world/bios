use crate::{get_tardis_inst, serv};
use bios_sdk_invoke::clients::{
    event_client::{BiosEventCenter, EventCenter},
    spi_kv_client::{event::KV_AVATAR, KvItemAddOrModifyReq, KvItemDeleteReq},
};
use tardis::basic::result::TardisResult;
use tardis::{
    basic::dto::TardisContext,
    log::{self as tracing, instrument},
};
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
    if let Some(bios_event_center) = BiosEventCenter::worker_queue() {
        bios_event_center.subscribe(handle_kv_add_event);
        bios_event_center.subscribe(handle_kv_delete_event);
        bios_event_center.add_avatar(KV_AVATAR);
    }
}
