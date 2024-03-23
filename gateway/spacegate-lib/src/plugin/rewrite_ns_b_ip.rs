use ipnet::IpNet;
use serde::{Deserialize, Serialize};

use spacegate_shell::extension::k8s_service::K8sService;
use spacegate_shell::hyper::Request;
use spacegate_shell::hyper::{http::uri, Response};
use spacegate_shell::kernel::extension::PeerAddr;
use spacegate_shell::plugin::{def_filter_plugin, Filter, PluginError};
use spacegate_shell::SgBody;
use std::net::IpAddr;
use std::str::FromStr;
use std::sync::Arc;

use tardis::log;

def_filter_plugin!("rewrite_ns", RewriteNsPlugin, SgFilterRewriteNs);

/// Kube available only!
#[derive(Clone)]
pub struct SgFilterRewriteNs {
    pub ip_list: Arc<[IpNet]>,
    pub target_ns: String,
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default)]
pub struct SgFilterRewriteNsConfig {
    pub ip_list: Vec<String>,
    pub target_ns: String,
}

impl<'de> Deserialize<'de> for SgFilterRewriteNs {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        SgFilterRewriteNsConfig::deserialize(deserializer).map(|config| {
            let ip_list: Vec<IpNet> = config
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
            SgFilterRewriteNs {
                ip_list: ip_list.into(),
                target_ns: config.target_ns,
            }
        })
    }
}

impl Default for SgFilterRewriteNsConfig {
    fn default() -> Self {
        SgFilterRewriteNsConfig {
            ip_list: vec![],
            target_ns: "default".to_string(),
        }
    }
}

impl Filter for SgFilterRewriteNs {
    fn filter(&self, mut req: Request<spacegate_shell::SgBody>) -> Result<Request<SgBody>, Response<SgBody>> {
        'change_ns: {
            if let Some(k8s_service) = req.extensions().get::<K8sService>().cloned() {
                let Some(ref ns) = k8s_service.0.namespace else { break 'change_ns };
                let ip = req.extensions().get::<PeerAddr>().expect("missing peer addr").0.ip();
                if self.ip_list.iter().any(|ipnet| ipnet.contains(&ip)) {
                    let uri = req.uri().clone();
                    let mut parts = uri.into_parts();
                    let new_authority = if let Some(prev_host) = parts.authority.as_ref().and_then(|a| a.port_u16()) {
                        format!("{svc}.{ns}:{port}", svc = k8s_service.0.name, ns = self.target_ns, port = prev_host)
                    } else {
                        format!("{svc}.{ns}", svc = k8s_service.0.name, ns = self.target_ns)
                    };
                    let new_authority = uri::Authority::from_str(&new_authority).map_err(PluginError::internal_error::<RewriteNsPlugin>)?;
                    parts.authority.replace(new_authority);
                    *req.uri_mut() = uri::Uri::from_parts(parts).map_err(PluginError::internal_error::<RewriteNsPlugin>)?;
                    log::debug!("[SG.Filter.Auth.Rewrite_Ns] change namespace from {} to {}", ns, self.target_ns);
                }
            }
        }
        Ok(req)
    }
}

// #[cfg(test)]
// mod test {
//     use crate::plugin::rewrite_ns_b_ip::SgFilterRewriteNs;
//     use spacegate_shell::config::gateway_dto::SgParameters;
//     use spacegate_shell::http::{HeaderMap, Method, Uri, Version};
//     use spacegate_shell::hyper::Body;
//     use spacegate_shell::instance::SgBackendInst;
//     use spacegate_shell::plugins::context::SgRoutePluginContext;
//     use spacegate_shell::plugins::filters::{SgPluginFilter, SgPluginFilterInitDto};
//     use tardis::tokio;

//     #[tokio::test]
//     async fn test() {
//         let mut filter_rens = SgFilterRewriteNs {
//             ip_list: vec!["198.168.1.0/24".to_string()],
//             target_ns: "target".to_string(),
//             ..Default::default()
//         };

//         filter_rens
//             .init(&SgPluginFilterInitDto {
//                 gateway_name: "".to_string(),
//                 gateway_parameters: SgParameters {
//                     redis_url: None,
//                     log_level: None,
//                     lang: None,
//                     ignore_tls_verification: None,
//                 },
//                 http_route_rules: vec![],
//                 attached_level: spacegate_shell::plugins::filters::SgAttachedLevel::Gateway,
//             })
//             .await
//             .unwrap();

//         let mut ctx = SgRoutePluginContext::new_http(
//             Method::POST,
//             Uri::from_static("http://sg.idealworld.group/test1"),
//             Version::HTTP_11,
//             HeaderMap::new(),
//             Body::from("test"),
//             "198.168.1.1:8080".parse().unwrap(),
//             "".to_string(),
//             None,
//         );
//         let back_inst = SgBackendInst {
//             name_or_host: "test".to_string(),
//             namespace: Some("Anamspace".to_string()),
//             port: 80,
//             ..Default::default()
//         };
//         ctx.set_chose_backend_inst(&back_inst);

//         let (_, ctx) = filter_rens.req_filter("", ctx).await.unwrap();
//         assert_eq!(ctx.request.uri.get().host().unwrap(), format!("test.target"))
//     }
// }
