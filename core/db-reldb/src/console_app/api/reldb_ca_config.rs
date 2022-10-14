use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::param::{Path, Query};
use tardis::web::poem_openapi::payload::Json;
use tardis::web::web_resp::{TardisApiResult, TardisPage, TardisResp, Void};

use crate::basic::dto::reldb_config_dto::{RelDbConfigAggAddOrModifyReq, RelDbConfigAggResp, RelDbConfigSummaryResp};
use crate::reldb_constants;

pub struct RelDbCaConfigApi;

/// App Console RelDb Config API
#[poem_openapi::OpenApi(prefix_path = "/ca/config", tag = "bios_basic::ApiTag::App")]
impl RelDbCaConfigApi {
    /// Add RelDb Config
    #[oai(path = "/", method = "post")]
    async fn add_config(&self, add_req: Json<RelDbConfigAggAddOrModifyReq>, ctx: TardisContextExtractor) -> TardisApiResult<String> {
        let mut funs = reldb_constants::get_tardis_inst();
        funs.begin().await?;
        // TODO
        funs.commit().await?;
        TardisResp::ok("".to_string())
    }

    /// Modify RelDb Config By Instance Id
    #[oai(path = "/:inst_id", method = "put")]
    async fn modify_config(&self, inst_id: Path<String>, modify_req: Json<RelDbConfigAggAddOrModifyReq>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = reldb_constants::get_tardis_inst();
        funs.begin().await?;
        // TODO
        funs.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Find RelDb Config
    #[oai(path = "/", method = "get")]
    async fn paginate_config(
        &self,
        inst_id: Query<Option<String>>,
        name: Query<Option<String>>,
        page_number: Query<u64>,
        page_size: Query<u64>,
        desc_by_create: Query<Option<bool>>,
        desc_by_update: Query<Option<bool>>,
        ctx: TardisContextExtractor,
    ) -> TardisApiResult<TardisPage<RelDbConfigSummaryResp>> {
        let funs = reldb_constants::get_tardis_inst();
        // TODO
        TardisResp::ok(TardisPage {
            page_size: todo!(),
            page_number: todo!(),
            total_size: todo!(),
            records: todo!(),
        })
    }

    /// Get RelDb Config By Instance Id
    #[oai(path = "/:inst_id", method = "get")]
    async fn get_config(&self, inst_id: Path<String>, ctx: TardisContextExtractor) -> TardisApiResult<RelDbConfigAggResp> {
        let funs = reldb_constants::get_tardis_inst();
        // TODO
        TardisResp::ok(RelDbConfigAggResp {
            id: todo!(),
            name: todo!(),
            own_paths: todo!(),
            owner: todo!(),
            owner_name: todo!(),
            create_time: todo!(),
            update_time: todo!(),
            scope_level: todo!(),
            disabled: todo!(),
            icon: todo!(),
            cert_user_name: todo!(),
            cert_password: todo!(),
            connect_uri: todo!(),
        })
    }

    /// Delete RelDb Config By Instance Id
    #[oai(path = "/:inst_id", method = "delete")]
    async fn delete_config(&self, inst_id: Path<String>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = reldb_constants::get_tardis_inst();
        funs.begin().await?;
        // TODO
        funs.commit().await?;
        TardisResp::ok(Void {})
    }
}
