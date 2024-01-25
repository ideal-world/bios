use std::time::Duration;

use bios_sdk_invoke::{
    clients::event_client::{self, EventTopicConfig},
    dto::search_item_dto::{SearchEventItemDeleteReq, SearchEventItemModifyReq},
};
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    log::{error, info, warn},
    tokio,
    web::ws_processor::TardisWebsocketMessage,
    TardisFuns,
};

use crate::{
    search_constants::{EVENT_ADD_SEARCH, EVENT_DELETE_SEARCH, EVENT_MODIFY_SEARCH},
    search_initializer::get_tardis_inst,
    serv,
};

pub const RECONNECT_INTERVAL: Duration = Duration::from_secs(10);

pub async fn start_search_event_service(config: &EventTopicConfig) -> TardisResult<()> {
    info!("[Bios.Search] starting event service");
    let funs = get_tardis_inst();
    let client = event_client::EventClient::new(config.base_url.as_str(), &funs);
    let mut event_conf = config.clone();
    if event_conf.avatars.is_empty() {
        event_conf.avatars.push(format!("{}/{}", event_conf.topic_code, tardis::pkg!()))
    }
    let resp = client.register(&event_conf.into()).await?;
    let ws_client = TardisFuns::ws_client(&resp.ws_addr, |message| async move {
        let Ok(json_str) = message.to_text() else { return None };
        let Ok(TardisWebsocketMessage { msg, event, .. }) = TardisFuns::json.str_to_obj(json_str) else {
            return None;
        };
        match event.as_deref() {
            Some(EVENT_ADD_SEARCH) => {
                let Ok((mut req, ctx)) = TardisFuns::json.json_to_obj(msg) else {
                    return None;
                };
                tokio::spawn(async move {
                    let funs = get_tardis_inst();
                    let result = serv::search_item_serv::add(&mut req, &funs, &ctx).await;
                    if let Err(err) = result {
                        error!("[Bios.Search] failed to search item: {}", err);
                    }
                });
            }
            Some(EVENT_MODIFY_SEARCH) => {
                let Ok((req, ctx)) = TardisFuns::json.json_to_obj::<(SearchEventItemModifyReq, TardisContext)>(msg) else {
                    return None;
                };
                let Ok(mut modify_req) = TardisFuns::json.json_to_obj(req.item) else { return None };
                tokio::spawn(async move {
                    let funs = get_tardis_inst();
                    let result = serv::search_item_serv::modify(&req.tag, &req.key, &mut modify_req, &funs, &ctx).await;
                    if let Err(err) = result {
                        error!("[Bios.Search] failed to search item: {}", err);
                    }
                });
            }
            Some(EVENT_DELETE_SEARCH) => {
                let Ok((req, ctx)) = TardisFuns::json.json_to_obj::<(SearchEventItemDeleteReq, TardisContext)>(msg) else {
                    return None;
                };
                tokio::spawn(async move {
                    let funs = get_tardis_inst();
                    let result = serv::search_item_serv::delete(&req.tag, &req.key, &funs, &ctx).await;
                    if let Err(err) = result {
                        error!("[Bios.Search] failed to search item: {}", err);
                    }
                });
            }
            Some(unknown_event) => {
                warn!("[Bios.Search] event receive unknown event {unknown_event}")
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
                error!("[Bios.Search] failed to reconnect to event service: {}", err);
            }
            tokio::time::sleep(RECONNECT_INTERVAL).await;
        }
    });
    Ok(())
}
