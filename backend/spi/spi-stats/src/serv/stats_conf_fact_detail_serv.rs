use crate::dto::stats_conf_dto::{StatsConfFactDetailAddReq, StatsConfFactDetailInfoResp, StatsConfFactDetailModifyReq};
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
        spi_constants::SPI_PG_KIND_CODE => pg::stats_pg_conf_fact_detail_serv,
    },
    @method: {
        add(
            fact_conf_key: &str,
            fact_conf_col_key: Option<&str>,
            add_req: &StatsConfFactDetailAddReq
        ) -> TardisResult<()>;
        modify(
            fact_conf_key: &str,
            fact_conf_col_key: Option<&str>,
            fact_conf_detail_key: &str,
            modify_req: &StatsConfFactDetailModifyReq
        ) -> TardisResult<()>;
        delete(
            fact_conf_key: &str,
            fact_conf_col_key: Option<&str>,
            fact_conf_detail_key: &str
        ) -> TardisResult<()> ;
        delete_by_fact_conf_key_and_col_key(
            fact_conf_key: &str,
            fact_conf_col_key: Option<&str>
        ) -> TardisResult<()>;
        delete_by_fact_conf_key(fact_conf_key: &str) -> TardisResult<()>;
        find_by_fact_conf_key(
            fact_conf_key: &str
        ) -> TardisResult<Vec<StatsConfFactDetailInfoResp>>;
        find_by_fact_key_and_col_conf_key(
            fact_conf_key: &str,
            fact_conf_col_key: &str
        ) -> TardisResult<Vec<StatsConfFactDetailInfoResp>>;
        find_up_by_fact_key_and_col_conf_key(
            fact_conf_key: &str,
            fact_conf_col_key: &str
        ) -> TardisResult<Vec<StatsConfFactDetailInfoResp>>;
        get_fact_detail(
            fact_conf_key: &str,
            fact_conf_col_key: &str,
            fact_conf_detail_key: &str
        ) -> TardisResult<Option<StatsConfFactDetailInfoResp>>;
        paginate(
            fact_conf_key: Option<String>,
            fact_conf_col_key: Option<String>,
            fact_conf_detail_key: Option<String>,
            show_name: Option<String>,
            page_number: u32,
            page_size: u32,
            desc_by_create: Option<bool>,
            desc_by_update: Option<bool>
        ) -> TardisResult<TardisPage<StatsConfFactDetailInfoResp>>;
    }
}
