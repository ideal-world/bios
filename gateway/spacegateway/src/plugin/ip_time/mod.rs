use std::net::IpAddr;

use ipnet::IpNet;
use serde::{Deserialize, Serialize};
use spacegate_kernel::plugins::{
    context::SgRoutePluginContext,
    filters::{BoxSgPluginFilter, SgPluginFilter, SgPluginFilterDef},
};
use spacegate_kernel::{functions::http_route::SgHttpRouteMatchInst, plugins::filters::SgPluginFilterInitDto};

use tardis::{async_trait::async_trait, basic::result::TardisResult, log, serde_json, TardisFuns};
pub const CODE: &str = "ip_time";
pub struct SgFilterIpTimeDef;

mod ip_time_rule;
#[cfg(test)]
mod tests;
pub use ip_time_rule::IpTimeRule;

impl SgPluginFilterDef for SgFilterIpTimeDef {
    fn inst(&self, spec: serde_json::Value) -> TardisResult<BoxSgPluginFilter> {
        let config = TardisFuns::json.json_to_obj::<SgFilterIpTimeConfig>(spec)?;
        let filter: SgFilterIpTime = config.into();
        Ok(filter.boxed())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SgFilterIpTimeConfig {
    pub rules: Vec<SgFilterIpTimeConfigRule>,
}

impl From<SgFilterIpTimeConfig> for SgFilterIpTime {
    fn from(value: SgFilterIpTimeConfig) -> Self {
        let mut rules = Vec::new();
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
        SgFilterIpTime { rules }
    }
}
#[derive(Debug, Serialize, Deserialize)]
pub struct SgFilterIpTimeConfigRule {
    pub ip_list: Vec<String>,
    pub time_rule: IpTimeRule,
}

#[derive(Debug)]
pub struct SgFilterIpTime {
    /// # enhancement:
    /// should be a time segment list
    /// - segment list
    ///     - ban: Set {}
    ///     - allow: Set {}
    /// - pointer to the lastest segment
    pub rules: Vec<(IpNet, IpTimeRule)>,
}

#[async_trait]
impl SgPluginFilter for SgFilterIpTime {
    async fn init(&mut self, _http_route_rule: &SgPluginFilterInitDto) -> TardisResult<()> {
        return Ok(());
    }

    async fn destroy(&self) -> TardisResult<()> {
        return Ok(());
    }

    /// white list is prior
    async fn req_filter(&self, _id: &str, ctx: SgRoutePluginContext, _matched_match_inst: Option<&SgHttpRouteMatchInst>) -> TardisResult<(bool, SgRoutePluginContext)> {
        let socket_addr = ctx.get_req_remote_addr();
        let ip = socket_addr.ip();
        let pass = self.rules.iter().any(|(net, rule)| net.contains(&ip) && rule.check_by_now());
        Ok((pass, ctx))
    }

    async fn resp_filter(&self, _id: &str, ctx: SgRoutePluginContext, _matched_match_inst: Option<&SgHttpRouteMatchInst>) -> TardisResult<(bool, SgRoutePluginContext)> {
        return Ok((true, ctx));
    }
}
