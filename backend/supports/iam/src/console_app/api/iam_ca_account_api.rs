use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::{param::Path, param::Query};
use tardis::web::web_resp::{TardisApiResult, TardisPage, TardisResp};

use bios_basic::rbum::dto::rbum_filer_dto::{RbumBasicFilterReq, RbumItemRelFilterReq};
use bios_basic::rbum::rbum_enumeration::RbumRelFromKind;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;

use crate::basic::dto::iam_account_dto::{IamAccountDetailAggResp, IamAccountSummaryAggResp};
use crate::basic::dto::iam_filer_dto::IamAccountFilterReq;
use crate::basic::serv::iam_account_serv::IamAccountServ;
use crate::basic::serv::iam_app_serv::IamAppServ;
use crate::iam_constants;
use crate::iam_enumeration::IamRelKind;
use bios_basic::helper::request_helper::try_set_real_ip_from_req_to_ctx;
use tardis::web::poem::Request;

#[derive(Clone, Default)]

pub struct IamCaAccountApi;

/// App Console Account API
/// 应用控制台账号API
#[poem_openapi::OpenApi(prefix_path = "/ca/account", tag = "bios_basic::ApiTag::App")]
impl IamCaAccountApi {
    /// Get Account By Account Id
    /// 根据账号ID获取账号
    #[oai(path = "/:id", method = "get")]
    async fn get(&self, id: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<IamAccountDetailAggResp> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        let result = IamAccountServ::get_account_detail_aggs(&id.0, &IamAccountFilterReq::default(), false, true, false, &funs, &ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Find Accounts
    /// 查找账号
    #[oai(path = "/", method = "get")]
    async fn paginate(
        &self,
        ids: Query<Option<String>>,
        name: Query<Option<String>>,
        role_ids: Query<Option<String>>,
        page_number: Query<u32>,
        page_size: Query<u32>,
        desc_by_create: Query<Option<bool>>,
        desc_by_update: Query<Option<bool>>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<TardisPage<IamAccountSummaryAggResp>> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        let rel2 = role_ids.0.map(|role_ids| {
            let role_ids = role_ids.split(',').map(|r| r.to_string()).collect::<Vec<_>>();
            RbumItemRelFilterReq {
                rel_by_from: true,
                tag: Some(IamRelKind::IamAccountRole.to_string()),
                from_rbum_kind: Some(RbumRelFromKind::Item),
                rel_item_ids: Some(role_ids),
                own_paths: Some(ctx.0.clone().own_paths),
                ..Default::default()
            }
        });
        let result = IamAccountServ::paginate_account_summary_aggs(
            &IamAccountFilterReq {
                basic: RbumBasicFilterReq {
                    ids: ids.0.map(|ids| ids.split(',').map(|id| id.to_string()).collect::<Vec<String>>()),
                    name: name.0,
                    // enabled: Some(true),
                    ..Default::default()
                },
                rel: IamAppServ::with_app_rel_filter(&ctx.0, &funs)?,
                rel2,
                ..Default::default()
            },
            false,
            true,
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

    /// Count Accounts
    /// 统计账号数量
    #[oai(path = "/total", method = "get")]
    async fn count(&self, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<u64> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        let result = IamAccountServ::count_items(&IamAccountFilterReq::default(), &funs, &ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }
}
