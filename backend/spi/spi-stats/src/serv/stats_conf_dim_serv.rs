use crate::dto::stats_conf_dto::{StatsConfDimAddReq, StatsConfDimInfoResp, StatsConfDimModifyReq};
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
        spi_constants::SPI_PG_KIND_CODE => pg::stats_pg_conf_dim_serv,
    },
    @method: {
        add(add_req: &StatsConfDimAddReq) -> TardisResult<()>;
        modify(dim_conf_key: &str, modify_req: &StatsConfDimModifyReq) -> TardisResult<()>;
        delete(dim_conf_key: &str) -> TardisResult<()>;
        paginate(
            dim_conf_key: Option<String>,
            show_name: Option<String>,
            page_number: u32,
            page_size: u32,
            desc_by_create: Option<bool>,
            desc_by_update: Option<bool>
        ) -> TardisResult<TardisPage<StatsConfDimInfoResp>>;
        create_inst(dim_conf_key: &str) -> TardisResult<()>;
    }
}
