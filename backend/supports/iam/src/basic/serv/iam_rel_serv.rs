use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::chrono::{Duration, Utc};
use tardis::web::web_resp::TardisPage;
use tardis::TardisFunsInst;

use bios_basic::rbum::dto::rbum_filer_dto::{RbumBasicFilterReq, RbumRelFilterReq};
use bios_basic::rbum::dto::rbum_rel_agg_dto::{RbumRelAggAddReq, RbumRelAggResp, RbumRelEnvAggAddReq};
use bios_basic::rbum::dto::rbum_rel_dto::{RbumRelAddReq, RbumRelBoneResp, RbumRelSimpleFindReq};
use bios_basic::rbum::rbum_enumeration::{RbumRelEnvKind, RbumRelFromKind};
use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;
use bios_basic::rbum::serv::rbum_rel_serv::RbumRelServ;

use crate::basic::dto::iam_filer_dto::{IamAccountFilterReq, IamResFilterReq, IamRoleFilterReq};
use crate::basic::serv::iam_cert_serv::IamCertServ;
use crate::basic::serv::iam_key_cache_serv::{IamCacheResRelAddOrModifyReq, IamCacheResRelDeleteReq, IamIdentCacheServ, IamResCacheServ};
use crate::basic::serv::iam_res_serv::IamResServ;
use crate::iam_enumeration::{IamRelKind, IamResKind};

use super::clients::iam_log_client::{IamLogClient, LogParamTag};
use super::clients::iam_search_client::IamSearchClient;
use super::iam_account_serv::IamAccountServ;
use super::iam_role_serv::IamRoleServ;

pub struct IamRelServ;

/// Example of role and resource relationship:
///
///  +------------------(3)--------------------+
///  |                                         |
/// API1------(1)---- ResMenu1------(2)----- Role1
///  |                                         |
///  +--------(4)-----ResMenu2 -----(5)--------+
impl IamRelServ {
    pub async fn add_simple_rel(
        rel_kind: &IamRelKind,
        from_iam_item_id: &str,
        to_iam_item_id: &str,
        start_timestamp: Option<i64>,
        end_timestamp: Option<i64>,
        ignore_exist_error: bool,
        to_is_outside: bool,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<()> {
        if Self::exist_rels(rel_kind, from_iam_item_id, to_iam_item_id, funs, ctx).await? {
            return if ignore_exist_error {
                Ok(())
            } else {
                Err(funs.err().conflict(&rel_kind.to_string(), "add_simple_rel", "associated already exists", "409-rbum-rel-exist"))
            };
        }
        let value1 = start_timestamp.unwrap_or_else(|| Utc::now().timestamp());
        let value2 = end_timestamp.unwrap_or_else(|| (Utc::now() + Duration::try_days(365 * 100).expect("TimeDelta::days out of bounds")).timestamp());
        let req = &mut RbumRelAggAddReq {
            rel: RbumRelAddReq {
                tag: rel_kind.to_string(),
                note: None,
                from_rbum_kind: RbumRelFromKind::Item,
                from_rbum_id: from_iam_item_id.to_string(),
                to_rbum_item_id: to_iam_item_id.to_string(),
                to_own_paths: ctx.own_paths.to_string(),
                to_is_outside,
                ext: None,
            },
            attrs: vec![],
            envs: if start_timestamp.is_some() || end_timestamp.is_some() {
                vec![RbumRelEnvAggAddReq {
                    kind: RbumRelEnvKind::DatetimeRange,
                    value1: value1.to_string(),
                    value2: Some(value2.to_string()),
                }]
            } else {
                vec![]
            },
        };
        RbumRelServ::add_rel(req, funs, ctx).await?;

        if rel_kind == &IamRelKind::IamAccountRole {
            IamIdentCacheServ::refresh_account_info_by_account_id(from_iam_item_id, funs).await?;
            let account_name = IamAccountServ::get_item(
                from_iam_item_id,
                &IamAccountFilterReq {
                    basic: RbumBasicFilterReq {
                        ignore_scope: true,
                        with_sub_own_paths: true,
                        own_paths: Some("".to_string()),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                funs,
                ctx,
            )
            .await?
            .name;
            let role_name = IamRoleServ::get_item(
                to_iam_item_id,
                &IamRoleFilterReq {
                    basic: RbumBasicFilterReq {
                        ignore_scope: true,
                        with_sub_own_paths: true,
                        own_paths: Some("".to_string()),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                funs,
                ctx,
            )
            .await?
            .name;
            let _ = IamLogClient::add_ctx_task(
                LogParamTag::IamAccount,
                Some(from_iam_item_id.to_string()),
                format!("账号{}增加{}角色", account_name, role_name),
                Some("AddTenantRoleAsAdmin".to_string()),
                ctx,
            )
            .await;

            let _ = IamLogClient::add_ctx_task(
                LogParamTag::IamRole,
                Some(to_iam_item_id.to_string()),
                format!("添加角色人员{}", account_name),
                Some("AddRoleAccount".to_string()),
                ctx,
            )
            .await;
            IamSearchClient::async_add_or_modify_account_search(from_iam_item_id, Box::new(true), "", funs, ctx).await?;
        }
        if rel_kind == &IamRelKind::IamAccountApp {
            IamSearchClient::async_add_or_modify_account_search(from_iam_item_id, Box::new(true), "", funs, ctx).await?;
        }
        if rel_kind == &IamRelKind::IamSubDeployAccount {
            IamSearchClient::sync_add_or_modify_account_search(to_iam_item_id, Box::new(true), "", funs, ctx).await?;
        }
        if rel_kind == &IamRelKind::IamSubDeployAuthAccount {
            IamSearchClient::sync_add_or_modify_account_search(to_iam_item_id, Box::new(true), "", funs, ctx).await?;
        }

        if rel_kind == &IamRelKind::IamResRole {
            let res_id = from_iam_item_id;
            let role_id = to_iam_item_id;
            let res = IamResServ::peek_item(
                res_id,
                &IamResFilterReq {
                    basic: RbumBasicFilterReq {
                        with_sub_own_paths: true,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                funs,
                ctx,
            )
            .await?;
            if res.kind == IamResKind::Api {
                // If add is an API resource, the API resource is bound to the role in the cache
                // See example (3)
                IamResCacheServ::add_or_modify_res_rel(
                    &res.code,
                    &res.method,
                    &IamCacheResRelAddOrModifyReq {
                        st: if start_timestamp.is_some() { Some(value1) } else { None },
                        et: if end_timestamp.is_some() { Some(value2) } else { None },
                        accounts: vec![],
                        roles: vec![role_id.to_string()],
                        groups: vec![],
                        apps: vec![],
                        tenants: vec![],
                        aks: vec![],
                        need_crypto_req: None,
                        need_crypto_resp: None,
                        need_double_auth: None,
                        need_login: None,
                    },
                    funs,
                )
                .await?;
            } else {
                // If add is a menu or element resource
                // See example (2) / (5)
                // 1) Find the list of associated API resources
                let sys_ctx = IamCertServ::use_sys_ctx_unsafe(ctx.clone())?;
                let rel_res_api_ids = Self::find_to_id_rels(&IamRelKind::IamResApi, res_id, None, None, funs, &sys_ctx).await?;
                let rel_res_apis = IamResServ::find_items(
                    &IamResFilterReq {
                        basic: RbumBasicFilterReq {
                            ids: Some(rel_res_api_ids),
                            with_sub_own_paths: true,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    None,
                    None,
                    funs,
                    &sys_ctx,
                )
                .await?;
                // 2) Create bindings of associated API resources to roles in the cache
                for rel_res_api in rel_res_apis {
                    IamResCacheServ::add_or_modify_res_rel(
                        &rel_res_api.code,
                        &rel_res_api.method,
                        &IamCacheResRelAddOrModifyReq {
                            st: if start_timestamp.is_some() { Some(value1) } else { None },
                            et: if end_timestamp.is_some() { Some(value2) } else { None },
                            accounts: vec![],
                            roles: vec![role_id.to_string()],
                            groups: vec![],
                            apps: vec![],
                            tenants: vec![],
                            aks: vec![],
                            need_crypto_req: None,
                            need_crypto_resp: None,
                            need_double_auth: None,
                            need_login: None,
                        },
                        funs,
                    )
                    .await?;
                }
            }
        } else if rel_kind == &IamRelKind::IamResApi {
            let res_api_id = from_iam_item_id;
            let res_other_id = to_iam_item_id;
            let res_api = IamResServ::peek_item(
                res_api_id,
                &IamResFilterReq {
                    basic: RbumBasicFilterReq {
                        with_sub_own_paths: true,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                funs,
                ctx,
            )
            .await?;
            let res_other = IamResServ::peek_item(
                res_other_id,
                &IamResFilterReq {
                    basic: RbumBasicFilterReq {
                        with_sub_own_paths: true,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                funs,
                ctx,
            )
            .await?;
            if res_api.kind != IamResKind::Api {
                return Err(funs.err().conflict("iam_rel", "add", "when add IamResApi kind from item must be api kind", "409-iam-rel-kind-api-conflict"));
            }
            // See example (1) / (4)
            // Find the list of roles associated with a menu or element resource
            let sys_ctx = IamCertServ::use_sys_ctx_unsafe(ctx.clone())?;
            let rel_role_ids = Self::find_from_id_rels(&IamRelKind::IamResRole, true, res_other_id, None, None, funs, &sys_ctx).await?;
            // Create API bindings to associated roles in the cache
            IamResCacheServ::add_or_modify_res_rel(
                &res_api.code,
                &res_api.method,
                &IamCacheResRelAddOrModifyReq {
                    st: if start_timestamp.is_some() { Some(value1) } else { None },
                    et: if end_timestamp.is_some() { Some(value2) } else { None },
                    accounts: vec![],
                    roles: rel_role_ids,
                    groups: vec![],
                    apps: vec![],
                    tenants: vec![],
                    aks: vec![],
                    need_crypto_req: None,
                    need_crypto_resp: None,
                    need_double_auth: None,
                    need_login: None,
                },
                funs,
            )
            .await?;

            let (op_describe, op_kind) = match res_other.kind {
                IamResKind::Api => ("", ""),
                IamResKind::Ele => ("添加操作API", "AddElementApi"),
                IamResKind::Menu => ("添加目录页面API", "AddContentPageApi"),
                _ => ("", ""),
            };
            let _ = IamLogClient::add_ctx_task(
                LogParamTag::IamRes,
                Some(from_iam_item_id.to_string()),
                op_describe.to_string(),
                Some(op_kind.to_string()),
                ctx,
            )
            .await;
        }

        Ok(())
    }

    pub async fn delete_simple_rel(rel_kind: &IamRelKind, from_iam_item_id: &str, to_iam_item_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let rel_ids = RbumRelServ::find_id_rbums(
            &RbumRelFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                tag: Some(rel_kind.to_string()),
                from_rbum_kind: Some(RbumRelFromKind::Item),
                from_rbum_id: Some(from_iam_item_id.to_string()),
                to_rbum_item_id: Some(to_iam_item_id.to_string()),
                ..Default::default()
            },
            None,
            None,
            funs,
            ctx,
        )
        .await?;
        if rel_ids.is_empty() {
            return Ok(());
        }
        for rel_id in rel_ids {
            RbumRelServ::delete_rbum(&rel_id, funs, ctx).await?;
        }
        match rel_kind {
            IamRelKind::IamResRole => {
                let res_id = from_iam_item_id;
                let role_id = to_iam_item_id;
                let res = IamResServ::peek_item(
                    res_id,
                    &IamResFilterReq {
                        basic: RbumBasicFilterReq {
                            with_sub_own_paths: true,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    funs,
                    ctx,
                )
                .await?;
                if res.kind == IamResKind::Api {
                    // If delete is an API resource, remove the API resource binding to the role from the cache
                    // See example (3)
                    IamResCacheServ::delete_res_rel(
                        &res.code,
                        &res.method,
                        &IamCacheResRelDeleteReq {
                            accounts: vec![],
                            roles: vec![role_id.to_string()],
                            groups: vec![],
                            apps: vec![],
                            tenants: vec![],
                            aks: vec![],
                        },
                        funs,
                    )
                    .await?;
                } else {
                    // If delete is a menu or element resource
                    // See example (2) / (5)
                    // 1) Find the list of associated API resources (ready to remove the binding to the role from the cache)
                    let sys_ctx = IamCertServ::use_sys_ctx_unsafe(ctx.clone())?;
                    let rel_res_api_ids = Self::find_to_id_rels(&IamRelKind::IamResApi, res_id, None, None, funs, &sys_ctx).await?;
                    for rel_res_api_id in rel_res_api_ids {
                        // 2) If the associated API resource is explicitly associated with a role, it cannot be removed
                        if Self::exist_rels(&IamRelKind::IamResRole, &rel_res_api_id, role_id, funs, &sys_ctx).await? {
                            continue;
                        }
                        // 3) Find the list of menu or element resources associated with the associated API resource (indirect relationship)
                        let rel_res_other_ids = Self::find_from_id_rels(&IamRelKind::IamResApi, true, &rel_res_api_id, None, None, funs, &sys_ctx)
                            .await?
                            .into_iter()
                            // 4) Exclude own Id
                            .filter(|r| r != res_id)
                            .collect::<Vec<String>>();
                        // 5) If these associated menus or element resources are explicitly associated with a role, they cannot be removed
                        for rel_res_other_id in rel_res_other_ids {
                            if Self::exist_rels(&IamRelKind::IamResRole, &rel_res_other_id, role_id, funs, &sys_ctx).await? {
                                break;
                            }
                        }
                        // 6) Get information about the resources
                        let rel_res_api = IamResServ::peek_item(
                            &rel_res_api_id,
                            &IamResFilterReq {
                                basic: RbumBasicFilterReq {
                                    with_sub_own_paths: true,
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            funs,
                            &sys_ctx,
                        )
                        .await?;
                        // 7) Remove API resources from binding to roles in the cache
                        IamResCacheServ::delete_res_rel(
                            &rel_res_api.code,
                            &rel_res_api.method,
                            &IamCacheResRelDeleteReq {
                                accounts: vec![],
                                roles: vec![role_id.to_string()],
                                groups: vec![],
                                apps: vec![],
                                tenants: vec![],
                                aks: vec![],
                            },
                            funs,
                        )
                        .await?;
                    }
                }
            }
            IamRelKind::IamResApi => {
                let res_api_id = from_iam_item_id;
                let res_other_id = to_iam_item_id;
                let res_api = IamResServ::peek_item(
                    res_api_id,
                    &IamResFilterReq {
                        basic: RbumBasicFilterReq {
                            with_sub_own_paths: true,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    funs,
                    ctx,
                )
                .await?;
                let res_other = IamResServ::peek_item(
                    res_other_id,
                    &IamResFilterReq {
                        basic: RbumBasicFilterReq {
                            with_sub_own_paths: true,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    funs,
                    ctx,
                )
                .await?;
                if res_api.kind != IamResKind::Api {
                    return Err(funs.err().conflict(
                        "iam_rel",
                        "delete",
                        "when delete IamResApi kind from item must be api kind",
                        "409-iam-rel-kind-api-conflict",
                    ));
                }
                // See example (1) / (4)
                // 1) Find the list of roles associated with a menu or element resource (ready to remove the binding to the API resource from the cache)
                let sys_ctx = IamCertServ::use_sys_ctx_unsafe(ctx.clone())?;
                let rel_role_ids = Self::find_from_id_rels(&IamRelKind::IamResRole, true, res_other_id, None, None, funs, &sys_ctx).await?;
                let rel_api_role_ids = Self::find_from_id_rels(&IamRelKind::IamResRole, true, res_api_id, None, None, funs, &sys_ctx).await?;
                let rel_api_res_ids = Self::find_from_id_rels(&IamRelKind::IamResApi, true, res_api_id, None, None, funs, &sys_ctx).await?;
                let mut remove_role_ids = Vec::new();
                for rel_role_id in rel_role_ids {
                    // 2) If an API resource is explicitly associated with a role, it cannot be removed
                    if rel_api_role_ids.contains(&rel_role_id) {
                        continue;
                    }
                    // 3) Find the list of resources associated with the associated role (indirect relationship)
                    let rel_res_ids = Self::find_to_id_rels(&IamRelKind::IamResRole, &rel_role_id, None, None, funs, &sys_ctx)
                        .await?
                        .into_iter()
                        // 4) Exclude own Id
                        .filter(|r| r != res_other_id)
                        .collect::<Vec<String>>();
                    // 5) If these associated resources are explicitly associated with API resources, they cannot be removed
                    let mut res_bound = false;
                    for rel_res_id in rel_res_ids {
                        if rel_api_res_ids.contains(&rel_res_id) {
                            res_bound = true;
                            break;
                        }
                    }
                    if !res_bound {
                        remove_role_ids.push(rel_role_id);
                    }
                }
                // 6) Remove API resources from binding to roles in the cache
                IamResCacheServ::delete_res_rel(
                    &res_api.code,
                    &res_api.method,
                    &IamCacheResRelDeleteReq {
                        accounts: vec![],
                        roles: remove_role_ids,
                        groups: vec![],
                        apps: vec![],
                        tenants: vec![],
                        aks: vec![],
                    },
                    funs,
                )
                .await?;

                let (op_describe, op_kind) = match res_other.kind {
                    IamResKind::Api => ("", ""),
                    IamResKind::Ele => ("移除操作API", "RemoveElementApi"),
                    IamResKind::Menu => ("移除目录页面API", "RemoveContentPageApi"),
                    _ => ("", ""),
                };
                let _ = IamLogClient::add_ctx_task(
                    LogParamTag::IamRes,
                    Some(from_iam_item_id.to_string()),
                    op_describe.to_string(),
                    Some(op_kind.to_string()),
                    ctx,
                )
                .await;
            }
            IamRelKind::IamAccountRole => {
                IamIdentCacheServ::refresh_account_info_by_account_id(from_iam_item_id, funs).await?;
                let account_name = IamAccountServ::get_item(
                    from_iam_item_id,
                    &IamAccountFilterReq {
                        basic: RbumBasicFilterReq {
                            ignore_scope: true,
                            with_sub_own_paths: true,
                            own_paths: Some("".to_string()),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    funs,
                    ctx,
                )
                .await?
                .name;
                let role_name = IamRoleServ::get_item(
                    to_iam_item_id,
                    &IamRoleFilterReq {
                        basic: RbumBasicFilterReq {
                            ignore_scope: true,
                            with_sub_own_paths: true,
                            own_paths: Some("".to_string()),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    funs,
                    ctx,
                )
                .await?
                .name;
                let _ = IamLogClient::add_ctx_task(
                    LogParamTag::IamAccount,
                    Some(from_iam_item_id.to_string()),
                    format!("账号{}被移除{}角色", account_name, role_name),
                    Some("RemoveTenantRoleAsAdmin".to_string()),
                    ctx,
                )
                .await;
                let _ = IamLogClient::add_ctx_task(
                    LogParamTag::IamRole,
                    Some(to_iam_item_id.to_string()),
                    format!("移除角色人员{}", account_name),
                    Some("RemoveRoleAccount".to_string()),
                    ctx,
                )
                .await;
                IamSearchClient::async_add_or_modify_account_search(from_iam_item_id, Box::new(true), "", funs, ctx).await?;
                // TODO reset account cache
                // let tenant_ctx = IamCertServ::use_sys_or_tenant_ctx_unsafe(ctx.clone())?;
                // IamCertServ::package_tardis_account_context_and_resp(from_iam_item_id, &tenant_ctx.own_paths, "".to_string(), None, funs, &tenant_ctx).await?;
            }
            IamRelKind::IamAccountApp => {
                // TODO reset account cache
                // IamIdentCacheServ::delete_tokens_and_contexts_by_account_id(from_iam_item_id, get_real_ip_from_ctx(ctx).await?, funs).await?;
                IamIdentCacheServ::refresh_account_info_by_account_id(from_iam_item_id, funs).await?;
                IamSearchClient::async_add_or_modify_account_search(from_iam_item_id, Box::new(true), "", funs, ctx).await?;
            }
            IamRelKind::IamAccountRel => {
                IamSearchClient::async_add_or_modify_account_search(from_iam_item_id, Box::new(true), "", funs, ctx).await?;
            }
            IamRelKind::IamSubDeployAccount => {
                IamSearchClient::sync_add_or_modify_account_search(to_iam_item_id, Box::new(true), "", funs, ctx).await?;
            }
            IamRelKind::IamSubDeployAuthAccount => {
                IamSearchClient::sync_add_or_modify_account_search(to_iam_item_id, Box::new(true), "", funs, ctx).await?;
            }
            _ => {}
        }
        Ok(())
    }

    pub async fn count_from_rels(rel_kind: &IamRelKind, with_sub: bool, from_iam_item_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<u64> {
        RbumRelServ::count_from_rels(&rel_kind.to_string(), &RbumRelFromKind::Item, with_sub, from_iam_item_id, funs, ctx).await
    }

    pub async fn find_from_id_rels(
        rel_kind: &IamRelKind,
        with_sub: bool,
        from_iam_item_id: &str,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<Vec<String>> {
        RbumRelServ::find_from_id_rels(
            &rel_kind.to_string(),
            &RbumRelFromKind::Item,
            with_sub,
            from_iam_item_id,
            desc_sort_by_create,
            desc_sort_by_update,
            funs,
            ctx,
        )
        .await
    }

    pub async fn find_from_simple_rels(
        rel_kind: &IamRelKind,
        with_sub: bool,
        from_iam_item_id: &str,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<Vec<RbumRelBoneResp>> {
        RbumRelServ::find_from_simple_rels(
            &rel_kind.to_string(),
            &RbumRelFromKind::Item,
            with_sub,
            from_iam_item_id,
            desc_sort_by_create,
            desc_sort_by_update,
            funs,
            ctx,
        )
        .await
    }

    pub async fn paginate_from_id_rels(
        rel_kind: &IamRelKind,
        with_sub: bool,
        from_iam_item_id: &str,
        page_number: u32,
        page_size: u32,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<TardisPage<String>> {
        RbumRelServ::paginate_from_id_rels(
            &rel_kind.to_string(),
            &RbumRelFromKind::Item,
            with_sub,
            from_iam_item_id,
            page_number,
            page_size,
            desc_sort_by_create,
            desc_sort_by_update,
            funs,
            ctx,
        )
        .await
    }

    pub async fn paginate_from_simple_rels(
        rel_kind: &IamRelKind,
        with_sub: bool,
        from_iam_item_id: &str,
        page_number: u32,
        page_size: u32,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<TardisPage<RbumRelBoneResp>> {
        RbumRelServ::paginate_from_simple_rels(
            &rel_kind.to_string(),
            &RbumRelFromKind::Item,
            with_sub,
            from_iam_item_id,
            page_number,
            page_size,
            desc_sort_by_create,
            desc_sort_by_update,
            funs,
            ctx,
        )
        .await
    }

    pub async fn count_to_rels(rel_kind: &IamRelKind, to_iam_item_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<u64> {
        RbumRelServ::count_to_rels(&rel_kind.to_string(), to_iam_item_id, funs, ctx).await
    }

    pub async fn find_to_id_rels(
        rel_kind: &IamRelKind,
        to_iam_item_id: &str,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<Vec<String>> {
        RbumRelServ::find_to_id_rels(&rel_kind.to_string(), to_iam_item_id, desc_sort_by_create, desc_sort_by_update, funs, ctx).await
    }

    pub async fn find_to_simple_rels(
        rel_kind: &IamRelKind,
        to_iam_item_id: &str,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<Vec<RbumRelBoneResp>> {
        RbumRelServ::find_to_simple_rels(&rel_kind.to_string(), to_iam_item_id, desc_sort_by_create, desc_sort_by_update, funs, ctx).await
    }

    pub async fn find_simple_rels(
        filter: &RbumRelFilterReq,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        is_from: bool,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<Vec<RbumRelBoneResp>> {
        RbumRelServ::find_simple_rels(filter, desc_sort_by_create, desc_sort_by_update, is_from, funs, ctx).await
    }

    pub async fn find_rels(
        filter: &RbumRelFilterReq,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<Vec<RbumRelAggResp>> {
        RbumRelServ::find_rels(filter, desc_sort_by_create, desc_sort_by_update, funs, ctx).await
    }

    pub async fn paginate_to_id_rels(
        rel_kind: &IamRelKind,
        to_iam_item_id: &str,
        page_number: u32,
        page_size: u32,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<TardisPage<String>> {
        RbumRelServ::paginate_to_id_rels(
            &rel_kind.to_string(),
            to_iam_item_id,
            page_number,
            page_size,
            desc_sort_by_create,
            desc_sort_by_update,
            funs,
            ctx,
        )
        .await
    }

    pub async fn paginate_to_simple_rels(
        rel_kind: &IamRelKind,
        to_iam_item_id: &str,
        page_number: u32,
        page_size: u32,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<TardisPage<RbumRelBoneResp>> {
        RbumRelServ::paginate_to_simple_rels(
            &rel_kind.to_string(),
            to_iam_item_id,
            page_number,
            page_size,
            desc_sort_by_create,
            desc_sort_by_update,
            funs,
            ctx,
        )
        .await
    }

    pub async fn exist_rels(rel_kind: &IamRelKind, from_iam_item_id: &str, to_iam_item_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<bool> {
        // TODO In-depth inspection
        RbumRelServ::check_simple_rel(
            &RbumRelSimpleFindReq {
                tag: Some(rel_kind.to_string()),
                from_rbum_kind: Some(RbumRelFromKind::Item),
                from_rbum_id: Some(from_iam_item_id.to_string()),
                to_rbum_item_id: Some(to_iam_item_id.to_string()),
                from_own_paths: Some(ctx.own_paths.to_string()),
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await
    }

    pub async fn exist_from_rels(rel_kind: &IamRelKind, from_iam_item_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<bool> {
        RbumRelServ::check_simple_rel(
            &RbumRelSimpleFindReq {
                tag: Some(rel_kind.to_string()),
                from_rbum_kind: Some(RbumRelFromKind::Item),
                from_rbum_id: Some(from_iam_item_id.to_string()),
                from_own_paths: Some(ctx.own_paths.to_string()),
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await
    }

    pub async fn exist_to_rel(rel_kind: &IamRelKind, to_iam_item_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<bool> {
        RbumRelServ::check_simple_rel(
            &RbumRelSimpleFindReq {
                tag: Some(rel_kind.to_string()),
                to_rbum_item_id: Some(to_iam_item_id.to_string()),
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await
    }
}
