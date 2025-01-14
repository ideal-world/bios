use http::StatusCode;
use serde::{Deserialize, Serialize};
use spacegate_shell::hyper::body::Body;
use spacegate_shell::plugin::{
    plugin_meta,
    schemars::{self, JsonSchema},
};
use spacegate_shell::plugin::{schema, Plugin, PluginSchemaExt};
use spacegate_shell::{BoxError, SgResponse, SgResponseExt};
use std::ops::Deref;
use std::str::FromStr;
use std::{fmt::Display, sync::Arc};
use tardis::regex::bytes::Regex as BytesRegex;
use tardis::serde_json;

use crate::extension::notification::ContentFilterForbiddenReport;

#[derive(Debug, Clone)]
pub enum BytesFilter {
    Regex(BytesRegex),
}

impl JsonSchema for BytesFilter {
    fn schema_name() -> String {
        String::schema_name()
    }

    fn json_schema(gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        String::json_schema(gen)
    }

    fn schema_id() -> std::borrow::Cow<'static, str> {
        String::schema_id()
    }

    fn is_referenceable() -> bool {
        String::is_referenceable()
    }
}

impl Display for BytesFilter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BytesFilter::Regex(regex) => write!(f, "{}", regex.as_str()),
        }
    }
}

impl FromStr for BytesFilter {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        BytesRegex::new(s).map(BytesFilter::Regex).map_err(|e| e.to_string())
    }
}
impl Serialize for BytesFilter {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            BytesFilter::Regex(re) => serializer.serialize_str(re.as_str()),
        }
    }
}

impl<'de> Deserialize<'de> for BytesFilter {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        BytesRegex::new(&s).map(BytesFilter::Regex).map_err(serde::de::Error::custom)
    }
}

impl BytesFilter {
    pub fn matches(&self, bytes: &[u8]) -> bool {
        match self {
            BytesFilter::Regex(re) => re.is_match(bytes),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, JsonSchema)]
pub struct ContentFilterConfig {
    content_length_limit: Option<u32>,
    forbidden_pq_filter: Vec<BytesFilter>,
    forbidden_content_filter: Vec<BytesFilter>,
}
#[derive(Debug, Clone)]
pub struct ContentFilterPlugin(Arc<ContentFilterConfig>);
impl Deref for ContentFilterPlugin {
    type Target = ContentFilterConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl Plugin for ContentFilterPlugin {
    const CODE: &'static str = "content-filter";
    fn meta() -> spacegate_shell::model::PluginMetaData {
        plugin_meta! {
            description: "Filter content based on type, keywords and length."
        }
    }

    async fn call(&self, mut req: spacegate_shell::SgRequest, inner: spacegate_shell::plugin::Inner) -> Result<SgResponse, BoxError> {
        if let Some(length_limit) = self.content_length_limit {
            let size = req.body().size_hint();
            if size.lower() > length_limit as u64 {
                return Ok(SgResponse::with_code_empty(StatusCode::PAYLOAD_TOO_LARGE));
            }
        }
        if !self.forbidden_pq_filter.is_empty() {
            if let Some(pq) = req.uri().path_and_query() {
                for f in &self.forbidden_pq_filter {
                    if f.matches(pq.as_str().as_bytes()) {
                        let mut response = SgResponse::with_code_empty(StatusCode::BAD_REQUEST);
                        response.extensions_mut().insert(ContentFilterForbiddenReport { forbidden_reason: format!("forbidden rule matched: {f}") });
                        return Ok(response);
                    }
                }
            }
        }
        if !self.forbidden_content_filter.is_empty() {
            let (parts, body) = req.into_parts();
            let body = body.dump().await?;
            for filter in &self.forbidden_content_filter {
                let bytes = body.get_dumped().expect("dumped");
                if filter.matches(bytes) {
                    let mut response = SgResponse::with_code_empty(StatusCode::BAD_REQUEST);
                    response.extensions_mut().insert(ContentFilterForbiddenReport {
                        forbidden_reason: format!("forbidden rule matched: {filter}") ,
                    });
                    return Ok(response);
                }
            }
            req = spacegate_shell::SgRequest::from_parts(parts, body);
        }
        Ok(inner.call(req).await)
    }

    fn create(plugin_config: spacegate_shell::model::PluginConfig) -> Result<Self, spacegate_shell::plugin::BoxError> {
        let config = serde_json::from_value(plugin_config.spec)?;
        Ok(ContentFilterPlugin(Arc::new(config)))
    }

    fn schema_opt() -> Option<schemars::schema::RootSchema> {
        Some(ContentFilterPlugin::schema())
    }
}

schema!(ContentFilterPlugin, ContentFilterConfig);
