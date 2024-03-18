use std::net::IpAddr;
use std::sync::Arc;

use ipnet::IpNet;
use serde::{Deserialize, Serialize};
use spacegate_shell::hyper::{Request, Response, StatusCode};
use spacegate_shell::kernel::{
    extension::PeerAddr,
    helper_layers::filter::{Filter, FilterRequestLayer},
};
use spacegate_shell::plugin::{JsonValue, MakeSgLayer, Plugin, SerdeJsonError};

use spacegate_shell::{BoxError, SgBody, SgBoxLayer, SgResponseExt};

use tardis::{log, serde_json};
pub const CODE: &str = "ip_time";

mod ip_time_rule;
#[cfg(test)]
mod tests;
pub use ip_time_rule::IpTimeRule;

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct SgFilterIpTimeConfig {
    /// ## When white_list_mode is **enabled**
    /// some rules passed, the request will be allowed
    /// ## When white_list_mode is **disabled**
    /// only when some rules blocked, the request will be blocked
    pub mode: SgFilterIpTimeMode,
    pub rules: Vec<SgFilterIpTimeConfigRule>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum SgFilterIpTimeMode {
    WhiteList,
    #[default]
    BlackList,
}

impl From<SgFilterIpTimeConfig> for SgFilterIpTime {
    fn from(value: SgFilterIpTimeConfig) -> Self {
        let mut rules = Vec::new();
        let white_list_mode = value.mode;
        for rule in value.rules {
            let nets: Vec<IpNet> = rule
                .ip_list
                .iter()
                .filter_map(|p| {
                    p.parse()
                        .or(p.parse::<IpAddr>().map(IpNet::from))
                        .map_err(|e| {
                            log::warn!("[{CODE}] Cannot parse ip `{p}` when loading config: {e}");
                        })
                        .ok()
                })
                .collect();
            for net in IpNet::aggregate(&nets) {
                rules.push((net, rule.time_rule.clone()))
            }
        }
        SgFilterIpTime {
            mode: white_list_mode,
            rules: rules.into(),
        }
    }
}
#[derive(Debug, Serialize, Deserialize)]
pub struct SgFilterIpTimeConfigRule {
    pub ip_list: Vec<String>,
    pub time_rule: IpTimeRule,
}

#[derive(Debug, Clone)]
pub struct SgFilterIpTime {
    // # enhancement:
    // should be a time segment list
    // - segment list
    //     - ban: Set {}
    //     - allow: Set {}
    // - pointer to the lastest segment
    pub mode: SgFilterIpTimeMode,
    pub rules: Arc<[(IpNet, IpTimeRule)]>,
}

impl SgFilterIpTime {
    pub fn check_ip(&self, ip: &IpAddr) -> bool {
        match self.mode {
            SgFilterIpTimeMode::WhiteList => {
                // when white list mode is enabled, if we find some rule passed, the request will be allowed, otherwise blocked
                self.rules.iter().any(|(net, rule)| net.contains(ip) && rule.check_by_now())
            }
            SgFilterIpTimeMode::BlackList => {
                // when black list mode is enabled, if we find some rule blocked, the request will be blocked, otherwise allowed
                !self.rules.iter().any(|(net, rule)| net.contains(ip) && !rule.check_by_now())
            }
        }
    }
}

impl Filter for SgFilterIpTime {
    fn filter(&self, req: Request<SgBody>) -> Result<Request<SgBody>, Response<SgBody>> {
        let Some(socket_addr) = req.extensions().get::<PeerAddr>() else {
            return Err(Response::with_code_message(StatusCode::BAD_GATEWAY, "Cannot get peer address, it's a implementation bug"));
        };
        let socket_addr = socket_addr.0;
        let passed = self.check_ip(&socket_addr.ip());
        log::trace!("[{CODE}] Check ip time rule from {socket_addr}, passed {passed}");
        if !passed {
            return Err(Response::with_code_message(StatusCode::FORBIDDEN, "[SG.Plugin.IpTime] Blocked by ip-time plugin"));
        }
        Ok(req)
    }
}

pub struct SgIpTimePlugin;

impl Plugin for SgIpTimePlugin {
    const CODE: &'static str = CODE;
    type MakeLayer = SgFilterIpTime;
    type Error = SerdeJsonError;
    fn create(value: JsonValue) -> Result<Self::MakeLayer, Self::Error> {
        let config: SgFilterIpTimeConfig = serde_json::from_value(value)?;
        let filter: SgFilterIpTime = config.into();
        Ok(filter)
    }
}

impl MakeSgLayer for SgFilterIpTime {
    fn make_layer(&self) -> Result<SgBoxLayer, BoxError> {
        let layer = FilterRequestLayer::new(self.clone());
        Ok(SgBoxLayer::new(layer))
    }
}
