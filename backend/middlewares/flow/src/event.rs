use bios_sdk_invoke::clients::{
    event_client::{asteroid_mq::prelude::TopicCode, get_topic, mq_error, ContextHandler},
    flow_client::{event::FLOW_AVATAR, FlowFrontChangeReq, FlowPostChangeReq},
};
use tardis::basic::{dto::TardisContext, error::TardisError, result::TardisResult};

use crate::{flow_constants::get_tardis_inst, serv::flow_event_serv::FlowEventServ};
pub const FLOW_TOPIC: TopicCode = TopicCode::const_new("flow");

pub async fn handle_events() -> TardisResult<()> {
    use bios_sdk_invoke::clients::event_client::asteroid_mq::prelude::*;
    if let Some(topic) = get_topic(&FLOW_TOPIC) {
        topic
            .create_endpoint([Interest::new("*")])
            .await
            .map_err(mq_error)?
            .create_event_loop()
            .with_handler(ContextHandler(handle_front_change))
            .with_handler(ContextHandler(handle_post_change))
            .spawn();
    }

    Ok(())
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
