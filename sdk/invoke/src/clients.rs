mod base_spi_client;
#[cfg(feature = "spi_kv")]
pub mod spi_kv_client;
#[cfg(feature = "spi_log")]
pub mod spi_log_client;
#[cfg(feature = "spi_search")]
pub mod spi_search_client;
