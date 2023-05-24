use std::collections::HashMap;

use tardis::tokio::{self, task};
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::param::Query;
use tardis::web::web_resp::{TardisApiResult, TardisResp};

use bios_basic::rbum::dto::rbum_set_dto::RbumSetTreeResp;

use crate::basic::dto::iam_res_dto::IamResSummaryResp;
use crate::basic::serv::iam_res_serv::IamResServ;
use crate::basic::serv::iam_set_serv::IamSetServ;
use crate::iam_constants;
use crate::iam_enumeration::IamSetKind;

pub struct IamCcResApi;

/// Common Console Res API
///
#[poem_openapi::OpenApi(prefix_path = "/cc/res", tag = "bios_basic::ApiTag::Common")]
impl IamCcResApi {
    /// Find Menu Tree
    #[oai(path = "/tree", method = "get")]
    async fn get_menu_tree(&self, ctx: TardisContextExtractor) -> TardisApiResult<RbumSetTreeResp> {
        let funs = iam_constants::get_tardis_inst();
        let set_id = IamSetServ::get_set_id_by_code(&IamSetServ::get_default_code(&IamSetKind::Res, ""), true, &funs, &ctx.0).await?;
        let result = IamSetServ::get_menu_tree_by_roles(&set_id, &ctx.0.roles, &funs, &ctx.0).await?;
        let task_handle = task::spawn_blocking(move || tokio::runtime::Runtime::new().unwrap().block_on(ctx.0.execute_task()));
        let _ = task_handle.await;
        TardisResp::ok(result)
    }

    /// Find res by apps
    #[oai(path = "/res", method = "get")]
    async fn get_res_by_app(&self, app_ids: Query<String>, ctx: TardisContextExtractor) -> TardisApiResult<HashMap<String, Vec<IamResSummaryResp>>> {
        let funs = iam_constants::get_tardis_inst();
        let ids = app_ids.0.split(',').map(|s| s.to_string()).collect();
        let result = IamResServ::get_res_by_app(ids, &funs, &ctx.0).await?;
        let task_handle = task::spawn_blocking(move || tokio::runtime::Runtime::new().unwrap().block_on(ctx.0.execute_task()));
        let _ = task_handle.await;
        TardisResp::ok(result)
    }
}
