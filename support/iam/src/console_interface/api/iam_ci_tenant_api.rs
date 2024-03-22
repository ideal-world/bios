use std::collections::HashMap;

use bios_basic::helper::request_helper::add_remote_ip;
use bios_basic::rbum::dto::rbum_filer_dto::{RbumBasicFilterReq, RbumItemRelFilterReq, RbumSetCateFilterReq, RbumSetItemRelFilterReq, RbumSetTreeFilterReq};
use bios_basic::rbum::dto::rbum_set_dto::RbumSetTreeMainResp;
use bios_basic::rbum::rbum_enumeration::{RbumRelFromKind, RbumSetCateLevelQueryKind};
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;
use tardis::web::poem::web::{Path, Query};
use tardis::web::poem_openapi;
use tardis::web::web_resp::TardisPage;
use tardis::web::{
    context_extractor::TardisContextExtractor,
    poem::Request,
    web_resp::{TardisApiResult, TardisResp},
};

use crate::basic::dto::iam_account_dto::{IamAccountDetailAggResp, IamAccountSummaryAggResp};
use crate::basic::dto::iam_filer_dto::{IamAccountFilterReq, IamRoleFilterReq};
use crate::basic::dto::iam_role_dto::IamRoleSummaryResp;
use crate::basic::serv::iam_account_serv::IamAccountServ;
use crate::basic::serv::iam_cert_serv::IamCertServ;
use crate::basic::serv::iam_role_serv::IamRoleServ;
use crate::basic::serv::iam_set_serv::IamSetServ;
use crate::iam_enumeration::{IamRelKind, IamSetKind};
use crate::{
    basic::{
        dto::{iam_filer_dto::IamTenantFilterReq, iam_tenant_dto::IamTenantAggDetailResp},
        serv::iam_tenant_serv::IamTenantServ,
    },
    iam_constants,
};

#[derive(Clone, Default)]
pub struct IamCiTenantApi;

#[poem_openapi::OpenApi(prefix_path = "/ci/tenant", tag = "bios_basic::ApiTag::Tenant")]
impl IamCiTenantApi {
    /// Get Current Tenant
    #[oai(path = "/", method = "get")]
    async fn get(&self, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<IamTenantAggDetailResp> {
        add_remote_ip(request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        let result = IamTenantServ::get_tenant_agg(&IamTenantServ::get_id_by_ctx(&ctx.0, &funs)?, &IamTenantFilterReq::default(), &funs, &ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Find Org Tree By Current Tenant
    ///
    /// * Without parameters: Query the whole tree
    /// * ``parent_sys_code=true`` : query only the next level. This can be used to query level by level when the tree is too large
    #[oai(path = "/orgs", method = "get")]
    async fn get_orgs(
        &self,
        parent_sys_code: Query<Option<String>>,
        set_id: Query<Option<String>>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<Vec<RbumSetTreeMainResp>> {
        let funs = iam_constants::get_tardis_inst();
        let ctx = IamSetServ::try_get_rel_ctx_by_set_id(set_id.0, &funs, ctx.0).await?;
        add_remote_ip(request, &ctx).await?;
        let set_id = IamSetServ::get_default_set_id_by_ctx(&IamSetKind::Org, &funs, &ctx).await?;
        let result = IamSetServ::get_tree(
            &set_id,
            &mut RbumSetTreeFilterReq {
                fetch_cate_item: true,
                sys_codes: parent_sys_code.0.map(|parent_sys_code| vec![parent_sys_code]),
                sys_code_query_kind: Some(RbumSetCateLevelQueryKind::Sub),
                sys_code_query_depth: Some(1),
                ..Default::default()
            },
            &funs,
            &ctx,
        )
        .await?;
        ctx.execute_task().await?;
        TardisResp::ok(result.main)
    }

    /// Find Accounts
    #[oai(path = "/accounts", method = "get")]
    #[allow(clippy::too_many_arguments)]
    async fn get_accounts(
        &self,
        ids: Query<Option<String>>,
        name: Query<Option<String>>,
        role_ids: Query<Option<String>>,
        app_ids: Query<Option<String>>,
        cate_ids: Query<Option<String>>,
        status: Query<Option<bool>>,
        app_id: Query<Option<String>>,
        with_sub: Query<Option<bool>>,
        page_number: Query<u32>,
        page_size: Query<u32>,
        desc_by_create: Query<Option<bool>>,
        desc_by_update: Query<Option<bool>>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<TardisPage<IamAccountSummaryAggResp>> {
        let ctx = IamCertServ::try_use_app_ctx(ctx.0, app_id.0)?;
        add_remote_ip(request, &ctx).await?;
        let funs = iam_constants::get_tardis_inst();
        let rel = role_ids.0.map(|role_ids| {
            let role_ids = role_ids.split(',').map(|r| r.to_string()).collect::<Vec<_>>();
            RbumItemRelFilterReq {
                rel_by_from: true,
                tag: Some(IamRelKind::IamAccountRole.to_string()),
                from_rbum_kind: Some(RbumRelFromKind::Item),
                rel_item_ids: Some(role_ids),
                own_paths: Some(ctx.own_paths.clone()),
                ..Default::default()
            }
        });
        let rel2 = app_ids.0.map(|app_ids| {
            let app_ids = app_ids.split(',').map(|r| r.to_string()).collect::<Vec<_>>();
            RbumItemRelFilterReq {
                rel_by_from: true,
                tag: Some(IamRelKind::IamAccountApp.to_string()),
                from_rbum_kind: Some(RbumRelFromKind::Item),
                rel_item_ids: Some(app_ids),
                own_paths: Some(ctx.own_paths.clone()),
                ..Default::default()
            }
        });
        let set_rel = if let Some(cate_ids) = cate_ids.0 {
            let cate_ids = cate_ids.split(',').map(|r| r.to_string()).collect::<Vec<_>>();
            let set_cate_vec = IamSetServ::find_set_cate(
                &RbumSetCateFilterReq {
                    basic: RbumBasicFilterReq {
                        own_paths: Some("".to_string()),
                        with_sub_own_paths: true,
                        ids: Some(cate_ids),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                None,
                None,
                &funs,
                &ctx,
            )
            .await?;
            Some(RbumSetItemRelFilterReq {
                set_ids_and_cate_codes: Some(
                    set_cate_vec.into_iter().map(|sc| (sc.rel_rbum_set_id, sc.sys_code)).fold(HashMap::new(), |mut acc, (key, value)| {
                        acc.entry(key).or_default().push(value);
                        acc
                    }),
                ),
                with_sub_set_cate_codes: false,
                ..Default::default()
            })
        } else {
            None
        };
        let result = IamAccountServ::paginate_account_summary_aggs(
            &IamAccountFilterReq {
                basic: RbumBasicFilterReq {
                    ids: ids.0.map(|ids| ids.split(',').map(|id| id.to_string()).collect::<Vec<String>>()),
                    name: name.0,
                    with_sub_own_paths: with_sub.0.unwrap_or(false),
                    enabled: status.0,
                    ..Default::default()
                },
                rel,
                rel2,
                set_rel,
                ..Default::default()
            },
            false,
            true,
            page_number.0,
            page_size.0,
            desc_by_create.0,
            desc_by_update.0,
            &funs,
            &ctx,
        )
        .await?;
        ctx.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Get Account
    #[oai(path = "/account/:id", method = "get")]
    async fn get_account(&self, id: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<IamAccountDetailAggResp> {
        add_remote_ip(request, &ctx.0).await?;
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
            false,
            true,
            &funs,
            &ctx.0,
        )
        .await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Find Roles
    #[oai(path = "/roles", method = "get")]
    async fn get_roles(
        &self,
        id: Query<Option<String>>,
        name: Query<Option<String>>,
        app_id: Query<Option<String>>,
        in_base: Query<Option<bool>>,
        in_embed: Query<Option<bool>>,
        extend_role_id: Query<Option<String>>,
        with_sub: Query<Option<bool>>,
        page_number: Query<u32>,
        page_size: Query<u32>,
        desc_by_create: Query<Option<bool>>,
        desc_by_update: Query<Option<bool>>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<TardisPage<IamRoleSummaryResp>> {
        let ctx = IamCertServ::try_use_app_ctx(ctx.0, app_id.0)?;
        add_remote_ip(request, &ctx).await?;
        let funs = iam_constants::get_tardis_inst();
        let result = IamRoleServ::paginate_items(
            &IamRoleFilterReq {
                basic: RbumBasicFilterReq {
                    ids: id.0.map(|id| vec![id]),
                    name: name.0,
                    with_sub_own_paths: with_sub.0.unwrap_or(false),
                    ..Default::default()
                },
                // kind: Some(IamRoleKind::Tenant),
                in_base: in_base.0,
                in_embed: in_embed.0,
                extend_role_id: extend_role_id.0,
                ..Default::default()
            },
            page_number.0,
            page_size.0,
            desc_by_create.0,
            desc_by_update.0,
            &funs,
            &ctx,
        )
        .await?;
        ctx.execute_task().await?;
        TardisResp::ok(result)
    }
}
