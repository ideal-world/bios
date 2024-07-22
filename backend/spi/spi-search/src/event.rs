use bios_sdk_invoke::{
    clients::{
        event_client::{BiosEventCenter, EventCenter},
        spi_search_client::event::SEARCH_AVATAR,
    },
    dto::search_item_dto::{SearchEventItemDeleteReq, SearchEventItemModifyReq, SearchItemAddReq},
};

use tardis::basic::{dto::TardisContext, result::TardisResult};

use crate::{search_initializer::get_tardis_inst, serv};

async fn handle_add_event(req: SearchItemAddReq, ctx: TardisContext) -> TardisResult<()> {
    let funs = get_tardis_inst();
    serv::search_item_serv::add(&mut req.into(), &funs, &ctx).await?;
    Ok(())
}

async fn handle_modify_event(req: SearchEventItemModifyReq, ctx: TardisContext) -> TardisResult<()> {
    let funs = get_tardis_inst();
    serv::search_item_serv::modify(&req.tag, &req.key, &mut req.item.into(), &funs, &ctx).await?;
    Ok(())
}

async fn handle_delete_event(req: SearchEventItemDeleteReq, ctx: TardisContext) -> TardisResult<()> {
    let funs = get_tardis_inst();
    serv::search_item_serv::delete(&req.tag, &req.key, &funs, &ctx).await?;
    Ok(())
}

pub(crate) fn register_search_events() {
    if let Some(bios_event_center) = BiosEventCenter::worker_queue() {
        bios_event_center.subscribe(handle_modify_event);
        bios_event_center.subscribe(handle_add_event);
        bios_event_center.subscribe(handle_delete_event);
        bios_event_center.add_avatar(SEARCH_AVATAR);
    }
}
