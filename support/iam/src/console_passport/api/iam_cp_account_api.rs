use bios_basic::process::task_processor::TaskProcessor;
use bios_basic::rbum::dto::rbum_filer_dto::{RbumSetItemFilterReq, RbumBasicFilterReq};
use bios_basic::rbum::dto::rbum_set_item_dto::RbumSetItemDetailResp;
use bios_basic::rbum::helper::rbum_event_helper;
use bios_basic::rbum::rbum_enumeration::RbumSetCateLevelQueryKind;
use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;
use bios_basic::rbum::serv::rbum_set_serv::RbumSetItemServ;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::payload::Json;
use tardis::web::web_resp::{TardisApiResult, TardisResp, Void};

use crate::basic::dto::iam_account_dto::IamAccountSelfModifyReq;
use crate::basic::serv::clients::iam_search_client::IamSearchClient;
use crate::basic::serv::iam_account_serv::IamAccountServ;
use crate::basic::serv::iam_cert_serv::IamCertServ;
use crate::basic::serv::iam_set_serv::IamSetServ;
use crate::console_passport::dto::iam_cp_account_dto::IamCpAccountInfoResp;
use crate::console_passport::serv::iam_cp_account_serv::IamCpAccountServ;
use crate::iam_config::IamBasicConfigApi;
use crate::iam_constants;
use bios_basic::helper::request_helper::add_remote_ip;
use tardis::web::poem::Request;
#[derive(Clone, Default)]
pub struct IamCpAccountApi;

/// Passport Console Account API
#[poem_openapi::OpenApi(prefix_path = "/cp/account", tag = "bios_basic::ApiTag::Passport")]
impl IamCpAccountApi {
    /// Modify Current Account
    #[oai(path = "/", method = "put")]
    async fn modify(&self, mut modify_req: Json<IamAccountSelfModifyReq>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        add_remote_ip(request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let ctx: tardis::basic::dto::TardisContext = IamCertServ::use_sys_or_tenant_ctx_unsafe(ctx.0)?;
        IamAccountServ::self_modify_account(&mut modify_req.0, &funs, &ctx).await?;
        IamSearchClient::async_add_or_modify_account_search(ctx.clone().owner, Box::new(true), "".to_string(), &funs, &ctx).await?;
        funs.commit().await?;
        ctx.execute_task().await?;
        if let Some(notify_events) = TaskProcessor::get_notify_event_with_ctx(&ctx).await? {
            rbum_event_helper::try_notifies(notify_events, &iam_constants::get_tardis_inst(), &ctx).await?;
        }
        TardisResp::ok(Void {})
    }

    /// Get Current Account
    #[oai(path = "/", method = "get")]
    async fn get_current_account_info(&self, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<IamCpAccountInfoResp> {
        add_remote_ip(request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        let result = IamCpAccountServ::get_current_account_info(true, &funs, &ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Find App Set Items (Account)
    #[oai(path = "/apps/item", method = "get")]
    async fn find_items(&self, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Vec<RbumSetItemDetailResp>> {
        let funs = iam_constants::get_tardis_inst();
        add_remote_ip(request, &ctx).await?;
        let ctx = IamCertServ::use_sys_or_tenant_ctx_unsafe(ctx.0)?;
        let set_id = IamSetServ::get_default_set_id_by_ctx(&IamSetKind::Apps, &funs, &ctx).await?;
        let cate_codes = RbumSetItemServ::find_detail_rbums(
            &RbumSetItemFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: false,
                    ..Default::default()
                },
                rel_rbum_item_disabled: Some(false),
                rel_rbum_set_id: Some(set_id.clone()),
                rel_rbum_item_ids: Some(vec![ctx.owner]),
                ..Default::default()
            },
            None,
            None,
            &funs,
            &ctx,
        )
        .await?
        .into_iter()
        .map(|resp| resp.rel_rbum_item_code.unwrap_or_default())
        .collect();
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
                rel_rbum_item_kind_ids: Some(vec![funs.iam_basic_kind_account_id()]),
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
