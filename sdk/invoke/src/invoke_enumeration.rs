use serde::{Deserialize, Serialize};
#[cfg(feature = "reldb-core")]
use tardis::db::sea_orm;
use tardis::{derive_more::Display, web::poem_openapi};

#[cfg(feature = "reldb-core")]
#[derive(Display, Clone, Debug, PartialEq, Eq, Deserialize, Serialize, poem_openapi::Enum, sea_orm::strum::EnumString)]
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
    Iam
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
}
