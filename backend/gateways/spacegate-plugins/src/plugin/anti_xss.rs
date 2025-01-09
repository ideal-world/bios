use std::fmt;

use serde::{Deserialize, Serialize};
use spacegate_shell::{
    hyper::{header, Response},
    kernel::helper_layers::function::Inner,
    plugin::{schemars, Plugin},
    BoxError, SgBody,
};

macro_rules! append_value {
    ($result:ident {$($field:literal: $value:expr ,)*}) => {
        $(if let Some(val) = $value {
            $result.push_str($field);
            $result.push(' ');
            $result.push_str(val.as_str());
        })*
    };
}

use tardis::serde_json;

spacegate_shell::plugin::schema!(AntiXssPlugin, CSPConfig);

#[derive(Default, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(default)]
pub struct AntiXssConfig {
    csp_config: CSPConfig,
}

#[derive(Serialize, Deserialize, schemars::JsonSchema)]
#[serde(default)]
pub struct CSPConfig {
    default_src: String,
    base_uri: Option<String>,
    child_src: Option<String>,
    connect_src: Option<String>,
    font_src: Option<String>,
    form_action: Option<String>,
    frame_ancestors: Option<String>,
    frame_src: Option<String>,
    img_src: Option<String>,
    manifest_src: Option<String>,
    media_src: Option<String>,
    object_src: Option<String>,
    sandbox: Option<SandBoxValue>,
    script_src: Option<String>,
    script_src_attr: Option<String>,
    script_src_elem: Option<String>,
    strict_dynamic: Option<String>,
    style_src: Option<String>,
    style_src_attr: Option<String>,
    style_src_elem: Option<String>,
    worker_src: Option<String>,
    report_only: bool,
    report_to: Option<String>,
}
impl CSPConfig {
    fn to_string_header_value(&self) -> String {
        let mut result = format!("default-src {};", self.default_src);
        append_value!(result {
            "base-uri": &self.base_uri,
            "child-src": &self.child_src,
            "connect-src": &self.connect_src,
            "font-src": &self.font_src,
            "form-action": &self.form_action,
            "frame-ancestors": &self.frame_ancestors,
            "frame-src": &self.frame_src,
            "img-src": &self.img_src,
            "manifest-src": &self.manifest_src,
            "media-src": &self.media_src,
            "object-src": &self.object_src,
            "sandbox": &self.sandbox,
            "script-src": &self.script_src,
            "script-src-attr": &self.script_src_attr,
            "script-src-elem": &self.script_src_elem,
            "strict-dynamic": &self.strict_dynamic,
            "style-src": &self.style_src,
            "style-src-attr": &self.style_src_attr,
            "style-src-elem": &self.style_src_elem,
            "worker-src": &self.worker_src,
            "report-to": &self.report_to,
        });
        result
    }
}
impl Default for CSPConfig {
    fn default() -> Self {
        Self {
            default_src: "'self'".to_string(),
            report_to: None,
            base_uri: None,
            child_src: None,
            script_src: None,
            img_src: None,
            report_only: false,
            connect_src: None,
            font_src: None,
            form_action: None,
            frame_ancestors: None,
            frame_src: None,
            manifest_src: None,
            media_src: None,
            object_src: None,
            sandbox: None,
            script_src_attr: None,
            script_src_elem: None,
            strict_dynamic: None,
            style_src: None,
            style_src_attr: None,
            style_src_elem: None,
            worker_src: None,
        }
    }
}

#[derive(Default, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SandBoxValue {
    #[default]
    None,
    AllowForms,
    AllowModals,
    AllowOrientationLock,
    AllowPointerLock,
    AllowPopups,
    AllowPopupsToEscapeSandbox,
    AllowPresentation,
    AllowSameOrigin,
    AllowScripts,
    AllowTopNavigation,
}

impl fmt::Display for SandBoxValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl SandBoxValue {
    pub fn as_str(&self) -> &'static str {
        match self {
            SandBoxValue::None => "",
            SandBoxValue::AllowForms => "allow-forms",
            SandBoxValue::AllowModals => "allow-modals",
            SandBoxValue::AllowOrientationLock => "allow-orientation-lock",
            SandBoxValue::AllowPointerLock => "allow-pointer-lock",
            SandBoxValue::AllowPopups => "allow-popups",
            SandBoxValue::AllowPopupsToEscapeSandbox => "allow-popups-to-escape-sandbox",
            SandBoxValue::AllowPresentation => "allow-presentation",
            SandBoxValue::AllowSameOrigin => "allow-same-origin",
            SandBoxValue::AllowScripts => "allow-scripts",
            SandBoxValue::AllowTopNavigation => "allow-top-navigation",
        }
    }
}

pub struct AntiXssPlugin {
    csp_config: CSPConfig,
    header: header::HeaderValue,
}

impl Plugin for AntiXssPlugin {
    const CODE: &'static str = "anti-xss";

    fn meta() -> spacegate_shell::model::PluginMetaData {
        spacegate_shell::model::plugin_meta!(
            description: "Anti XSS plugin"
        )
    }

    fn create(plugin_config: spacegate_shell::plugin::PluginConfig) -> Result<Self, BoxError> {
        let config: AntiXssConfig = serde_json::from_value(plugin_config.spec)?;
        let header = header::HeaderValue::from_str(&config.csp_config.to_string_header_value())?;
        Ok(AntiXssPlugin {
            csp_config: config.csp_config,
            header,
        })
    }
    async fn call(&self, req: http::Request<SgBody>, inner: Inner) -> Result<Response<SgBody>, BoxError> {
        let report_only = self.csp_config.report_only;
        let mut resp = inner.call(req).await;
        let header = &self.header;

        let mut enable = false;
        if let Some(content_type) = resp.headers().get(header::CONTENT_TYPE) {
            enable = content_type.eq("text/html") || content_type.eq("text/css") || content_type.eq("application/javascript") || content_type.eq("application/x-javascript");
        };
        if enable {
            if report_only {
                let _ = resp.headers_mut().append(header::CONTENT_SECURITY_POLICY_REPORT_ONLY, header.clone());
            } else {
                let _ = resp.headers_mut().append(header::CONTENT_SECURITY_POLICY, header.clone());
            }
        }
        Ok(resp)
    }
}
