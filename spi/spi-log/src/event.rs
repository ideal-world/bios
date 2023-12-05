use std::time::Duration;

use crate::{
    event::event_client::EventListenerRegisterReq,
    log_constants::{DOMAIN_CODE, EVENT_ADD},
    log_initializer::get_tardis_inst,
    serv,
};
use bios_sdk_invoke::clients::event_client;
use tardis::{basic::result::TardisResult, log::{error, info}, tokio, web::ws_client, TardisFuns, TardisFunsInst};
pub struct LogEventServ {}
pub const RECONNECT_INTERVAL: Duration = Duration::from_secs(10);

pub async fn start_log_event_service(base_url: &str, topic_sk: &str) -> TardisResult<()> {
    info!("[Bios.Log] starting event service");
    let funs = get_tardis_inst();
    let client = event_client::EventClient::new(base_url, &funs);
    let req = EventListenerRegisterReq {
        topic_code: DOMAIN_CODE.to_string(),
        topic_sk: Some(topic_sk.to_string()),
        events: Some(vec![EVENT_ADD.into()]),
        avatars: vec![format!("{DOMAIN_CODE}/server")],
        subscribe_mode: false,
    };
    let resp = client.register(&req).await?;
    let ws_client = TardisFuns::ws_client(&resp.ws_addr, |message| async move {
        let Ok(json_str) = message.to_text() else { return None };
        let Ok((mut req, ctx)) = TardisFuns::json.str_to_obj(json_str) else {
            return None;
        };
        tokio::spawn(async move {
            let funs = get_tardis_inst();
            let result = serv::log_item_serv::add(&mut req, &funs, &ctx).await;
            if let Err(err) = result {
                error!("[Bios.Log] failed to log item: {}", err);
            }
        });
        None
    })
    .await?;
    tokio::spawn(async move {
        loop {
            // it's ok todo so, reconnect will be blocked until the previous ws_client is dropped
            let result = ws_client.reconnect().await;
            if let Err(err) = result {
                error!("[Bios.Log] failed to reconnect to event service: {}", err);
            }
            tokio::time::sleep(RECONNECT_INTERVAL).await;
        }
    });
    Ok(())
}
