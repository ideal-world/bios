use serde::{Deserialize, Serialize};
#[cfg(feature = "reldb-core")]
use tardis::db::sea_orm;
use tardis::web::poem_openapi;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, poem_openapi::Enum)]
#[cfg_attr(feature = "reldb-core", derive(strum::EnumString))]
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

#[cfg(not(feature = "reldb-core"))]
#[derive(Display, Clone, Debug, PartialEq, Eq, Deserialize, Serialize, poem_openapi::Enum)]
pub enum InvokeModuleKind {
    Search,
    Plugin,
    Kv,
    Log,
    Object,
    Cache,
    Graph,
    Stats,
    Schedule,
    Iam,
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
