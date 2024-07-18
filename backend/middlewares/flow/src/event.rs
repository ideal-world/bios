use bios_sdk_invoke::clients::{
    event_client::{BiosEventCenter, EventCenter},
    flow_client::{event::FLOW_AVATAR, FlowFrontChangeReq, FlowPostChangeReq},
};
use tardis::basic::{dto::TardisContext, result::TardisResult};

use crate::{flow_constants::get_tardis_inst, serv::flow_event_serv::FlowEventServ};

pub fn flow_register_events() {
    if let Some(event_center) = BiosEventCenter::worker_queue() {
        event_center.subscribe(handle_front_change);
        event_center.subscribe(handle_post_change);
        event_center.add_avatar(FLOW_AVATAR);
    }
}

async fn handle_front_change(req: FlowFrontChangeReq, ctx: TardisContext) -> TardisResult<()> {
    let funs = get_tardis_inst();
    FlowEventServ::do_front_change(&req.inst_id, &ctx, &funs).await?;
    Ok(())
}

async fn handle_post_change(req: FlowPostChangeReq, ctx: TardisContext) -> TardisResult<()> {
    let funs = get_tardis_inst();
    FlowEventServ::do_post_change(&req.inst_id, &req.next_transition_id, &ctx, &funs).await?;
    Ok(())
}
