use serde::{Deserialize, Serialize};
use tardis::{db::sea_orm, derive_more::Display, web::poem_openapi};

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
}
