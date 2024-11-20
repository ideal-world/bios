use crate::dto::stats_conf_dto::{StatsConfDimGroupAddReq, StatsConfDimGroupInfoResp, StatsConfDimGroupModifyReq};
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
        spi_constants::SPI_PG_KIND_CODE => pg::stats_pg_conf_dim_group_serv,
    },
    @method: {
        add(add_req: &StatsConfDimGroupAddReq) -> TardisResult<()>;
        modify(dim_conf_key: &str, modify_req: &StatsConfDimGroupModifyReq) -> TardisResult<()>;
        paginate(
          dim_group_key: Option<String>,
          page_number: u32,
          page_size: u32,
          desc_by_create: Option<bool>,
          desc_by_update: Option<bool>
      ) -> TardisResult<TardisPage<StatsConfDimGroupInfoResp>>;
    }
}
