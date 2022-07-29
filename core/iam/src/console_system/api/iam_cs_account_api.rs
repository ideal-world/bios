use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::{param::Path, param::Query, payload::Json};
use tardis::web::web_resp::{TardisApiResult, TardisPage, TardisResp, Void};

use bios_basic::rbum::dto::rbum_filer_dto::{RbumBasicFilterReq, RbumItemRelFilterReq};
use bios_basic::rbum::rbum_enumeration::RbumRelFromKind;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;

use crate::basic::dto::iam_account_dto::{IamAccountAggAddReq, IamAccountAggModifyReq, IamAccountDetailAggResp, IamAccountSummaryAggResp};
use crate::basic::dto::iam_filer_dto::IamAccountFilterReq;
use crate::basic::serv::iam_account_serv::IamAccountServ;
use crate::basic::serv::iam_cert_serv::IamCertServ;
use crate::iam_constants;
use crate::iam_enumeration::IamRelKind;

pub struct IamCsAccountApi;

/// System Console Account API
#[poem_openapi::OpenApi(prefix_path = "/cs/account", tag = "crate::iam_enumeration::Tag::System")]
impl IamCsAccountApi {
    /// Add Account By Tenant Id
    #[oai(path = "/", method = "post")]
    async fn add(&self, tenant_id: Query<Option<String>>, add_req: Json<IamAccountAggAddReq>, ctx: TardisContextExtractor) -> TardisApiResult<String> {
        let ctx = IamCertServ::try_use_tenant_ctx(ctx.0, tenant_id.0)?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let result = IamAccountServ::add_account_agg(&add_req.0, &funs, &ctx).await?;
        funs.commit().await?;
        TardisResp::ok(result)
    }

    /// Modify Account By Account Id
    #[oai(path = "/:id", method = "put")]
    async fn modify(&self, id: Path<String>, tenant_id: Query<Option<String>>, modify_req: Json<IamAccountAggModifyReq>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let ctx = IamCertServ::try_use_tenant_ctx(ctx.0, tenant_id.0)?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamAccountServ::modify_account_agg(&id.0, &modify_req.0, &funs, &ctx).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Get Account By Account Id
    #[oai(path = "/:id", method = "get")]
    async fn get(&self, id: Path<String>, tenant_id: Query<Option<String>>, ctx: TardisContextExtractor) -> TardisApiResult<IamAccountDetailAggResp> {
        let ctx = IamCertServ::try_use_tenant_ctx(ctx.0, tenant_id.0)?;
        let funs = iam_constants::get_tardis_inst();
        let result = IamAccountServ::get_account_detail_aggs(
            &id.0,
            &IamAccountFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                ..Default::default()
            },
            true,
            false,
            &funs,
            &ctx,
        )
        .await?;
        TardisResp::ok(result)
    }

    /// Find Accounts
    #[oai(path = "/", method = "get")]
    async fn paginate(
        &self,
        id: Query<Option<String>>,
        name: Query<Option<String>>,
        role_id: Query<Option<String>>,
        tenant_id: Query<Option<String>>,
        with_sub: Query<Option<bool>>,
        page_number: Query<u64>,
        page_size: Query<u64>,
        desc_by_create: Query<Option<bool>>,
        desc_by_update: Query<Option<bool>>,
        ctx: TardisContextExtractor,
    ) -> TardisApiResult<TardisPage<IamAccountSummaryAggResp>> {
        let ctx = IamCertServ::try_use_tenant_ctx(ctx.0, tenant_id.0.clone())?;
        let funs = iam_constants::get_tardis_inst();
        let rel = role_id.0.map(|role_id| RbumItemRelFilterReq {
            rel_by_from: true,
            tag: Some(IamRelKind::IamAccountRole.to_string()),
            from_rbum_kind: Some(RbumRelFromKind::Item),
            rel_item_id: Some(role_id),
            ..Default::default()
        });
        let result = IamAccountServ::paginate_account_summary_aggs(
            &IamAccountFilterReq {
                basic: RbumBasicFilterReq {
                    ids: id.0.map(|id| vec![id]),
                    name: name.0,
                    with_sub_own_paths: with_sub.0.unwrap_or(false),
                    ..Default::default()
                },
                rel,
                ..Default::default()
            },
            tenant_id.0.is_none(),
            tenant_id.0.is_none(),
            page_number.0,
            page_size.0,
            desc_by_create.0,
            desc_by_update.0,
            &funs,
            &ctx,
        )
        .await?;
        TardisResp::ok(result)
    }

    /// Delete Account By Account Id
    #[oai(path = "/:id", method = "delete")]
    async fn delete(&self, id: Path<String>, tenant_id: Query<Option<String>>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let ctx = IamCertServ::try_use_tenant_ctx(ctx.0, tenant_id.0)?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamAccountServ::delete_item_with_all_rels(&id.0, &funs, &ctx).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Delete Token By Account Id
    #[oai(path = "/:id/token", method = "delete")]
    async fn offline(&self, id: Path<String>, tenant_id: Query<Option<String>>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let ctx = IamCertServ::try_use_tenant_ctx(ctx.0, tenant_id.0)?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamAccountServ::delete_tokens(&id.0, &funs, &ctx).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Count Accounts By Tenant Id
    #[oai(path = "/total", method = "get")]
    async fn count(&self, tenant_id: Query<Option<String>>, ctx: TardisContextExtractor) -> TardisApiResult<u64> {
        let ctx = IamCertServ::try_use_tenant_ctx(ctx.0, tenant_id.0)?;
        let funs = iam_constants::get_tardis_inst();
        let result = IamAccountServ::count_items(
            &IamAccountFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                ..Default::default()
            },
            &funs,
            &ctx,
        )
        .await?;
        TardisResp::ok(result)
    }
}
