use tardis::TardisFuns;
use tardis::TardisFunsInst;

pub const COMPONENT_CODE: &str = "reldb";

pub fn get_tardis_inst() -> TardisFunsInst {
    TardisFuns::inst_with_db_conn(COMPONENT_CODE.to_string(), None)
}
