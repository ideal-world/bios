use http::StatusCode;
use serde::{Deserialize, Serialize};
use spacegate_shell::hyper::{Request, Response};
use spacegate_shell::kernel::helper_layers::function::Inner;
use spacegate_shell::plugin::{schemars, Plugin};
use spacegate_shell::{BoxError, SgBody, SgResponseExt};
use std::sync::Arc;
use tardis::serde_json;

spacegate_shell::plugin::schema!(HttpMethodOverridePlugin, HttpMethodOverrideConfig);

#[derive(Serialize, Deserialize, schemars::JsonSchema)]
#[serde(default)]
pub struct HttpMethodOverrideConfig {
    /// Header name to check for method override (default: "X-HTTP-Method-Override")
    pub header_name: String,

    /// List of original methods that can be overridden
    /// Empty list = allow all methods (default: ["POST"])
    pub allowed_source_methods: Vec<String>,

    /// Paths to skip (prefix match)
    pub skip_paths: Vec<String>,
}

impl Default for HttpMethodOverrideConfig {
    fn default() -> Self {
        Self {
            header_name: "X-HTTP-Method-Override".to_string(),
            allowed_source_methods: vec!["POST".to_string()],
            skip_paths: Vec::new(),
        }
    }
}

#[derive(Clone)]
pub struct HttpMethodOverridePlugin {
    header_name: http::header::HeaderName,
    allowed_source_methods: Arc<Vec<String>>,
    skip_paths: Arc<Vec<String>>,
}

impl Plugin for HttpMethodOverridePlugin {
    const CODE: &'static str = "http-method-override";

    fn meta() -> spacegate_shell::model::PluginMetaData {
        spacegate_shell::model::plugin_meta!(
            description: "Override HTTP method via header (e.g., X-HTTP-Method-Override)"
        )
    }

    fn create(plugin_config: spacegate_shell::plugin::PluginConfig) -> Result<Self, BoxError> {
        let config: HttpMethodOverrideConfig = serde_json::from_value(plugin_config.spec)?;
        let header_name = http::header::HeaderName::from_bytes(config.header_name.as_bytes())?;

        // Normalize methods to uppercase
        let allowed_source_methods = config.allowed_source_methods.into_iter().map(|m| m.to_uppercase()).collect::<Vec<_>>();

        Ok(Self {
            header_name,
            allowed_source_methods: Arc::new(allowed_source_methods),
            skip_paths: Arc::new(config.skip_paths),
        })
    }

    async fn call(&self, mut req: Request<SgBody>, inner: Inner) -> Result<Response<SgBody>, BoxError> {
        // 1. Check skip_paths
        if self.should_skip_path(&req) {
            return Ok(inner.call(req).await);
        }

        // 2. Extract override header - if not present, skip all processing
        let Some(override_value) = req.headers().get(&self.header_name) else {
            return Ok(inner.call(req).await);
        };

        // 3. Check if source method is allowed to be overridden
        if !self.is_source_method_allowed(req.method()) {
            return Ok(inner.call(req).await);
        }

        // 4. Validate header value as UTF-8
        let method_str = match override_value.to_str() {
            Ok(s) => s,
            Err(_) => {
                return Ok(Response::with_code_message(
                    StatusCode::BAD_REQUEST,
                    format!("[SG.Plugin.HttpMethodOverride] Invalid header value encoding in {}", self.header_name),
                ));
            }
        };

        // 5. Parse as HTTP method
        let new_method = match http::Method::from_bytes(method_str.as_bytes()) {
            Ok(method) => method,
            Err(_) => {
                return Ok(Response::with_code_message(
                    StatusCode::BAD_REQUEST,
                    format!("[SG.Plugin.HttpMethodOverride] Invalid HTTP method in {}: {}", self.header_name, method_str),
                ));
            }
        };

        // 6. Override the method
        *req.method_mut() = new_method;

        // 7. Call inner handler with modified request
        Ok(inner.call(req).await)
    }
}

impl HttpMethodOverridePlugin {
    fn should_skip_path(&self, req: &Request<SgBody>) -> bool {
        if self.skip_paths.is_empty() {
            return false;
        }

        let path = req.uri().path();
        self.skip_paths.iter().any(|skip_path| path.starts_with(skip_path))
    }

    fn is_source_method_allowed(&self, method: &http::Method) -> bool {
        // Empty list = allow all
        if self.allowed_source_methods.is_empty() {
            return true;
        }

        let method_str = method.as_str();
        self.allowed_source_methods.iter().any(|allowed| allowed.eq_ignore_ascii_case(method_str))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use http::{Method, Request, Uri, Version};
    use spacegate_shell::{
        model::{PluginInstanceId, PluginInstanceName},
        plugin::PluginConfig,
        SgBody,
    };
    use tardis::serde_json::json;

    fn create_test_plugin(spec: serde_json::Value) -> HttpMethodOverridePlugin {
        HttpMethodOverridePlugin::create(PluginConfig {
            id: PluginInstanceId {
                code: "http-method-override".into(),
                name: PluginInstanceName::mono(),
            },
            spec,
        })
        .unwrap()
    }

    #[test]
    fn test_plugin_creation() {
        let plugin = create_test_plugin(json!({
            "header_name": "X-HTTP-Method-Override",
            "allowed_source_methods": ["POST"],
            "skip_paths": ["/health"]
        }));

        assert_eq!(plugin.header_name.as_str(), "x-http-method-override");
        assert_eq!(*plugin.allowed_source_methods, vec!["POST"]);
        assert_eq!(*plugin.skip_paths, vec!["/health"]);
    }

    #[test]
    fn test_should_skip_path() {
        let plugin = create_test_plugin(json!({
            "header_name": "X-HTTP-Method-Override",
            "allowed_source_methods": ["POST"],
            "skip_paths": ["/health", "/metrics"]
        }));

        let req = Request::builder().uri(Uri::from_static("http://example.com/health")).body(SgBody::empty()).unwrap();
        assert!(plugin.should_skip_path(&req));

        let req = Request::builder().uri(Uri::from_static("http://example.com/health/check")).body(SgBody::empty()).unwrap();
        assert!(plugin.should_skip_path(&req));

        let req = Request::builder().uri(Uri::from_static("http://example.com/api/test")).body(SgBody::empty()).unwrap();
        assert!(!plugin.should_skip_path(&req));
    }

    #[test]
    fn test_is_source_method_allowed_with_whitelist() {
        let plugin = create_test_plugin(json!({
            "header_name": "X-HTTP-Method-Override",
            "allowed_source_methods": ["POST", "PUT"],
            "skip_paths": []
        }));

        assert!(plugin.is_source_method_allowed(&Method::POST));
        assert!(plugin.is_source_method_allowed(&Method::PUT));
        assert!(!plugin.is_source_method_allowed(&Method::GET));
        assert!(!plugin.is_source_method_allowed(&Method::DELETE));
    }

    #[test]
    fn test_is_source_method_allowed_empty_list() {
        let plugin = create_test_plugin(json!({
            "header_name": "X-HTTP-Method-Override",
            "allowed_source_methods": [],
            "skip_paths": []
        }));

        // Empty list means allow all
        assert!(plugin.is_source_method_allowed(&Method::GET));
        assert!(plugin.is_source_method_allowed(&Method::POST));
        assert!(plugin.is_source_method_allowed(&Method::PUT));
        assert!(plugin.is_source_method_allowed(&Method::DELETE));
    }

    #[test]
    fn test_method_parsing() {
        // Valid standard methods
        assert!(http::Method::from_bytes(b"GET").is_ok());
        assert!(http::Method::from_bytes(b"POST").is_ok());
        assert!(http::Method::from_bytes(b"PUT").is_ok());
        assert!(http::Method::from_bytes(b"PATCH").is_ok());
        assert!(http::Method::from_bytes(b"DELETE").is_ok());
        assert!(http::Method::from_bytes(b"HEAD").is_ok());
        assert!(http::Method::from_bytes(b"OPTIONS").is_ok());

        // Case sensitivity - lowercase works
        assert!(http::Method::from_bytes(b"put").is_ok());
        assert!(http::Method::from_bytes(b"get").is_ok());

        // Extension methods are also valid per RFC 7231
        assert!(http::Method::from_bytes(b"CUSTOM").is_ok());

        // Invalid methods (non-token characters)
        assert!(http::Method::from_bytes(b"INV ALID").is_err()); // space
        assert!(http::Method::from_bytes(b"").is_err()); // empty
    }

    #[test]
    fn test_custom_header_name() {
        let plugin = create_test_plugin(json!({
            "header_name": "X-Method",
            "allowed_source_methods": ["POST"],
            "skip_paths": []
        }));

        assert_eq!(plugin.header_name.as_str(), "x-method");
    }

    #[test]
    fn test_default_config() {
        let plugin = create_test_plugin(json!({}));

        assert_eq!(plugin.header_name.as_str(), "x-http-method-override");
        assert_eq!(*plugin.allowed_source_methods, vec!["POST"]);
        assert!(plugin.skip_paths.is_empty());
    }
}
