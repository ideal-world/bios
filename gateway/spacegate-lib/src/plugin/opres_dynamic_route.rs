use serde::{Deserialize, Serialize};
use spacegate_shell::{kernel::{helper_layers::async_filter::AsyncFilter, ReqOrResp}, plugin::{def_plugin, MakeSgLayer}, spacegate_ext_redis::RedisClientRepoError, SgRequestExt};
use tardis::futures::future::BoxFuture;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpresDynamicRoute {
    
}


impl AsyncFilter for OpresDynamicRoute {
    type Future = BoxFuture<'static, ReqOrResp>;

    fn filter(&self, req: http::Request<spacegate_shell::SgBody>) -> Self::Future {
        async move {
            let Some(client) = req.get_redis_client_by_gateway_name() else {
                RedisClientRepoError::new(name, message).await;
            }
        }
    }
}

impl MakeSgLayer for OpresDynamicRoute {
    fn make_layer(&self) -> spacegate_shell::kernel::BoxResult<spacegate_shell::SgBoxLayer> {
        
    }
}

def_plugin!("opres-dynamic-route", OpresDynamicRoutePlugin, OpresDynamicRoute)