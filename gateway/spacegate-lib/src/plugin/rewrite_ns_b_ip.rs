use ipnet::IpNet;
use serde::{Deserialize, Serialize};
use spacegate_kernel::config::gateway_dto::SgProtocol;
use spacegate_kernel::def_filter;
use spacegate_kernel::plugins::context::{SgRouteFilterRequestAction, SgRoutePluginContext};
use spacegate_kernel::plugins::filters::{SgPluginFilter, SgPluginFilterInitDto};
use std::net::IpAddr;
use tardis::async_trait::async_trait;
use tardis::basic::error::TardisError;
use tardis::basic::result::TardisResult;
use tardis::{log, serde_json};

def_filter!("rewrite_ns", SgFilterRewriteNsDef, SgFilterRewriteNs);

/// Kube available only!
#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct SgFilterRewriteNs {
    pub ip_list: Vec<String>,
    pub target_ns: String,
    #[serde(skip)]
    pub ip_net: Vec<IpNet>,
}

impl Default for SgFilterRewriteNs {
    fn default() -> Self {
        SgFilterRewriteNs {
            ip_list: vec![],
            target_ns: "default".to_string(),
            ip_net: vec![],
        }
    }
}

#[async_trait]
impl SgPluginFilter for SgFilterRewriteNs {
    async fn init(&mut self, _: &SgPluginFilterInitDto) -> TardisResult<()> {
        let ip_net = self.ip_list.iter().filter_map(|ip| ip.parse::<IpNet>().or(ip.parse::<IpAddr>().map(IpNet::from)).ok()).collect();
        self.ip_net = ip_net;
        Ok(())
    }

    async fn destroy(&self) -> TardisResult<()> {
        Ok(())
    }

    async fn req_filter(&self, id: &str, mut ctx: SgRoutePluginContext) -> TardisResult<(bool, SgRoutePluginContext)> {
        if let Some(backend) = ctx.get_chose_backend() {
            if backend.namespace.is_some() {
                let ip = ctx.get_remote_addr().ip();
                if self.ip_net.iter().any(|ipnet| ipnet.contains(&IpNet::from(ip))) {
                    ctx.set_action(SgRouteFilterRequestAction::Redirect);
                    let scheme = backend.protocol.as_ref().unwrap_or(&SgProtocol::Http);
                    let host = format!("{}{}", backend.name_or_host, format_args!(".{}", self.target_ns));
                    let port =
                        if (backend.port == 0 || backend.port == 80) && scheme == &SgProtocol::Http || (backend.port == 0 || backend.port == 443) && scheme == &SgProtocol::Https {
                            "".to_string()
                        } else {
                            format!(":{}", backend.port)
                        };
                    let url = format!("{}://{}{}{}", scheme, host, port, ctx.request.get_uri().path_and_query().map(|p| p.as_str()).unwrap_or(""));
                    ctx.request.set_uri(url.parse().map_err(|e| TardisError::wrap(&format!("[SG.Filter.Auth.Rewrite_Ns({id})] parse url:{e}"), ""))?);
                    log::debug!("[SG.Filter.Auth.Rewrite_Ns({id})] change namespace to {}", self.target_ns);
                }
            }
        }
        return Ok((true, ctx));
    }

    async fn resp_filter(&self, _: &str, ctx: SgRoutePluginContext) -> TardisResult<(bool, SgRoutePluginContext)> {
        return Ok((true, ctx));
    }
}

#[cfg(test)]
mod test {
    use crate::plugin::rewrite_ns_b_ip::SgFilterRewriteNs;
    use spacegate_kernel::config::gateway_dto::SgParameters;
    use spacegate_kernel::http::{HeaderMap, Method, Uri, Version};
    use spacegate_kernel::hyper::Body;
    use spacegate_kernel::instance::SgBackendInst;
    use spacegate_kernel::plugins::context::SgRoutePluginContext;
    use spacegate_kernel::plugins::filters::{SgPluginFilter, SgPluginFilterInitDto};
    use tardis::tokio;

    #[tokio::test]
    async fn test() {
        let mut filter_rens = SgFilterRewriteNs {
            ip_list: vec!["198.168.1.0/24".to_string()],
            target_ns: "target".to_string(),
            ..Default::default()
        };

        filter_rens
            .init(&SgPluginFilterInitDto {
                gateway_name: "".to_string(),
                gateway_parameters: SgParameters {
                    redis_url: None,
                    log_level: None,
                    lang: None,
                    ignore_tls_verification: None,
                },
                http_route_rules: vec![],
                attached_level: spacegate_kernel::plugins::filters::SgAttachedLevel::Gateway,
            })
            .await
            .unwrap();

        let mut ctx = SgRoutePluginContext::new_http(
            Method::POST,
            Uri::from_static("http://sg.idealworld.group/test1"),
            Version::HTTP_11,
            HeaderMap::new(),
            Body::from("test"),
            "198.168.1.1:8080".parse().unwrap(),
            "".to_string(),
            None,
        );
        let back_inst = SgBackendInst {
            name_or_host: "test".to_string(),
            namespace: Some("Anamspace".to_string()),
            port: 80,
            ..Default::default()
        };
        ctx.set_chose_backend_inst(&back_inst);

        let (_, ctx) = filter_rens.req_filter("", ctx).await.unwrap();
        assert_eq!(ctx.request.uri.get().host().unwrap(), format!("test.target"))
    }
}
