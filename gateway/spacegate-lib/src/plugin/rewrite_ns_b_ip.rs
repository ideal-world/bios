use ipnet::IpNet;
use serde::{Deserialize, Serialize};
use spacegate_kernel::def_filter;
use spacegate_kernel::plugins::context::SgRoutePluginContext;
use spacegate_kernel::plugins::filters::{SgPluginFilter, SgPluginFilterInitDto};
use std::net::IpAddr;
use tardis::async_trait::async_trait;
use tardis::basic::result::TardisResult;
use tardis::serde_json;

def_filter!("rewrite_ns", RewriteNsDef, RewriteNs);

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct RewriteNs {
    pub ip_list: Vec<String>,
    #[serde(skip)]
    pub ip_net: Vec<IpNet>,
}

impl Default for RewriteNs {
    fn default() -> Self {
        RewriteNs { ip_list: vec![], ip_net: vec![] }
    }
}

#[async_trait]
impl SgPluginFilter for RewriteNs {
    async fn init(&mut self, init_dto: &SgPluginFilterInitDto) -> TardisResult<()> {
        let ip_net = self.ip_list.iter().filter_map(|ip| ip.parse::<IpNet>().or(ip.parse::<IpAddr>().map(IpNet::from)).ok()).collect();
        self.ip_net = ip_net;
        Ok(())
    }

    async fn destroy(&self) -> TardisResult<()> {
        todo!()
    }

    async fn req_filter(&self, id: &str, ctx: SgRoutePluginContext) -> TardisResult<(bool, SgRoutePluginContext)> {
        todo!()
    }

    async fn resp_filter(&self, id: &str, ctx: SgRoutePluginContext) -> TardisResult<(bool, SgRoutePluginContext)> {
        todo!()
    }
}

#[cfg(test)]
mod test {
    use ipnet::IpNet;
    use std::net::IpAddr;

    #[test]
    fn test() {
        let a = "192.168.31.1".parse::<IpAddr>().map(IpNet::from).unwrap();
        println!("{a:?}")
    }
}
