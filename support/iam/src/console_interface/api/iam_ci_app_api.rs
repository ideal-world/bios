use crate::basic::dto::iam_app_dto::{IamAppAggAddReq, IamAppAggModifyReq};
use crate::basic::serv::iam_app_serv::IamAppServ;

use crate::basic::serv::iam_cert_serv::IamCertServ;
use crate::basic::serv::iam_set_serv::IamSetServ;
use crate::iam_config::IamBasicConfigApi;
use crate::iam_constants::{self};
use crate::iam_enumeration::IamSetKind;
use bios_basic::process::task_processor::TaskProcessor;
use bios_basic::rbum::dto::rbum_filer_dto::{RbumBasicFilterReq, RbumSetItemFilterReq};
use bios_basic::rbum::dto::rbum_set_item_dto::RbumSetItemDetailResp;
use bios_basic::rbum::rbum_enumeration::RbumSetCateLevelQueryKind;
use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;
use bios_basic::rbum::serv::rbum_set_serv::RbumSetItemServ;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi;

use bios_basic::helper::request_helper::add_remote_ip;
use tardis::web::poem::Request;
use tardis::web::poem_openapi::payload::Json;
use tardis::web::web_resp::{TardisApiResult, TardisResp};
#[derive(Clone, Default)]
pub struct IamCiAppApi;

/// # Interface Console Manage Cert API
///
/// Allow Management Of aksk (an authentication method between applications)
#[poem_openapi::OpenApi(prefix_path = "/ci/app", tag = "bios_basic::ApiTag::Interface")]
impl IamCiAppApi {
    /// Add App
    #[oai(path = "/", method = "post")]
    async fn add(&self, add_req: Json<IamAppAggAddReq>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<String> {
        add_remote_ip(request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let result = IamAppServ::add_app_agg(&add_req.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Modify Current App
    ///
    /// When code = 202, the return value is the asynchronous task id
    #[oai(path = "/", method = "put")]
    async fn modify(&self, modify_req: Json<IamAppAggModifyReq>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Option<String>> {
        add_remote_ip(request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;

        IamAppServ::modify_app_agg(&IamAppServ::get_id_by_ctx(&ctx.0, &funs)?, &modify_req, &funs, &ctx.0).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        if let Some(task_id) = TaskProcessor::get_task_id_with_ctx(&ctx.0).await? {
            TardisResp::accepted(Some(task_id))
        } else {
            TardisResp::ok(None)
        }
    }

    /// Find App Set Items (app)
    #[oai(path = "/apps/item/ctx", method = "get")]
    async fn find_items(&self, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Vec<RbumSetItemDetailResp>> {
        let funs = iam_constants::get_tardis_inst();
        let ctx = IamCertServ::use_sys_or_tenant_ctx_unsafe(ctx.0)?;
        add_remote_ip(request, &ctx).await?;
        let set_id = IamSetServ::get_default_set_id_by_ctx(&IamSetKind::Apps, &funs, &ctx).await?;
        let cate_codes = RbumSetItemServ::find_detail_rbums(
            &RbumSetItemFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: false,
                    ..Default::default()
                },
                rel_rbum_item_disabled: Some(false),
                rel_rbum_set_id: Some(set_id.clone()),
                rel_rbum_item_ids: Some(vec![ctx.owner.clone()]),
                ..Default::default()
            },
            None,
            None,
            &funs,
            &ctx,
        )
        .await?
        .into_iter()
        .map(|resp| resp.rel_rbum_item_code)
        .collect::<Vec<String>>();
        if cate_codes.is_empty() {
            return TardisResp::ok(vec![]);
        }
        let result = RbumSetItemServ::find_detail_rbums(
            &RbumSetItemFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: false,
                    ..Default::default()
                },
                rel_rbum_item_disabled: Some(false),
                rel_rbum_set_id: Some(set_id.clone()),
                rel_rbum_set_cate_sys_codes: Some(cate_codes),
                sys_code_query_kind: Some(RbumSetCateLevelQueryKind::CurrentAndSub),
                rel_rbum_item_kind_ids: Some(vec![funs.iam_basic_kind_app_id()]),
                ..Default::default()
            },
            None,
            None,
            &funs,
            &ctx,
        )
        .await?;
        ctx.execute_task().await?;
        TardisResp::ok(result)
    }
}
