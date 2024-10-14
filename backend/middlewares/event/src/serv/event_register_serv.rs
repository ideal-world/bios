use std::collections::HashMap;

use asteroid_mq::prelude::NodeId;
use tardis::tokio::sync::RwLock;
use tardis::{
    basic::{dto::TardisContext, error::TardisError, result::TardisResult},
    tardis_static,
};
#[derive(Clone, Default, Debug)]
pub struct EventRegisterServ {}
tardis_static! {
    #[inline]
    pub nid_ctx_map: RwLock<HashMap<NodeId, TardisContext>>;
}
impl EventRegisterServ {
    pub async fn register_ctx(&self, ctx: &TardisContext) -> TardisResult<NodeId> {
        let id = NodeId::snowflake();
        nid_ctx_map().write().await.insert(id, ctx.clone());
        Ok(id)
    }
    pub async fn unregister_ctx(&self, id: NodeId) -> TardisResult<()> {
        nid_ctx_map().write().await.remove(&id);
        Ok(())
    }
    pub async fn get_ctx(&self, id: NodeId) -> TardisResult<TardisContext> {
        nid_ctx_map().read().await.get(&id).cloned().ok_or_else(|| TardisError::not_found("node not found", "event-node-not-found"))
    }
}
