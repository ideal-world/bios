use tardis::{TardisFunsInst, TardisFuns};
const DOMAIN_CODE: &str = "reach";

pub fn get_tardis_inst() -> TardisFunsInst {
    TardisFuns::inst_with_db_conn(DOMAIN_CODE.to_string(), None)
}