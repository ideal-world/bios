use std::convert::Infallible;

use http::{Request, Response};
use serde::{Deserialize, Serialize};
use spacegate_shell::{
    hyper::service::Service,
    kernel::{
        helper_layers::{
            async_filter::AsyncFilterRequestLayer,
            check::{redis::RedisCheck, CheckLayer},
        },
        BoxHyperService, Layer,
    },
    plugin::{def_plugin, MakeSgLayer},
    SgBody, SgBoxLayer,
};

pub mod count_limit;
pub mod dynamic_route;
pub mod freq_limit;
pub mod time_limit;
#[cfg(feature = "schema")]
use spacegate_plugin::schemars;
#[cfg(feature = "schema")]
spacegate_plugin::schema!(OpresPlugin, OpresPluginConfig);
#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct OpresPluginConfig {
    prefix: String,
}

pub struct OpresPluginConfigLayer {
    count_limit: CheckLayer<RedisCheck, crate::marker::OpresKey>,
    freq_limit: CheckLayer<RedisCheck, crate::marker::OpresKey>,
    time_limit: CheckLayer<RedisCheck, crate::marker::OpresKey>,
    dynamic_route: AsyncFilterRequestLayer<crate::plugin::opres::dynamic_route::OpresDynamicRoute>,
}

impl<S> Layer<S> for OpresPluginConfigLayer
where
    S: Clone + Send + Sync + Service<Request<SgBody>, Response = Response<SgBody>, Error = Infallible> + 'static,
    <S as Service<Request<SgBody>>>::Future: Send,
{
    type Service = BoxHyperService;
    fn layer(&self, service: S) -> Self::Service {
        BoxHyperService::new(self.dynamic_route.layer(self.time_limit.layer(self.freq_limit.layer(self.count_limit.layer(service)))))
    }
}

impl OpresPluginConfig {
    pub fn create_layer(&self, gateway_name: &str) -> spacegate_shell::kernel::BoxResult<spacegate_shell::SgBoxLayer> {
        use count_limit::OpresCountLimitConfig;
        use dynamic_route::OpresDynamicRoute;
        use freq_limit::OpresFreqLimitConfig;
        use time_limit::OpresTimeLimitConfig;
        let count_limit_layer = CheckLayer::<_, crate::marker::OpresKey>::new(OpresCountLimitConfig { prefix: self.prefix.clone() }.create_check(gateway_name)?);
        let freq_limit_layer = CheckLayer::<_, crate::marker::OpresKey>::new(OpresFreqLimitConfig { prefix: self.prefix.clone() }.create_check(gateway_name)?);
        let time_limit_layer = CheckLayer::<_, crate::marker::OpresKey>::new(OpresTimeLimitConfig { prefix: self.prefix.clone() }.create_check(gateway_name)?);
        let dynamic_route_layer = AsyncFilterRequestLayer::new(OpresDynamicRoute { prefix: self.prefix.clone() });
        let layer = OpresPluginConfigLayer {
            count_limit: count_limit_layer,
            freq_limit: freq_limit_layer,
            time_limit: time_limit_layer,
            dynamic_route: dynamic_route_layer,
        };
        Ok(SgBoxLayer::new(layer))
    }
}

impl MakeSgLayer for OpresPluginConfig {
    fn make_layer(&self) -> spacegate_shell::kernel::BoxResult<spacegate_shell::SgBoxLayer> {
        self.create_layer("")
    }
    fn install_on_backend(&self, backend: &mut spacegate_shell::kernel::layers::http_route::builder::SgHttpBackendLayerBuilder) -> Result<(), spacegate_shell::BoxError> {
        let gateway_name = backend.extensions.get::<spacegate_shell::kernel::extension::GatewayName>().ok_or_else(|| spacegate_shell::BoxError::from("missing gateway name"))?;
        backend.plugins.push(self.create_layer(gateway_name.as_ref())?);
        Ok(())
    }
    fn install_on_gateway(&self, gateway: &mut spacegate_shell::kernel::layers::gateway::builder::SgGatewayLayerBuilder) -> Result<(), spacegate_shell::BoxError> {
        let gateway_name = gateway.extension.get::<spacegate_shell::kernel::extension::GatewayName>().ok_or_else(|| spacegate_shell::BoxError::from("missing gateway name"))?;
        gateway.http_plugins.push(self.create_layer(gateway_name.as_ref())?);
        Ok(())
    }
    fn install_on_route(&self, route: &mut spacegate_shell::kernel::layers::http_route::builder::SgHttpRouteLayerBuilder) -> Result<(), spacegate_shell::BoxError> {
        let gateway_name = route.extensions.get::<spacegate_shell::kernel::extension::GatewayName>().ok_or_else(|| spacegate_shell::BoxError::from("missing gateway name"))?;
        route.plugins.push(self.create_layer(gateway_name.as_ref())?);
        Ok(())
    }
    fn install_on_rule(&self, rule: &mut spacegate_shell::kernel::layers::http_route::builder::SgHttpRouteRuleLayerBuilder) -> Result<(), spacegate_shell::BoxError> {
        let gateway_name = rule.extensions.get::<spacegate_shell::kernel::extension::GatewayName>().ok_or_else(|| spacegate_shell::BoxError::from("missing gateway name"))?;
        rule.plugins.push(self.create_layer(gateway_name.as_ref())?);
        Ok(())
    }
}

def_plugin!("opres", OpresPlugin, OpresPluginConfig);
