use http::{
    uri::{Authority, PathAndQuery},
    Response, StatusCode, Uri,
};
use serde::{Deserialize, Serialize};
use spacegate_shell::{
    kernel::{
        extension::MatchedSgRouter,
        helper_layers::async_filter::{AsyncFilter, AsyncFilterRequestLayer},
        layers::http_route::match_request::SgHttpPathMatch,
        ReqOrResp,
    },
    plugin::{
        def_plugin,
        model::{SgHttpPathModifier, SgHttpPathModifierType},
        MakeSgLayer, PluginError,
    },
    spacegate_ext_redis::{redis::AsyncCommands, AsRedisKey},
    SgBoxLayer, SgRequestExt, SgResponseExt,
};
use tardis::{futures::future::BoxFuture, tardis_static};

use crate::marker::OpresKey;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpresDynamicRoute {
    pub prefix: String,
}
tardis_static! {
    modifier: SgHttpPathModifier =SgHttpPathModifier{
        kind: SgHttpPathModifierType::ReplacePrefixMatch,
        value: "/".to_string(),
    };
}
impl AsyncFilter for OpresDynamicRoute {
    type Future = BoxFuture<'static, ReqOrResp>;

    fn filter(&self, mut req: http::Request<spacegate_shell::SgBody>) -> Self::Future {
        let prefix = self.prefix.clone();
        let task = async move {
            let Some(client) = req.get_redis_client_by_gateway_name() else {
                return Err(Response::with_code_message(StatusCode::BAD_GATEWAY, "missing redis client."));
            };
            let Some(token) = req.extract_marker::<OpresKey>() else {
                return Err(Response::with_code_message(StatusCode::UNAUTHORIZED, "missing op res authorization"));
            };
            let Some(matched) = req.extensions().get::<MatchedSgRouter>() else {
                return Err(Response::with_code_message(StatusCode::BAD_GATEWAY, "plugin called on fallback match or gateway layer."));
            };
            let Some(SgHttpPathMatch::Prefix(prefix_match)) = &matched.path else {
                return Err(Response::with_code_message(StatusCode::BAD_GATEWAY, "plugin should be attached on prefix match."));
            };
            let key = format!("{}:rewrite", token.as_redis_key(prefix));
            let mut conn = client.get_conn().await;
            let domain: String = conn.get(key).await.map_err(PluginError::bad_gateway::<OpresDynamicRoutePlugin>)?;
            let mut uri_parts = req.uri().clone().into_parts();
            let path = req.uri().path();
            let mut new_pq = modifier()
                .replace(path, Some(prefix_match.as_str()))
                .ok_or_else(|| Response::with_code_message(StatusCode::BAD_GATEWAY, "gateway internal error: fail to rewrite path."))?;
            if let Some(query) = req.uri().query().filter(|q| !q.is_empty()) {
                new_pq.push('?');
                new_pq.push_str(query);
            }
            uri_parts.authority = Some(Authority::from_maybe_shared(domain).map_err(PluginError::bad_gateway::<OpresDynamicRoutePlugin>)?);
            uri_parts.path_and_query = Some(PathAndQuery::from_maybe_shared(new_pq).map_err(PluginError::bad_gateway::<OpresDynamicRoutePlugin>)?);
            *req.uri_mut() = Uri::from_parts(uri_parts).map_err(PluginError::bad_gateway::<OpresDynamicRoutePlugin>)?;
            Ok(req)
        };
        Box::pin(task)
    }
}

impl MakeSgLayer for OpresDynamicRoute {
    fn make_layer(&self) -> spacegate_shell::kernel::BoxResult<spacegate_shell::SgBoxLayer> {
        Ok(SgBoxLayer::new(AsyncFilterRequestLayer::new(self.clone())))
    }
}

def_plugin!("opres-dynamic-route", OpresDynamicRoutePlugin, OpresDynamicRoute);
