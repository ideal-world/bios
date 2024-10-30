use std::sync::Arc;

use asteroid_mq::prelude::NodeId;
use tardis::basic::{dto::TardisContext, error::TardisError, result::TardisResult};
use tardis::chrono::{TimeDelta, Utc};
use tardis::{TardisFuns, TardisFunsInst};

use crate::dto::event_dto::EventRegisterResp;
#[derive(Clone)]
pub struct EventRegisterServ {
    pub(crate) funs: Arc<TardisFunsInst>,
}

impl EventRegisterServ {
    pub fn cache_key(&self, id_base64: &str) -> String {
        format!("bios:event:node:{}", id_base64)
    }
    const EXPIRE_SEC: u64 = 60 * 30;
    pub async fn register_ctx(&self, ctx: &TardisContext) -> TardisResult<EventRegisterResp> {
        let cache = self.funs.cache();
        let id_base64 = NodeId::snowflake().to_base64();
        let now = Utc::now();

        cache.set_ex(&self.cache_key(&id_base64), &ctx.to_json()?, Self::EXPIRE_SEC).await?;
        Ok(EventRegisterResp {
            node_id: id_base64,
            expire_at: now + TimeDelta::seconds(Self::EXPIRE_SEC as i64),
        })
    }
    pub async fn unregister_ctx(&self, id: NodeId) -> TardisResult<()> {
        let cache = self.funs.cache();
        let id_base_64 = id.to_base64();
        cache.del(&self.cache_key(&id_base_64)).await?;
        Ok(())
    }
    pub async fn get_ctx(&self, id: NodeId) -> TardisResult<TardisContext> {
        let cache = self.funs.cache();
        let id_base64 = id.to_base64();
        let cache_key = self.cache_key(&id_base64);
        let ctx_str = cache.get(&cache_key).await?.ok_or_else(|| TardisError::not_found("node not found", "event-node-not-found"))?;
        let ctx = TardisFuns::json.str_to_obj(&ctx_str)?;
        cache.set_ex(&cache_key, &ctx_str, Self::EXPIRE_SEC).await?;
        Ok(ctx)
    }
}
