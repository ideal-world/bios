use std::fmt;

use async_trait::async_trait;

use serde::{Deserialize, Serialize};
use spacegate_kernel::http;
use spacegate_kernel::plugins::filters::SgPluginFilterInitDto;
use spacegate_kernel::plugins::{
    context::SgRoutePluginContext,
    filters::{BoxSgPluginFilter, SgPluginFilter, SgPluginFilterAccept, SgPluginFilterDef},
};

use tardis::{
    async_trait,
    basic::result::TardisResult,
    serde_json::{self},
    TardisFuns,
};

macro_rules! append_value {
    ($result:expr, $field:expr, $value:expr) => {
        if let Some(val) = $value {
            $result.push_str(&format!("{} {};", $field, val));
        }
    };
}
pub const CODE: &str = "anti_xss";
pub struct SgFilterAntiXSSDef;

impl SgPluginFilterDef for SgFilterAntiXSSDef {
    fn get_code(&self) -> &str {
        CODE
    }
    fn inst(&self, spec: serde_json::Value) -> TardisResult<BoxSgPluginFilter> {
        let filter = TardisFuns::json.json_to_obj::<SgFilterAntiXSS>(spec)?;
        Ok(filter.boxed())
    }

    fn get_code(&self) -> &str {
        CODE
    }
}

#[derive(Serialize, Deserialize)]
#[serde(default)]
#[derive(Default)]
pub struct SgFilterAntiXSS {
    csp_config: CSPConfig,
}

#[derive(Serialize, Deserialize)]
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
        append_value!(result, "base-uri", &self.base_uri);
        append_value!(result, "child-src", &self.child_src);
        append_value!(result, "connect-src", &self.connect_src);
        append_value!(result, "font-src", &self.font_src);
        append_value!(result, "form-action", &self.form_action);
        append_value!(result, "frame-ancestors", &self.frame_ancestors);
        append_value!(result, "frame-src", &self.frame_src);
        append_value!(result, "img-src", &self.img_src);
        append_value!(result, "manifest-src", &self.manifest_src);
        append_value!(result, "media-src", &self.media_src);
        append_value!(result, "object-src", &self.object_src);
        append_value!(result, "sandbox", &self.sandbox);
        append_value!(result, "script-src", &self.script_src);
        append_value!(result, "script-src-attr", &self.script_src_attr);
        append_value!(result, "script-src-elem", &self.script_src_elem);
        append_value!(result, "strict-dynamic", &self.strict_dynamic);
        append_value!(result, "style-src", &self.style_src);
        append_value!(result, "style-src-attr", &self.style_src_attr);
        append_value!(result, "style-src-elem", &self.style_src_elem);
        append_value!(result, "worker-src", &self.worker_src);
        append_value!(result, "report-to", &self.report_to);

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

#[derive(Default, Serialize, Deserialize)]
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
        match self {
            SandBoxValue::None => write!(f, ""),
            SandBoxValue::AllowForms => write!(f, "allow-forms"),
            SandBoxValue::AllowModals => write!(f, "allow-modals"),
            SandBoxValue::AllowOrientationLock => write!(f, "allow-orientation-lock"),
            SandBoxValue::AllowPointerLock => write!(f, "allow-pointer-lock"),
            SandBoxValue::AllowPopups => write!(f, "allow-popups"),
            SandBoxValue::AllowPopupsToEscapeSandbox => write!(f, "allow-popups-to-escape-sandbox"),
            SandBoxValue::AllowPresentation => write!(f, "allow-presentation"),
            SandBoxValue::AllowSameOrigin => write!(f, "allow-same-origin"),
            SandBoxValue::AllowScripts => write!(f, "allow-scripts"),
            SandBoxValue::AllowTopNavigation => write!(f, "allow-top-navigation"),
        }
    }
}

#[async_trait]
impl SgPluginFilter for SgFilterAntiXSS {
    fn accept(&self) -> SgPluginFilterAccept {
        SgPluginFilterAccept::default()
    }

    async fn init(&mut self, _: &SgPluginFilterInitDto) -> TardisResult<()> {
        Ok(())
    }

    async fn destroy(&self) -> TardisResult<()> {
        Ok(())
    }

    async fn req_filter(&self, _: &str, ctx: SgRoutePluginContext) -> TardisResult<(bool, SgRoutePluginContext)> {
        Ok((true, ctx))
    }

    async fn resp_filter(&self, _: &str, mut ctx: SgRoutePluginContext) -> TardisResult<(bool, SgRoutePluginContext)> {
        let mut enable = false;
        if let Some(content_type) = ctx.response.get_headers().get(http::header::CONTENT_TYPE) {
            enable = content_type.eq("text/html") || content_type.eq("text/css") || content_type.eq("application/javascript") || content_type.eq("application/x-javascript");
        };
        if enable {
            if self.csp_config.report_only {
                let _ = ctx.response.set_header(http::header::CONTENT_SECURITY_POLICY_REPORT_ONLY, &self.csp_config.to_string_header_value());
            } else {
                let _ = ctx.response.set_header(http::header::CONTENT_SECURITY_POLICY, &self.csp_config.to_string_header_value());
            }
        }
        Ok((true, ctx))
    }
}
