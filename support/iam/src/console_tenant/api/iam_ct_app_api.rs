use bios_basic::process::task_processor::TaskProcessor;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::{param::Path, param::Query, payload::Json};
use tardis::web::web_resp::{TardisApiResult, TardisPage, TardisResp, Void};

use bios_basic::rbum::dto::rbum_filer_dto::RbumBasicFilterReq;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;

use crate::basic::dto::iam_app_dto::{IamAppAggAddReq, IamAppDetailResp, IamAppModifyReq, IamAppSummaryResp};
use crate::basic::dto::iam_filer_dto::IamAppFilterReq;
use crate::basic::serv::iam_app_serv::IamAppServ;
use crate::iam_constants;
use bios_basic::helper::request_helper::add_remote_ip;
use tardis::web::poem::Request;
#[derive(Clone, Default)]
pub struct IamCtAppApi;

/// Tenant Console App API
#[poem_openapi::OpenApi(prefix_path = "/ct/app", tag = "bios_basic::ApiTag::Tenant")]
impl IamCtAppApi {
    /// Add App
    #[oai(path = "/", method = "post")]
    async fn add(&self, add_req: Json<IamAppAggAddReq>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<String> {
        add_remote_ip(&request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let result = IamAppServ::add_app_agg(&add_req.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Modify App By App Id
    ///
    /// When code = 202, the return value is the asynchronous task id
    #[oai(path = "/:id", method = "put")]
    async fn modify(&self, id: Path<String>, mut modify_req: Json<IamAppModifyReq>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Option<String>> {
        add_remote_ip(&request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamAppServ::modify_item(&id.0, &mut modify_req, &funs, &ctx.0).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        if let Some(task_id) = TaskProcessor::get_task_id_with_ctx(&ctx.0).await? {
            TardisResp::accepted(Some(task_id))
        } else {
            TardisResp::ok(None)
        }
    }

    /// Get App By App Id
    #[oai(path = "/:id", method = "get")]
    async fn get(&self, id: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<IamAppDetailResp> {
        add_remote_ip(&request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        let result = IamAppServ::get_item(&id.0, &IamAppFilterReq::default(), &funs, &ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

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
        request: &Request,
    ) -> TardisApiResult<TardisPage<IamAppSummaryResp>> {
        add_remote_ip(&request, &ctx.0).await?;
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

    /// Delete App By App Id
    #[oai(path = "/:id", method = "delete")]
    async fn delete(&self, id: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        add_remote_ip(&request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamAppServ::delete_item_with_all_rels(&id.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }
}
