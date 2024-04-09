extern crate lazy_static;

pub mod enumeration;
pub mod dto;
pub mod helper;
pub mod process;
pub mod rbum;
pub mod spi;
#[cfg(feature = "test")]
pub mod test;

pub use enumeration::ApiTag;
use tardis::{TardisFuns, TardisFunsInst};

pub trait TardisFunInstExtractor {
    fn tardis_fun_inst(&self) -> TardisFunsInst;
}

#[cfg(feature = "default")]
impl TardisFunInstExtractor for tardis::web::poem::Request {
    fn tardis_fun_inst(&self) -> TardisFunsInst {
        let serv_domain = self.original_uri().path().split('/').collect::<Vec<&str>>()[1];
        TardisFuns::inst_with_db_conn(serv_domain.to_string(), None)
    }
}
