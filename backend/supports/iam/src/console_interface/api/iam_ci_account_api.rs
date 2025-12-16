use std::collections::HashMap;

use bios_basic::rbum::dto::rbum_filer_dto::{RbumBasicFilterReq, RbumCertFilterReq, RbumItemRelFilterReq, RbumSetCateFilterReq, RbumSetItemFilterReq, RbumSetItemRelFilterReq};
use bios_basic::rbum::dto::rbum_set_item_dto::RbumSetItemDetailResp;
use bios_basic::rbum::helper::rbum_scope_helper::check_without_owner_and_unsafe_fill_ctx;
use bios_basic::rbum::rbum_enumeration::{RbumRelFromKind, RbumSetCateLevelQueryKind};
use bios_basic::rbum::serv::rbum_set_serv::RbumSetItemServ;
use tardis::basic::dto::TardisContext;
use tardis::futures::future::join_all;
use tardis::log;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi;

use tardis::web::poem_openapi::{param::Path, param::Query, payload::Json};
use tardis::web::web_resp::{TardisApiResult, TardisPage, TardisResp, Void};
use tardis::TardisFuns;

use crate::basic::dto::iam_account_dto::{IamAccountAggAddReq, IamAccountAggModifyReq, IamAccountAppInfoResp, IamAccountBindRoleReq, IamAccountDetailAggResp, IamAccountDetailResp, IamAccountOthersIdInitReq, IamAccountSummaryAggResp};
use crate::basic::dto::iam_app_dto::IamAppKind;
use crate::basic::dto::iam_filer_dto::IamAccountFilterReq;
use crate::basic::serv::clients::iam_search_client::IamSearchClient;
use crate::basic::serv::iam_account_serv::IamAccountServ;
use crate::basic::serv::iam_app_serv::IamAppServ;
use crate::basic::serv::iam_cert_serv::IamCertServ;
use crate::basic::serv::iam_key_cache_serv::IamIdentCacheServ;
use crate::basic::serv::iam_role_serv::IamRoleServ;
use crate::basic::serv::iam_set_serv::IamSetServ;
use crate::iam_config::IamBasicConfigApi;
use crate::iam_constants::{self, RBUM_SCOPE_LEVEL_APP};
use crate::iam_enumeration::{IamCertKernelKind, IamRelKind, IamSetKind};
use bios_basic::helper::request_helper::try_set_real_ip_from_req_to_ctx;
use bios_basic::rbum::serv::rbum_cert_serv::RbumCertServ;
use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;
use tardis::web::poem::Request;

#[derive(Clone, Default)]
pub struct IamCiAccountApi;

/// Interface Console Account API
/// 接口控制台帐户API
#[poem_openapi::OpenApi(prefix_path = "/ci/account", tag = "bios_basic::ApiTag::Interface")]
impl IamCiAccountApi {
    /// Add Account
    /// 添加帐户
    #[oai(path = "/", method = "post")]
    async fn add(&self, mut add_req: Json<IamAccountAggAddReq>, mut ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<String> {
        let mut funs = iam_constants::get_tardis_inst();
        check_without_owner_and_unsafe_fill_ctx(request, &funs, &mut ctx.0)?;
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        funs.begin().await?;
        let mock_ctx = TardisContext {
            owner: TardisFuns::field.nanoid(),
            ..ctx.0.clone()
        };
        add_req.0.id = Some(mock_ctx.owner.clone().into());
        let result = IamAccountServ::add_account_agg(&add_req.0, false, &funs, &mock_ctx).await?;
        IamSearchClient::async_add_or_modify_account_search(&result, Box::new(false), "", &funs, &mock_ctx).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Modify Account
    /// 修改帐户
    #[oai(path = "/:others_id", method = "put")]
    async fn modify_by_others_id(&self, others_id: Path<String>, modify_req: Json<IamAccountAggModifyReq>, mut ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        check_without_owner_and_unsafe_fill_ctx(request, &funs, &mut ctx.0)?;
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        funs.begin().await?;
        if let Some(account) = IamAccountServ::find_one_item(&IamAccountFilterReq {
            basic: RbumBasicFilterReq {
                with_sub_own_paths: true,
                ..Default::default()
            },
            others_id: Some(others_id.0),
            ..Default::default()
        }, &funs, &ctx.0).await? {
            IamAccountServ::modify_account_agg(&account.id, &modify_req.0, &funs, &ctx.0).await?;
            IamSearchClient::async_add_or_modify_account_search(&account.id, Box::new(true), "", &funs, &ctx.0).await?;
        }
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Batch Add Account
    /// 批量添加帐户
    #[oai(path = "/batch", method = "post")]
    async fn batch_add(&self, batch_add_req: Json<Vec<IamAccountAggAddReq>>, mut ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        let funs = iam_constants::get_tardis_inst();
        check_without_owner_and_unsafe_fill_ctx(request, &funs, &mut ctx.0)?;
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let ctx_clone = ctx.0.clone();
        
        join_all(batch_add_req.0.into_iter().map(|mut add_req| {
            let mock_ctx = TardisContext {
                owner: TardisFuns::field.nanoid(),
                ..ctx_clone.clone()
            };
            add_req.id = Some(mock_ctx.owner.clone().into());
            async move {
                let mut funs_cp = iam_constants::get_tardis_inst();
                let others_id = add_req.others_id.clone();
                funs_cp.begin().await.unwrap_or_default();
                match IamAccountServ::add_account_agg(&add_req, false, &funs_cp, &mock_ctx).await {
                    Ok(result) => {
                        let _ = IamSearchClient::async_add_or_modify_account_search(&result, Box::new(false), "", &funs_cp, &mock_ctx).await;
                        funs_cp.commit().await.unwrap_or_default();
                    },
                    Err(err) => {
                        funs_cp.rollback().await.unwrap_or_default();
                        log::error!("[IAM] batch_add_account_agg error: others_id {:?} error: {:?}", others_id, err);
                    }
                }
            }
        }).collect::<Vec<_>>()).await;
        
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Modify Account
    /// 修改帐户
    #[oai(path = "/batch", method = "put")]
    async fn batch_modify_by_others_id(&self,  batch_modify_req: Json<HashMap<String, IamAccountAggModifyReq>>, mut ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        check_without_owner_and_unsafe_fill_ctx(request, &funs, &mut ctx.0)?;
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        funs.begin().await?;
        for (others_id, modify_req) in batch_modify_req.0.into_iter() {
            if let Some(account) = IamAccountServ::find_one_item(&IamAccountFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                others_id: Some(others_id),
                ..Default::default()
            }, &funs, &ctx.0).await? {
                IamAccountServ::modify_account_agg(&account.id, &modify_req, &funs, &ctx.0).await?;
                IamSearchClient::async_add_or_modify_account_search(&account.id, Box::new(true), "", &funs, &ctx.0).await?;
            }
        }
        
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Init OthersId By Phone
    /// 通过手机号批量初始化账号的 others_id
    ///
    /// 入参为一组手机号与对应 others_id，按照手机号匹配账号，
    /// 匹配成功则为该账号设置 others_id；未匹配到的记录将原样返回。
    #[oai(path = "/others-id/init-by-phone", method = "put")]
    async fn init_others_id_by_phone(
        &self,
        init_reqs: Json<Vec<IamAccountOthersIdInitReq>>,
        mut ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<Vec<IamAccountOthersIdInitReq>> {
        let mut funs = iam_constants::get_tardis_inst();
        check_without_owner_and_unsafe_fill_ctx(request, &funs, &mut ctx.0)?;
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        funs.begin().await?;

        let mut not_matched = Vec::new();

        for req in init_reqs.0.into_iter() {
            // 通过手机号证书查找账号
            let cert_opt = RbumCertServ::find_one_detail_rbum(
                &RbumCertFilterReq {
                    basic: RbumBasicFilterReq {
                        own_paths: Some("".to_string()),
                        with_sub_own_paths: true,
                        ..Default::default()
                    },
                    ak: Some(req.phone.clone()),
                    kind: Some(IamCertKernelKind::PhoneVCode.to_string()),
                    ..Default::default()
                },
                &funs,
                &ctx.0,
            )
            .await?;

            if let Some(cert) = cert_opt {
                // 为匹配到的账号初始化 others_id
                IamAccountServ::init_others_id_by_id(&cert.rel_rbum_id, &req.others_id, &funs, &ctx.0).await?;
            } else {
                // 未匹配到账号的记录收集返回
                not_matched.push(req);
            }
        }

        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(not_matched)
    }

    /// Find Accounts
    /// 查找帐户
    #[oai(path = "/", method = "get")]
    #[allow(clippy::too_many_arguments)]
    async fn paginate(
        &self,
        ids: Query<Option<String>>,
        name: Query<Option<String>>,
        role_ids: Query<Option<String>>,
        app_ids: Query<Option<String>>,
        cate_ids: Query<Option<String>>,
        status: Query<Option<bool>>,
        tenant_id: Query<Option<String>>,
        with_sub: Query<Option<bool>>,
        page_number: Query<u32>,
        page_size: Query<u32>,
        desc_by_create: Query<Option<bool>>,
        desc_by_update: Query<Option<bool>>,
        mut ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<TardisPage<IamAccountSummaryAggResp>> {
        let funs = iam_constants::get_tardis_inst();
        check_without_owner_and_unsafe_fill_ctx(request, &funs, &mut ctx.0)?;
        let ctx = IamCertServ::try_use_tenant_ctx(ctx.0, tenant_id.0.clone())?;
        try_set_real_ip_from_req_to_ctx(request, &ctx).await?;
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
        let rel2 = app_ids.0.map(|app_ids| {
            let app_ids = app_ids.split(',').map(|r| r.to_string()).collect::<Vec<_>>();
            RbumItemRelFilterReq {
                rel_by_from: true,
                tag: Some(IamRelKind::IamAccountApp.to_string()),
                from_rbum_kind: Some(RbumRelFromKind::Item),
                rel_item_ids: Some(app_ids),
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

    /// Get Context By Account Id
    /// 根据帐户Id获取上下文
    #[oai(path = "/:id/ctx", method = "get")]
    async fn get_account_context(&self, id: Path<String>, app_id: Query<Option<String>>, mut ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<String> {
        let funs = iam_constants::get_tardis_inst();
        check_without_owner_and_unsafe_fill_ctx(request, &funs, &mut ctx.0)?;
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let mut ctx_resp = IamIdentCacheServ::get_account_context(&id.0, &app_id.0.unwrap_or((&"").to_string()), &funs).await?;
        ctx_resp.own_paths = ctx.0.own_paths;
        TardisResp::ok(TardisFuns::crypto.base64.encode(TardisFuns::json.obj_to_string(&ctx_resp).unwrap_or_default()))
    }

    /// Get Account By Account Id
    /// 通过帐户Id获取帐户
    #[oai(path = "/:id", method = "get")]
    async fn get(&self, id: Path<String>, tenant_id: Query<Option<String>>, mut ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<IamAccountDetailAggResp> {
        let funs = iam_constants::get_tardis_inst();
        check_without_owner_and_unsafe_fill_ctx(request, &funs, &mut ctx.0)?;
        let ctx = IamCertServ::try_use_tenant_ctx(ctx.0, tenant_id.0)?;
        try_set_real_ip_from_req_to_ctx(request, &ctx).await?;
        let mut result = IamAccountServ::get_account_detail_aggs(
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
            false,
            &funs,
            &ctx,
        )
        .await?;
        // 添加项目组下的 `app` 及角色
        let mut apps = result.apps.clone();
        if ctx.own_paths != "" {
            let old_app_ids = apps.iter().map(|a| a.app_id.clone()).collect::<Vec<String>>();
            let set_id = IamSetServ::get_default_set_id_by_ctx(&IamSetKind::Apps, &funs, &ctx).await?;
            let app_items = IamSetServ::get_app_with_auth_by_account(&set_id, &id, &funs, &ctx).await?;
            let mut app_role_read = HashMap::new();
            app_role_read.insert(funs.iam_basic_role_app_read_id(), iam_constants::RBUM_ITEM_NAME_APP_READ_ROLE.to_string());
            for (app_id, app_name) in app_items {
                if old_app_ids.contains(&app_id) {
                    continue;
                }
                apps.push(IamAccountAppInfoResp {
                    app_id: app_id.clone(),
                    app_name: app_name.clone(),
                    app_kind: IamAppKind::Project,
                    app_own_paths: format!("{}/{}", ctx.own_paths, app_id),
                    app_icon: "".to_string(),
                    roles: app_role_read.clone(),
                    groups: HashMap::default(),
                });
            }
        }
        result.apps = apps;
        ctx.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Find Account By Ak
    /// 通过Ak查找帐户
    ///
    /// if kind is none,query default kind(UserPwd)
    /// 如果kind为空，则查询默认kind(UserPwd)
    #[oai(path = "/:ak/ak", method = "get")]
    async fn find_account_by_ak(
        &self,
        ak: Path<String>,
        kind: Query<Option<String>>,
        tenant_id: Query<Option<String>>,
        supplier: Query<Option<String>>,
        mut ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<Option<IamAccountDetailResp>> {
        let funs = iam_constants::get_tardis_inst();
        check_without_owner_and_unsafe_fill_ctx(request, &funs, &mut ctx.0)?;
        let ctx = IamCertServ::try_use_tenant_ctx(ctx.0, tenant_id.0.clone())?;
        try_set_real_ip_from_req_to_ctx(request, &ctx).await?;
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
                &ctx,
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
                        &ctx,
                    )
                    .await?,
                )
            } else {
                None
            }
        } else {
            None
        };

        ctx.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Find Account By ThirdParty Cert ak
    /// 通过三方凭证ak查找帐户
    ///
    #[oai(path = "/:supplier/:ak/third-party", method = "get")]
    async fn find_by_third_party(&self, supplier: Path<String>, ak: Path<String>, mut ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<IamAccountDetailResp> {
        let funs = iam_constants::get_tardis_inst();
        check_without_owner_and_unsafe_fill_ctx(request, &funs, &mut ctx.0)?;
        let cert = IamCertServ::get_3th_kind_cert_by_ak(&supplier.0, &ak.0, true, &funs, &ctx.0).await?;
        let result = IamAccountServ::get_item(
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
        .await?;

        TardisResp::ok(result)
    }

    /// Batch Find Account By ThirdParty Cert ak
    /// 通过三方凭证ak批量查找帐户
    ///
    #[oai(path = "/batch-third-party", method = "get")]
    async fn batch_by_third_party(
        &self,
        supplier: Query<String>,
        aks: Query<String>,
        mut ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<Vec<IamAccountDetailResp>> {
        let funs = iam_constants::get_tardis_inst();
        check_without_owner_and_unsafe_fill_ctx(request, &funs, &mut ctx.0)?;
        let mut result = vec![];
        for ak in aks.0.split(',') {
            let cert = IamCertServ::get_3th_kind_cert_by_ak(&supplier.0, ak, true, &funs, &ctx.0).await?;
            result.push(
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
            );
        }

        TardisResp::ok(result)
    }

    /// Find App Set Items (Account) ctx
    /// 查找应用集合项（帐户）上下文
    #[oai(path = "/apps/item/ctx", method = "get")]
    async fn find_items_ctx(
        &self,
        cate_ids: Query<Option<String>>,
        item_ids: Query<Option<String>>,
        mut ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<Vec<RbumSetItemDetailResp>> {
        let funs = iam_constants::get_tardis_inst();
        check_without_owner_and_unsafe_fill_ctx(request, &funs, &mut ctx.0)?;
        let ctx = IamCertServ::use_sys_or_tenant_ctx_unsafe(ctx.0)?;
        try_set_real_ip_from_req_to_ctx(request, &ctx).await?;
        let set_id = IamSetServ::get_default_set_id_by_ctx(&IamSetKind::Apps, &funs, &ctx).await?;
        let cate_codes = RbumSetItemServ::find_detail_rbums(
            &RbumSetItemFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: false,
                    ..Default::default()
                },
                rel_rbum_item_disabled: Some(false),
                rel_rbum_item_can_not_exist: Some(true),
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
        .map(|resp| resp.rel_rbum_set_cate_sys_code.unwrap_or("".to_string()))
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
                rel_rbum_item_can_not_exist: Some(true),
                rel_rbum_set_id: Some(set_id.clone()),
                rel_rbum_set_cate_sys_codes: Some(cate_codes),
                sys_code_query_kind: Some(RbumSetCateLevelQueryKind::CurrentAndSub),
                rel_rbum_item_kind_ids: Some(vec![funs.iam_basic_kind_account_id()]),
                rel_rbum_set_cate_ids: cate_ids.0.map(|ids| ids.split(',').map(|id| id.to_string()).collect::<Vec<String>>()),
                rel_rbum_item_ids: item_ids.0.map(|ids| ids.split(',').map(|id| id.to_string()).collect::<Vec<String>>()),
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

    /// Find App Set Items (Account)
    /// 查找应用集合项（帐户）
    #[oai(path = "/apps/item", method = "get")]
    async fn find_items(
        &self,
        cate_sys_codes: Query<Option<String>>,
        sys_code_query_kind: Query<Option<RbumSetCateLevelQueryKind>>,
        sys_code_query_depth: Query<Option<i16>>,
        cate_ids: Query<Option<String>>,
        item_ids: Query<Option<String>>,
        mut ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<Vec<RbumSetItemDetailResp>> {
        let funs = iam_constants::get_tardis_inst();
        check_without_owner_and_unsafe_fill_ctx(request, &funs, &mut ctx.0)?;
        let ctx = IamCertServ::use_sys_or_tenant_ctx_unsafe(ctx.0)?;
        try_set_real_ip_from_req_to_ctx(request, &ctx).await?;
        let set_id = IamSetServ::get_default_set_id_by_ctx(&IamSetKind::Apps, &funs, &ctx).await?;
        let result = RbumSetItemServ::find_detail_rbums(
            &RbumSetItemFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: false,
                    ..Default::default()
                },
                rel_rbum_item_disabled: Some(false),
                rel_rbum_item_can_not_exist: Some(true),
                rel_rbum_set_id: Some(set_id.clone()),
                rel_rbum_set_cate_sys_codes: cate_sys_codes.0.map(|codes| codes.split(',').map(|code| code.to_string()).collect::<Vec<String>>()),
                sys_code_query_kind: sys_code_query_kind.0,
                sys_code_query_depth: sys_code_query_depth.0,
                rel_rbum_set_cate_ids: cate_ids.0.map(|ids| ids.split(',').map(|id| id.to_string()).collect::<Vec<String>>()),
                rel_rbum_item_ids: item_ids.0.map(|ids| ids.split(',').map(|id| id.to_string()).collect::<Vec<String>>()),
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

    /// Batch Bind Account To Role
    /// 批量绑定账号到角色
    #[oai(path = "/batch/bind_role", method = "put")]
    async fn batch_bind_role(
        &self,
        bind_reqs: Json<Vec<IamAccountBindRoleReq>>,
        mut ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        check_without_owner_and_unsafe_fill_ctx(request, &funs, &mut ctx.0)?;
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        funs.begin().await?;
        let app_id = IamAppServ::get_id_by_ctx(&ctx.0, &funs)?;
        for bind_req in bind_reqs.0 {
            IamAppServ::add_rel_account(&app_id, &bind_req.account_id, true, &funs, &ctx.0).await?;
            IamRoleServ::add_rel_account(&bind_req.role_id, &bind_req.account_id, Some(RBUM_SCOPE_LEVEL_APP), &funs, &ctx.0).await?;
        }
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }
}
