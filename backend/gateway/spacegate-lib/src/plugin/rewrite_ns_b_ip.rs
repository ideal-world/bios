use ipnet::IpNet;
use serde::{Deserialize, Serialize};

use spacegate_shell::extension::k8s_service::K8sService;
use spacegate_shell::hyper::{http::uri, Response};
use spacegate_shell::kernel::extension::PeerAddr;
use spacegate_shell::kernel::helper_layers::function::Inner;
use spacegate_shell::kernel::SgRequest;
use spacegate_shell::plugin::{Plugin, PluginConfig, PluginError};
use spacegate_shell::{BoxError, SgBody};
use std::net::IpAddr;
use std::str::FromStr;
use std::sync::Arc;

use tardis::{log, serde_json};

/// Kube available only!
#[derive(Clone)]
pub struct RewriteNsPlugin {
    pub ip_list: Arc<[IpNet]>,
    pub target_ns: String,
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
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
        plugin::{Plugin as _, PluginConfig},
        SgBody,
    };
    use tardis::{serde_json::json, tokio};

    use crate::plugin::rewrite_ns_b_ip::RewriteNsPlugin;

    #[tokio::test]
    async fn test() {
        let plugin = RewriteNsPlugin::create(PluginConfig {
            code: "rewrite-ns".into(),
            spec: json!({"ip_list":["198.168.1.0/24"],"target_ns":"target"}),
            name: None,
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
