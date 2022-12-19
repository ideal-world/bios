use tardis::TardisFuns;
use tardis::TardisFunsInst;

pub const SPI_CERT_KIND: &str = "spi";
pub const SPI_IDENT_REL_TAG: &str = "spi_ident";

#[cfg(feature = "default")]
pub fn get_tardis_inst_from_req(web: &tardis::web::poem::Request) -> TardisFunsInst {
    let serv_domain = web.uri().path().split('/').collect::<Vec<&str>>()[0];
    TardisFuns::inst_with_db_conn(serv_domain.to_string(), None)
}
