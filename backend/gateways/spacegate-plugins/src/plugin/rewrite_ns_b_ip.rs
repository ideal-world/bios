use ipnet::IpNet;
use serde::{Deserialize, Serialize};

use spacegate_shell::extension::k8s_service::K8sService;
use spacegate_shell::hyper::{http::uri, Response};
use spacegate_shell::kernel::extension::OriginalIpAddr;
use spacegate_shell::kernel::helper_layers::function::Inner;
use spacegate_shell::kernel::SgRequest;
use spacegate_shell::plugin::{schemars, Plugin, PluginConfig};
use spacegate_shell::{BoxError, SgBody, SgRequestExt};
use std::net::IpAddr;
use std::str::FromStr;
use std::sync::Arc;
use tardis::log::{trace, warn};

use tardis::{log, serde_json};

/// Kube available only!
#[derive(Clone)]
pub struct RewriteNsPlugin {
    pub ip_list: Arc<[IpNet]>,
    pub target_ns: String,
}

#[derive(Serialize, Deserialize, schemars::JsonSchema)]
#[serde(default)]
pub struct RewriteNsConfig {
    pub ip_list: Vec<String>,
    pub target_ns: String,
}

impl<'de> Deserialize<'de> for RewriteNsPlugin {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        RewriteNsConfig::deserialize(deserializer).map(|config| {
            let ip_list: Vec<IpNet> = config
                .ip_list
                .iter()
                .filter_map(|p| {
                    p.parse()
                        .or(p.parse::<IpAddr>().map(IpNet::from))
                        .map_err(|e| {
                            log::warn!("Cannot parse ip `{p}` when loading config: {e}");
                        })
                        .ok()
                })
                .collect();
            RewriteNsPlugin {
                ip_list: ip_list.into(),
                target_ns: config.target_ns,
            }
        })
    }
}

impl Default for RewriteNsConfig {
    fn default() -> Self {
        RewriteNsConfig {
            ip_list: vec![],
            target_ns: "default".to_string(),
        }
    }
}

impl Plugin for RewriteNsPlugin {
    const CODE: &'static str = "rewrite-ns";

    fn meta() -> spacegate_shell::model::PluginMetaData {
        spacegate_shell::model::plugin_meta!(
            description: "Rewrite namespace for request.Kubernetes available only!"
        )
    }

    fn create(plugin_config: PluginConfig) -> Result<Self, spacegate_shell::BoxError> {
        let config: RewriteNsConfig = serde_json::from_value(plugin_config.spec)?;
        let ip_list: Vec<IpNet> = config
            .ip_list
            .iter()
            .filter_map(|p| {
                p.parse()
                    .or(p.parse::<IpAddr>().map(IpNet::from))
                    .map_err(|e| {
                        log::warn!("Cannot parse ip `{p}` when loading config: {e}");
                    })
                    .ok()
            })
            .collect();
        Ok(RewriteNsPlugin {
            ip_list: ip_list.into(),
            target_ns: config.target_ns,
        })
    }
    async fn call(&self, mut req: SgRequest, inner: Inner) -> Result<Response<SgBody>, BoxError> {
        self.req(&mut req)?;
        Ok(inner.call(req).await)
    }
}
impl RewriteNsPlugin {
    fn req(&self, req: &mut SgRequest) -> Result<(), BoxError> {
        let original_ip = req
                    .extract::<OriginalIpAddr>().into_inner();
        if self.ip_list.iter().any(|ipnet| ipnet.contains(&original_ip)) {
            let defer = req.extensions_mut().get_or_insert_default::<spacegate_shell::kernel::extension::Defer>();
            let target_ns = self.target_ns.clone();
            defer.push_back(move |mut req| {
                if let Some(k8s_service) = req.extensions().get::<K8sService>().cloned() {
                    let k8s_service = k8s_service.0;
                    let Some(ref ns) = k8s_service.namespace else {
                        return req;
                    };
                    let uri = req.uri().clone();
                    let mut parts = uri.into_parts();
                    let new_authority = if let Some(prev_port) = parts.authority.as_ref().and_then(|a| a.port_u16()) {
                        format!("{svc}.{ns}:{port}", svc = k8s_service.name, ns = target_ns, port = prev_port)
                    } else {
                        format!("{svc}.{ns}", svc = k8s_service.name, ns = target_ns)
                    };
                    let Ok(new_authority) = uri::Authority::from_str(&new_authority) else {
                        warn!("Failed to rewrite ns: invalid url");
                        return req;
                    };
                    parts.authority.replace(new_authority);
                    let Ok(uri) = uri::Uri::from_parts(parts) else {
                        warn!("Failed to rewrite ns: invalid url");
                        return req;
                    };
                    *req.uri_mut() = uri;
                    log::debug!("change namespace from {} to {}", ns, target_ns);
                } else {
                    trace!("No k8s service found, skip rewrite ns");
                }
                req
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {

    use http::{Method, Request, Uri, Version};
    use spacegate_shell::{
        config::K8sServiceData,
        extension::k8s_service::K8sService,
        kernel::extension::PeerAddr,
        model::{PluginInstanceId, PluginInstanceName},
        plugin::{Plugin as _, PluginConfig},
        SgBody,
    };
    use tardis::{serde_json::json, tokio};

    use crate::plugin::rewrite_ns_b_ip::RewriteNsPlugin;

    #[tokio::test]
    async fn test() {
        let plugin = RewriteNsPlugin::create(PluginConfig {
            id: PluginInstanceId {
                code: "rewrite-ns".into(),
                name: PluginInstanceName::mono(),
            },
            spec: json!({"ip_list":["198.168.1.0/24"],"target_ns":"target"}),
        })
        .unwrap();

        let mut req = Request::builder()
            .method(Method::POST)
            .uri(Uri::from_static("http://sg.idealworld:80/test"))
            .version(Version::HTTP_11)
            .extension(K8sService(
                K8sServiceData {
                    name: "sg".to_string(),
                    namespace: Some("idealworld".to_string()),
                }
                .into(),
            ))
            .extension(PeerAddr("198.168.1.1:8080".parse().unwrap()))
            .body(SgBody::full("test"))
            .unwrap();

        plugin.req(&mut req).unwrap();

        assert_eq!(req.uri().host(), Some("sg.target"));
    }
}
