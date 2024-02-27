use async_trait::async_trait;
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    log::info,
    web::{ws_client::TardisWSClient, ws_processor::TardisWebsocketReq},
    TardisFuns,
};

use crate::flow_constants::{EVENT_FRONT_CHANGE, EVENT_MODIFY_ASSIGNED, EVENT_POST_CHANGE};

#[async_trait]
pub trait FlowEventExt {
    async fn publish_modify_assigned(&self, inst_id: String, assigned_id: String, from: String, spi_app_id: String, ctx: &TardisContext) -> TardisResult<()>;
    async fn publish_front_change(&self, inst_id: String, from: String, spi_app_id: String, ctx: &TardisContext) -> TardisResult<()>;
    async fn publish_post_change(&self, inst_id: String, next_transition_id: String, from: String, spi_app_id: String, ctx: &TardisContext) -> TardisResult<()>;
}

#[async_trait]
impl FlowEventExt for TardisWSClient {
    async fn publish_modify_assigned(&self, inst_id: String, assigned_id: String, from: String, spi_app_id: String, ctx: &TardisContext) -> TardisResult<()> {
        let spi_ctx = TardisContext { owner: spi_app_id, ..ctx.clone() };
        let req = TardisWebsocketReq {
            msg: TardisFuns::json.obj_to_json(&(inst_id, assigned_id, spi_ctx)).expect("invalid json"),
            to_avatars: Some(vec!["flow/service".into()]),
            from_avatar: from,
            event: Some(EVENT_MODIFY_ASSIGNED.into()),
            ..Default::default()
        };
        info!("event add log {}", TardisFuns::json.obj_to_string(&req).expect("invalid json"));
        self.send_obj(&req).await?;
        return Ok(());
    }
    async fn publish_front_change(&self, inst_id: String, from: String, spi_app_id: String, ctx: &TardisContext) -> TardisResult<()> {
        let spi_ctx = TardisContext { owner: spi_app_id, ..ctx.clone() };
        let req = TardisWebsocketReq {
            msg: TardisFuns::json.obj_to_json(&(inst_id, spi_ctx)).expect("invalid json"),
            to_avatars: Some(vec!["flow/service".into()]),
            from_avatar: from,
            event: Some(EVENT_FRONT_CHANGE.into()),
            ..Default::default()
        };
        info!("event add log {}", TardisFuns::json.obj_to_string(&req).expect("invalid json"));
        self.send_obj(&req).await?;
        return Ok(());
    }

    async fn publish_post_change(&self, inst_id: String, next_transition_id: String, from: String, spi_app_id: String, ctx: &TardisContext) -> TardisResult<()> {
        let spi_ctx = TardisContext { owner: spi_app_id, ..ctx.clone() };
        let req = TardisWebsocketReq {
            msg: TardisFuns::json.obj_to_json(&(inst_id, next_transition_id, spi_ctx)).expect("invalid json"),
            to_avatars: Some(vec!["flow/service".into()]),
            from_avatar: from,
            event: Some(EVENT_POST_CHANGE.into()),
            ..Default::default()
        };
        info!("event add log {}", TardisFuns::json.obj_to_string(&req).expect("invalid json"));
        self.send_obj(&req).await?;
        return Ok(());
    }
}
