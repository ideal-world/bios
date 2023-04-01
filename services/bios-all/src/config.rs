use bios_auth::auth_config::AuthConfig;
use bios_spi_cache::cache_config::CacheConfig;
use bios_spi_graph::graph_config::GraphConfig;
use bios_spi_kv::kv_config::KvConfig;
use bios_spi_log::log_config::LogConfig;
use bios_spi_object::object_config::ObjectConfig;
use bios_spi_reldb::reldb_config::ReldbConfig;
use bios_spi_search::search_config::SearchConfig;
use bios_spi_stats::stats_config::StatsConfig;
use tardis::serde::{Deserialize, Serialize};

use bios_iam::iam_config::IamConfig;

#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct BiosConfig {
    pub iam: IamConfig,
    pub auth: AuthConfig,
    pub cache: CacheConfig,
    pub graph: GraphConfig,
    pub kv: KvConfig,
    pub log: LogConfig,
    pub object: ObjectConfig,
    pub reldb: ReldbConfig,
    pub search: SearchConfig,
    pub stats: StatsConfig,
}
