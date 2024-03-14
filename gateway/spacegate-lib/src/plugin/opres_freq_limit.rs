pub struct OpresLimitCheck {}

use std::sync::Arc;

use serde::{Deserialize, Serialize};
use spacegate_shell::{
    kernel::{
        extension::GatewayName,
        helper_layers::check::{redis::RedisCheck, Check, CheckLayer, CheckService},
        BoxResult,
    },
    plugin::{def_plugin, MakeSgLayer},
    spacegate_ext_redis::{global_repo, redis::Script, RedisClient, RedisClientRepoError},
    SgBoxLayer,
};


#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct OpresFreqLimitConfig {
    prefix: String,
}

impl Default for OpresFreqLimitConfig {
    fn default() -> Self {
        Self { prefix: crate::consts::OP_RES_HEADER_DEFAULT.into() }
    }
}

impl OpresFreqLimitConfig {
    pub fn create_check(&self, gateway_name: &str) -> BoxResult<RedisCheck> {
        let check_script = Script::new(include_str!("./opres_freq_limit/check.lua"));
        let check = RedisCheck {
            check_script: Some(check_script.into()),
            response_script: None,
            key_prefix: <Arc<str>>::from(format!("{}:frequency", self.prefix)),
            client: global_repo().get(gateway_name).ok_or(RedisClientRepoError::new(gateway_name, "missing redis client"))?,
        };
        Ok(check)
    }
    pub fn make_layer_with_gateway_name(&self, gateway_name: &str) -> BoxResult<spacegate_shell::SgBoxLayer> {
        let layer = CheckLayer::<_, crate::marker::BiosAuth>::new(self.create_check(gateway_name.as_ref())?);
        Ok(SgBoxLayer::new(layer))
    }
}

impl MakeSgLayer for OpresFreqLimitConfig {
    fn make_layer(&self) -> BoxResult<spacegate_shell::SgBoxLayer> {
        self.make_layer_with_gateway_name("")
    }
    fn install_on_backend(&self, backend: &mut spacegate_shell::kernel::layers::http_route::builder::SgHttpBackendLayerBuilder) -> Result<(), spacegate_shell::BoxError> {
        let Some(gateway_name) = backend.extensions.get::<GatewayName>() else { return Ok(()) };
        backend.plugins.push(self.make_layer_with_gateway_name(gateway_name.as_ref())?);
        Ok(())
    }
    fn install_on_gateway(&self, gateway: &mut spacegate_shell::kernel::layers::gateway::builder::SgGatewayLayerBuilder) -> Result<(), spacegate_shell::BoxError> {
        let Some(gateway_name) = gateway.extension.get::<GatewayName>() else { return Ok(()) };
        gateway.http_plugins.push(self.make_layer_with_gateway_name(gateway_name.as_ref())?);
        Ok(())
    }
    fn install_on_route(&self, route: &mut spacegate_shell::kernel::layers::http_route::builder::SgHttpRouteLayerBuilder) -> Result<(), spacegate_shell::BoxError> {
        let Some(gateway_name) = route.extensions.get::<GatewayName>() else { return Ok(()) };
        route.plugins.push(self.make_layer_with_gateway_name(gateway_name.as_ref())?);
        Ok(())
    }
    fn install_on_rule(&self, rule: &mut spacegate_shell::kernel::layers::http_route::builder::SgHttpRouteRuleLayerBuilder) -> Result<(), spacegate_shell::BoxError> {
        let Some(gateway_name) = rule.extensions.get::<GatewayName>() else { return Ok(()) };
        rule.plugins.push(self.make_layer_with_gateway_name(gateway_name.as_ref())?);
        Ok(())
    }
}

def_plugin!("opres-freq-limit",  OpresFreqLimitPlugin, OpresFreqLimitConfig);