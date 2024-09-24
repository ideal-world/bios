pub const DOMAIN_CODE: &str = "spi-log";

pub const EVENT_ADD_LOG: &str = "spi-log/add";

//log表的flag
pub const TABLE_LOG_FLAG: &str = "log";
//pg v2 spi kind code
pub const SPI_PG_V2_KIND_CODE: &str = "spi-bs-pg-v2";
pub const TABLE_LOG_FLAG_V2: &str = "logv2";
//父表表名
pub const PARENT_TABLE_NAME: &str = "spi_log_parent";
//配置表名
pub const CONFIG_TABLE_NAME: &str = "spi_log_config";
//ref flag  __STARSYS_LOG_REF__@{ts}#{key}
pub const LOG_REF_FLAG: &str = "__STARSYS_LOG_REF__";
