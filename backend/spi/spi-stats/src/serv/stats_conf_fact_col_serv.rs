use crate::dto::stats_conf_dto::{StatsConfFactColAddReq, StatsConfFactColInfoResp, StatsConfFactColModifyReq};
use crate::stats_enumeration::StatsFactColKind;
use crate::stats_initializer;
use bios_basic::spi::spi_constants;
use bios_basic::spi::spi_funs::SpiBsInstExtractor;
use bios_basic::spi_dispatch_service;
use tardis::basic::result::TardisResult;
use tardis::web::web_resp::TardisPage;

use super::pg;

spi_dispatch_service! {
    @mgr: true,
    @init: stats_initializer::init_fun,
    @dispatch: {
        #[cfg(feature = "spi-pg")]
        spi_constants::SPI_PG_KIND_CODE => pg::stats_pg_conf_fact_col_serv,
    },
    @method: {
        add(fact_conf_key: &str, add_req: &StatsConfFactColAddReq) -> TardisResult<()>;
        modify(
            fact_conf_key: &str,
            fact_col_conf_key: &str,
            modify_req: &StatsConfFactColModifyReq
        ) -> TardisResult<()>;
        delete(
            fact_conf_key: &str,
            fact_col_conf_key: Option<&str>,
            rel_external_id: Option<String>,
            kind: Option<StatsFactColKind>
        ) -> TardisResult<()> ;
        find_by_fact_conf_key(
            fact_conf_key: &str
        ) -> TardisResult<Vec<StatsConfFactColInfoResp>>;
        paginate(
            fact_conf_key: Option<String>,
            fact_col_conf_key: Option<String>,
            dim_key: Option<String>,
            dim_group_key: Option<String>,
            show_name: Option<String>,
            rel_external_id: Option<String>,
            page_number: u32,
            page_size: u32,
            desc_by_create: Option<bool>,
            desc_by_update: Option<bool>
        ) -> TardisResult<TardisPage<StatsConfFactColInfoResp>>;
    }
}
