use serde::{Deserialize, Serialize};
use tardis::web::poem_openapi;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, poem_openapi::Enum)]
#[serde(rename_all = "lowercase")]
pub enum InvokeModuleKind {
    #[oai(rename = "search")]
    Search,
    #[oai(rename = "plugin")]
    Plugin,
    #[oai(rename = "kv")]
    Kv,
    #[oai(rename = "log")]
    Log,
    #[oai(rename = "object")]
    Object,
    #[oai(rename = "cache")]
    Cache,
    #[oai(rename = "graph")]
    Graph,
    #[oai(rename = "stats")]
    Stats,
    #[oai(rename = "schedule")]
    Schedule,
    #[oai(rename = "iam")]
    Iam,
    #[oai(rename = "event")]
    Event,
}

impl std::fmt::Display for InvokeModuleKind {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            InvokeModuleKind::Search => write!(f, "search"),
            InvokeModuleKind::Plugin => write!(f, "Plugin"),
            InvokeModuleKind::Kv => write!(f, "kv"),
            InvokeModuleKind::Log => write!(f, "log"),
            InvokeModuleKind::Object => write!(f, "object"),
            InvokeModuleKind::Cache => write!(f, "cache"),
            InvokeModuleKind::Graph => write!(f, "graph"),
            InvokeModuleKind::Stats => write!(f, "stats"),
            InvokeModuleKind::Schedule => write!(f, "schedule"),
            InvokeModuleKind::Iam => write!(f, "iam"),
            InvokeModuleKind::Event => write!(f, "event"),
        }
    }
}
