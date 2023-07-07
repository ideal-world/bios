use bios_basic::rbum::dto::rbum_filer_dto::RbumBasicFilterReq;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::param::Query;
use tardis::web::web_resp::{TardisApiResult, TardisPage, TardisResp};

use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;

use crate::basic::dto::iam_app_dto::IamAppSummaryResp;
use crate::basic::dto::iam_filer_dto::IamAppFilterReq;
use crate::basic::serv::iam_app_serv::IamAppServ;
use crate::iam_constants;

#[derive(Clone, Default)]
pub struct IamCcAppApi;

/// Common Console App API
#[poem_openapi::OpenApi(prefix_path = "/cc/app", tag = "bios_basic::ApiTag::Common")]
impl IamCcAppApi {
    /// Find Apps
    #[oai(path = "/", method = "get")]
    async fn paginate(
        &self,
        id: Query<Option<String>>,
        name: Query<Option<String>>,
        desc_by_create: Query<Option<bool>>,
        desc_by_update: Query<Option<bool>>,
        page_number: Query<u32>,
        page_size: Query<u32>,
        ctx: TardisContextExtractor,
    ) -> TardisApiResult<TardisPage<IamAppSummaryResp>> {
        let funs = iam_constants::get_tardis_inst();

        let result = IamAppServ::paginate_items(
            &IamAppFilterReq {
                basic: RbumBasicFilterReq {
                    ids: id.0.map(|id| vec![id]),
                    name: name.0,
                    with_sub_own_paths: true,
                    enabled: Some(true),
                    ..Default::default()
                },
                ..Default::default()
            },
            page_number.0,
            page_size.0,
            desc_by_create.0,
            desc_by_update.0,
            &funs,
            &ctx.0,
        )
        .await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Find App Name and icon By Ids
    ///
    /// Return format: ["<id>,<name>,<icon>"]
    #[oai(path = "/name", method = "get")]
    async fn find_name_by_ids(
        &self,
        // App Ids, multiple ids separated by ,
        ids: Query<String>,
        ctx: TardisContextExtractor,
    ) -> TardisApiResult<Vec<String>> {
        let funs = iam_constants::get_tardis_inst();
        let ids = ids.0.split(',').map(|s| s.to_string()).collect();
        let result = IamAppServ::find_name_by_ids(
            IamAppFilterReq {
                basic: RbumBasicFilterReq {
                    ids: Some(ids),
                    with_sub_own_paths: true,
                    own_paths: Some("".to_string()),
                    ignore_scope: true,
                    ..Default::default()
                },
                ..Default::default()
            },
            &funs,
            &ctx.0,
        )
        .await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }
}
