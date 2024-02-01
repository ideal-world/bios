use std::time::Duration;

use bios_sdk_invoke::clients::event_client::{self, EventTopicConfig};
use tardis::{
    basic::result::TardisResult,
    log::{error, info, warn},
    tokio,
    web::ws_processor::TardisWebsocketMessage,
    TardisFuns,
};

use crate::{flow_constants::{self, get_tardis_inst, EVENT_FRONT_CHANGE, EVENT_POST_CHANGE, EVENT_MODIFY_ASSIGNED}, serv::flow_event_serv::FlowEventServ};
pub const RECONNECT_INTERVAL: Duration = Duration::from_secs(10);

pub async fn start_flow_event_service(config: &EventTopicConfig) -> TardisResult<()> {
    info!("[Bios.Flow] starting event service");
    let funs = flow_constants::get_tardis_inst();
    let client = event_client::EventClient::new(config.base_url.as_str(), &funs);
    let mut event_conf = config.clone();
    if event_conf.avatars.is_empty() {
        event_conf.avatars.push(format!("{}/{}", event_conf.topic_code, tardis::pkg!()))
    }
    let resp = client.register(&event_conf.into()).await?;
    let ws_client = TardisFuns::ws_client(&resp.ws_addr, |message| async move {
        let Ok(json_str) = message.to_text() else { return None };
        info!("[Bios.Flow] event msg: {json_str}");
        let Ok(TardisWebsocketMessage { msg, event, .. }) = TardisFuns::json.str_to_obj(json_str) else {
            return None;
        };
        match event.as_deref() {
            Some(EVENT_FRONT_CHANGE) => {
                let Ok((inst_id, ctx)) = TardisFuns::json.json_to_obj::<(String, _)>(msg) else {
                    return None;
                };
                tokio::spawn(async move {
                    let funs = get_tardis_inst();
                    let result = FlowEventServ::do_front_change(&inst_id, &ctx, &funs).await;
                    if let Err(err) = result {
                        error!("[Bios.Log] failed to do front change: {}, inst_id: {}", err, inst_id);
                    }
                });
            },
            Some(EVENT_POST_CHANGE) => {

            },
            Some(EVENT_MODIFY_ASSIGNED) => {

            },
            Some(unknown_event) => {
                warn!("[Bios.Flow] event receive unknown event {unknown_event}")
            }
            _ => {}
        }
        None
    })
    .await?;
    tokio::spawn(async move {
        loop {
            // it's ok todo so, reconnect will be blocked until the previous ws_client is dropped
            let result = ws_client.reconnect().await;
            if let Err(err) = result {
                error!("[Bios.Flow] failed to reconnect to event service: {}", err);
            }
            tokio::time::sleep(RECONNECT_INTERVAL).await;
        }
    });
    Ok(())
}
