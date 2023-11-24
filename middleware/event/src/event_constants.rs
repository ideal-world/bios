use tardis::{TardisFunsInst, TardisFuns};

pub const DOMAIN_CODE: &str = "event";
pub const KIND_CODE: &str = "event";

pub fn get_tardis_inst() -> TardisFunsInst {
    TardisFuns::inst_with_db_conn(DOMAIN_CODE.to_string(), None)
}
