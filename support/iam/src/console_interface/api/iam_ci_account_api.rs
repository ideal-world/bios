use bios_basic::rbum::dto::rbum_filer_dto::{RbumBasicFilterReq, RbumCertFilterReq, RbumItemRelFilterReq, RbumSetCateFilterReq, RbumSetItemFilterReq, RbumSetItemRelFilterReq};
use bios_basic::rbum::rbum_enumeration::{RbumRelFromKind, RbumSetCateLevelQueryKind};
use bios_basic::rbum::serv::rbum_set_serv::RbumSetItemServ;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::{param::Path, param::Query};
use tardis::web::web_resp::{TardisApiResult, TardisPage, TardisResp};
use tardis::TardisFuns;

use crate::basic::dto::iam_account_dto::{IamAccountDetailAggResp, IamAccountDetailResp, IamAccountSummaryAggResp};
use crate::basic::dto::iam_filer_dto::IamAccountFilterReq;
use crate::basic::serv::iam_account_serv::IamAccountServ;
use crate::basic::serv::iam_cert_serv::IamCertServ;
use crate::basic::serv::iam_key_cache_serv::IamIdentCacheServ;
use crate::basic::serv::iam_set_serv::IamSetServ;
use crate::iam_config::IamBasicConfigApi;
use crate::iam_constants;
use crate::iam_enumeration::{IamRelKind, IamSetKind};
use bios_basic::helper::request_helper::add_remote_ip;
use bios_basic::rbum::serv::rbum_cert_serv::RbumCertServ;
use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;
use tardis::web::poem::Request;

#[derive(Clone, Default)]
pub struct IamCiAccountApi;

/// Interface Console Account API	接口控制台帐户API
#[poem_openapi::OpenApi(prefix_path = "/ci/account", tag = "bios_basic::ApiTag::Interface")]
impl IamCiAccountApi {
    /// Find Accounts
    #[oai(path = "/", method = "get")]
    #[allow(clippy::too_many_arguments)]
    async fn paginate(
        &self,
        ids: Query<Option<String>>,
        name: Query<Option<String>>,
        role_ids: Query<Option<String>>,
        cate_ids: Query<Option<String>>,
        status: Query<Option<bool>>,
        tenant_id: Query<Option<String>>,
        with_sub: Query<Option<bool>>,
        page_number: Query<u32>,
        page_size: Query<u32>,
        desc_by_create: Query<Option<bool>>,
        desc_by_update: Query<Option<bool>>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<TardisPage<IamAccountSummaryAggResp>> {
        let ctx = IamCertServ::try_use_tenant_ctx(ctx.0, tenant_id.0.clone())?;
        add_remote_ip(request, &ctx).await?;
        let funs = iam_constants::get_tardis_inst();
        let rel = role_ids.0.map(|role_ids| {
            let role_ids = role_ids.split(',').map(|r| r.to_string()).collect::<Vec<_>>();
            RbumItemRelFilterReq {
                rel_by_from: true,
                tag: Some(IamRelKind::IamAccountRole.to_string()),
                from_rbum_kind: Some(RbumRelFromKind::Item),
                rel_item_ids: Some(role_ids),
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
                set_ids_and_cate_codes: Some(set_cate_vec.into_iter().map(|sc| (sc.rel_rbum_set_id, sc.sys_code)).collect()),
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
                set_rel,
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
        ctx.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Get Context By Account Id	根据帐户Id获取上下文
    #[oai(path = "/:id/ctx", method = "get")]
    async fn get_account_context(&self, id: Path<String>, app_id: Query<Option<String>>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<String> {
        add_remote_ip(request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        let mut ctx_resp = IamIdentCacheServ::get_account_context(&id.0, &app_id.0.unwrap_or((&"").to_string()), &funs).await?;
        ctx_resp.own_paths = ctx.0.own_paths;
        TardisResp::ok(TardisFuns::crypto.base64.encode(TardisFuns::json.obj_to_string(&ctx_resp).unwrap_or_default()))
    }

    /// Get Account By Account Id	通过帐户Id获取帐户
    #[oai(path = "/:id", method = "get")]
    async fn get(&self, id: Path<String>, tenant_id: Query<Option<String>>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<IamAccountDetailAggResp> {
        let ctx = IamCertServ::try_use_tenant_ctx(ctx.0, tenant_id.0)?;
        add_remote_ip(request, &ctx).await?;
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
            true,
            &funs,
            &ctx,
        )
        .await?;
        ctx.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Find Account Id By Ak
    ///
    /// if kind is none,query default kind(UserPwd)
    #[oai(path = "/:ak/ak", method = "get")]
    async fn find_account_by_ak(
        &self,
        ak: Path<String>,
        kind: Query<Option<String>>,
        tenant_id: Query<Option<String>>,
        supplier: Query<Option<String>>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<Option<IamAccountDetailResp>> {
        add_remote_ip(request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        let supplier = supplier.0.unwrap_or_default();
        let kind = kind.0.unwrap_or_else(|| "UserPwd".to_string());
        let kind = if kind.is_empty() { "UserPwd".to_string() } else { kind };

        let result = if let Ok(conf_id) = IamCertServ::get_cert_conf_id_by_kind_supplier(&kind, &supplier.clone(), tenant_id.0.clone(), &funs).await {
            if let Some(cert) = RbumCertServ::find_one_detail_rbum(
                &RbumCertFilterReq {
                    basic: RbumBasicFilterReq {
                        own_paths: if let Some(tenant_id) = tenant_id.0 { Some(tenant_id) } else { Some("".to_string()) },
                        ..Default::default()
                    },
                    ak: Some(ak.0),
                    rel_rbum_cert_conf_ids: Some(vec![conf_id]),
                    ..Default::default()
                },
                &funs,
                &ctx.0,
            )
            .await?
            {
                Some(
                    IamAccountServ::get_item(
                        &cert.rel_rbum_id,
                        &IamAccountFilterReq {
                            basic: RbumBasicFilterReq {
                                own_paths: Some("".to_string()),
                                with_sub_own_paths: true,
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        &funs,
                        &ctx.0,
                    )
                    .await?,
                )
            } else {
                None
            }
        } else {
            None
        };

        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Find App Set Items (Account)
    #[oai(path = "/apps/account/ctx", method = "get")]
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
