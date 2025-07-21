use crate::dto::stats_conf_dto::{StatsConfFactAddReq, StatsConfFactInfoResp, StatsConfFactModifyReq};
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
        spi_constants::SPI_PG_KIND_CODE => pg::stats_pg_conf_fact_serv,
    },
    @method: {
        add(add_req: &StatsConfFactAddReq) -> TardisResult<()>;
        modify(fact_conf_key: &str, modify_req: &StatsConfFactModifyReq) -> TardisResult<()>;
        delete(fact_conf_key: &str) -> TardisResult<()>;
        paginate(
            fact_conf_keys: Option<Vec<String>>,
            show_name: Option<String>,
            dim_rel_conf_dim_keys: Option<Vec<String>>,
            is_online: Option<bool>,
            page_number: u32,
            page_size: u32,
            desc_by_create: Option<bool>,
            desc_by_update: Option<bool>
        ) -> TardisResult<TardisPage<StatsConfFactInfoResp>>;
        find(
            fact_conf_keys: Option<Vec<String>>,
            show_name: Option<String>,
            dim_rel_conf_dim_keys: Option<Vec<String>>,
            is_online: Option<bool>,
            desc_by_create: Option<bool>,
            desc_by_update: Option<bool>
        ) -> TardisResult<Vec<StatsConfFactInfoResp>>;
        create_inst(fact_conf_key: &str) -> TardisResult<()>;
    }
}
