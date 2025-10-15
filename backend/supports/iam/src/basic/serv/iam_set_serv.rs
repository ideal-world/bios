use std::collections::{HashMap, HashSet};

use bios_basic::rbum::dto::rbum_filer_dto::{RbumBasicFilterReq, RbumRelFilterReq, RbumSetCateFilterReq, RbumSetFilterReq, RbumSetItemFilterReq, RbumSetTreeFilterReq};

use bios_basic::rbum::dto::rbum_set_cate_dto::{RbumSetCateAddReq, RbumSetCateModifyReq, RbumSetCateSummaryResp};
use bios_basic::rbum::dto::rbum_set_dto::{RbumSetAddReq, RbumSetPathResp, RbumSetTreeNodeResp, RbumSetTreeResp};
use bios_basic::rbum::dto::rbum_set_item_dto::{RbumSetItemAddReq, RbumSetItemDetailResp, RbumSetItemModifyReq, RbumSetItemRelInfoResp};
use bios_basic::rbum::helper::rbum_scope_helper;
use bios_basic::rbum::rbum_config::RbumConfigApi;
use bios_basic::rbum::rbum_enumeration::{RbumRelFromKind, RbumScopeLevelKind, RbumSetCateLevelQueryKind};
use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;
use bios_basic::rbum::serv::rbum_rel_serv::RbumRelServ;
use bios_basic::rbum::serv::rbum_set_serv::{RbumSetCateServ, RbumSetItemServ, RbumSetServ};
use itertools::Itertools;
use tardis::basic::dto::TardisContext;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;

use tardis::serde_json::json;
use tardis::web::web_resp::TardisPage;
use tardis::{TardisFuns, TardisFunsInst};

use crate::basic::dto::iam_filer_dto::IamAccountFilterReq;
use crate::basic::dto::iam_set_dto::{IamResSetTreeExtResp, IamResSetTreeResp, IamSetCateAddReq, IamSetCateModifyReq, IamSetItemAddReq};
use crate::iam_config::{IamBasicConfigApi, IamConfig};
use crate::iam_constants::{RBUM_SCOPE_LEVEL_APP, RBUM_SCOPE_LEVEL_TENANT};
use crate::iam_enumeration::{IamRelKind, IamSetCateKind, IamSetKind};

use super::clients::iam_kv_client::IamKvClient;
use super::clients::iam_log_client::{IamLogClient, LogParamTag};
use super::clients::iam_search_client::IamSearchClient;
use super::clients::iam_stats_client::IamStatsClient;
use super::iam_account_serv::IamAccountServ;
use super::iam_cert_serv::IamCertServ;
use super::iam_rel_serv::IamRelServ;
use super::iam_sub_deploy_serv::IamSubDeployServ;

const SET_AND_ITEM_SPLIT_FLAG: &str = ":";

pub const MENU_ROOT_SET_BUS_CODE: &str = "__menus__";
pub const API_ROOT_SET_BUS_CODE: &str = "__apis__";
pub const DATA_GUARD_ROOT_SET_BUS_CODE: &str = "__data_guards__";

pub struct IamSetServ;

impl IamSetServ {
    pub async fn init_set(set_kind: IamSetKind, scope_level: RbumScopeLevelKind, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<(String, Option<(String, String)>)> {
        let code = Self::get_default_code(&set_kind, &ctx.own_paths);
        let set_id = RbumSetServ::add_rbum(
            &mut RbumSetAddReq {
                code: TrimString(code.clone()),
                kind: TrimString(set_kind.to_string()),
                name: TrimString(code),
                note: None,
                icon: None,
                sort: None,
                ext: None,
                scope_level: Some(scope_level.clone()),
                disabled: None,
                id: None,
            },
            funs,
            ctx,
        )
        .await?;
        let cates = match set_kind {
            IamSetKind::Res => {
                let cate_menu_id = RbumSetCateServ::add_rbum(
                    &mut RbumSetCateAddReq {
                        name: TrimString("Menus".to_string()),
                        bus_code: TrimString(MENU_ROOT_SET_BUS_CODE.to_string()),
                        icon: None,
                        sort: None,
                        ext: Some(IamSetCateKind::Root.to_string()),
                        rbum_parent_cate_id: None,
                        rel_rbum_set_id: set_id.clone(),
                        scope_level: Some(scope_level.clone()),
                        id: None,
                    },
                    funs,
                    ctx,
                )
                .await?;
                let cate_api_id = RbumSetCateServ::add_rbum(
                    &mut RbumSetCateAddReq {
                        name: TrimString("Apis".to_string()),
                        bus_code: TrimString(API_ROOT_SET_BUS_CODE.to_string()),
                        icon: None,
                        sort: None,
                        ext: None,
                        rbum_parent_cate_id: None,
                        rel_rbum_set_id: set_id.clone(),
                        scope_level: Some(scope_level.clone()),
                        id: None,
                    },
                    funs,
                    ctx,
                )
                .await?;
                let _ = RbumSetCateServ::add_rbum(
                    &mut RbumSetCateAddReq {
                        name: TrimString("DataGuards".to_string()),
                        bus_code: TrimString(DATA_GUARD_ROOT_SET_BUS_CODE.to_string()),
                        icon: None,
                        sort: None,
                        ext: None,
                        rbum_parent_cate_id: None,
                        rel_rbum_set_id: set_id.clone(),
                        scope_level: Some(scope_level.clone()),
                        id: None,
                    },
                    funs,
                    ctx,
                )
                .await?;
                Some((cate_menu_id, cate_api_id))
            }
            _ => None,
        };
        Ok((set_id, cates))
    }

    pub async fn get_default_set_id_by_ctx(kind: &IamSetKind, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<String> {
        Self::get_set_id_by_code(&Self::get_default_code(kind, &ctx.own_paths), true, funs, ctx).await
    }

    pub async fn get_set_id_by_code(code: &str, with_sub: bool, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<String> {
        RbumSetServ::get_rbum_set_id_by_code(code, with_sub, funs, ctx)
            .await?
            .ok_or_else(|| funs.err().not_found("iam_set", "get_id", &format!("not found set by code {code}"), "404-rbum-set-code-not-exist"))
    }

    pub async fn try_get_rel_ctx_by_set_id(set_id: Option<String>, funs: &TardisFunsInst, mut ctx: TardisContext) -> TardisResult<TardisContext> {
        if let Some(set_id) = set_id {
            let code = Self::get_code_ctx_by_set_id(&set_id, funs, ctx.clone())
                .await?
                .ok_or_else(|| funs.err().not_found("iam_set", "get_rel_ctx", &format!("not found set by set_id {set_id}"), "404-rbum-set-id-not-exist"))?;
            let splits = code.split(':').collect::<Vec<_>>();
            if let Some(own_paths) = splits.first() {
                ctx.own_paths = own_paths.to_string();
            }
            Ok(ctx)
        } else {
            Ok(ctx)
        }
    }

    pub async fn get_code_ctx_by_set_id(set_id: &str, funs: &TardisFunsInst, ctx: TardisContext) -> TardisResult<Option<String>> {
        let mock_ctx = TardisContext { own_paths: "".to_string(), ..ctx };
        if let Some(rbum_set) = RbumSetServ::find_one_rbum(
            &RbumSetFilterReq {
                basic: RbumBasicFilterReq {
                    ids: Some(vec![set_id.to_string()]),
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                ..Default::default()
            },
            funs,
            &mock_ctx,
        )
        .await?
        {
            Ok(Some(rbum_set.code))
        } else {
            Ok(None)
        }
    }

    pub fn get_default_org_code_by_system() -> String {
        Self::get_default_code(&IamSetKind::Org, "")
    }

    pub fn get_default_org_code_by_tenant(funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<String> {
        if let Some(own_paths) = rbum_scope_helper::get_path_item(RBUM_SCOPE_LEVEL_TENANT.to_int(), &ctx.own_paths) {
            Ok(Self::get_default_code(&IamSetKind::Org, &own_paths))
        } else {
            Err(funs.err().not_found("iam_set", "get_org_code", "not found tenant own_paths", "404-rbum-set-code-not-exist"))
        }
    }

    pub fn get_default_org_code_by_app(funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<String> {
        if let Some(own_paths) = rbum_scope_helper::get_path_item(RBUM_SCOPE_LEVEL_APP.to_int(), &ctx.own_paths) {
            Ok(Self::get_default_code(&IamSetKind::Org, &own_paths))
        } else {
            Err(funs.err().not_found("iam_set", "get_org_code", "not found app own_paths", "404-rbum-set-code-not-exist"))
        }
    }

    pub fn get_default_code(kind: &IamSetKind, own_paths: &str) -> String {
        format!("{}:{}", own_paths, kind.to_string().to_lowercase())
    }

    pub async fn add_set_cate(set_id: &str, add_req: &IamSetCateAddReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<String> {
        let result = RbumSetCateServ::add_rbum(
            &mut RbumSetCateAddReq {
                name: add_req.name.clone(),
                bus_code: add_req.bus_code.as_ref().unwrap_or(&TrimString("".to_string())).clone(),
                icon: add_req.icon.clone(),
                sort: add_req.sort,
                ext: add_req.ext.clone(),
                rbum_parent_cate_id: add_req.rbum_parent_cate_id.clone(),
                rel_rbum_set_id: set_id.to_string(),
                scope_level: add_req.scope_level.clone(),
                id: None,
            },
            funs,
            ctx,
        )
        .await;

        let item = RbumSetServ::get_rbum(set_id, &RbumSetFilterReq::default(), funs, ctx).await?;
        let mut kind = item.kind;

        if kind == IamSetKind::Apps.to_string() && result.is_ok() {
            IamKvClient::add_or_modify_key_name(&funs.conf::<IamConfig>().spi.kv_apps_prefix.clone(), &result.clone()?, &add_req.name, None, funs, ctx).await?;
        } else if kind == IamSetKind::Org.to_string() && result.is_ok() {
            IamStatsClient::async_org_fact_record_load(result.clone().unwrap(), funs, ctx).await?;
        }
        if result.is_ok() {
            kind.make_ascii_lowercase();
            let (op_describe, tag, op_kind) = match kind.as_str() {
                "org" => ("添加部门".to_string(), Some(LogParamTag::IamOrg), Some("Add".to_string())),
                "res" => ("添加目录".to_string(), Some(LogParamTag::IamRes), Some("Add".to_string())),
                _ => (String::new(), None, None),
            };

            if let Some(tag) = tag {
                let _ = IamLogClient::add_ctx_task(tag, Some(result.clone()?), op_describe, op_kind, ctx).await;
            }
        }

        result
    }

    pub async fn modify_set_cate(set_cate_id: &str, modify_req: &IamSetCateModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let result = RbumSetCateServ::modify_rbum(
            set_cate_id,
            &mut RbumSetCateModifyReq {
                bus_code: modify_req.bus_code.clone(),
                name: modify_req.name.clone(),
                icon: modify_req.icon.clone(),
                sort: modify_req.sort,
                ext: modify_req.ext.clone(),
                rbum_parent_cate_id: None,
                scope_level: modify_req.scope_level.clone(),
            },
            funs,
            ctx,
        )
        .await;
        if result.is_ok() {
            let set_cate_item = RbumSetCateServ::get_rbum(
                set_cate_id,
                &RbumSetCateFilterReq {
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
            let item = RbumSetServ::get_rbum(
                &set_cate_item.rel_rbum_set_id,
                &RbumSetFilterReq {
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
            let mut kind = item.kind;
            if kind == IamSetKind::Apps.to_string() {
                IamKvClient::add_or_modify_key_name(
                    &funs.conf::<IamConfig>().spi.kv_apps_prefix.clone(),
                    set_cate_id,
                    &set_cate_item.name.clone(),
                    None,
                    funs,
                    ctx,
                )
                .await?;
            } else if kind == IamSetKind::Org.to_string() && result.is_ok() {
                IamStatsClient::async_org_fact_record_load(set_cate_id.to_owned(), funs, ctx).await?;
            }
            kind.make_ascii_lowercase();
            match kind.as_str() {
                "org" => {
                    if let Some(name) = &modify_req.name {
                        let _ = IamLogClient::add_ctx_task(
                            LogParamTag::IamOrg,
                            Some(set_cate_id.to_string()),
                            format!("重命名部门为{}", name),
                            Some("Rename".to_string()),
                            ctx,
                        )
                        .await;
                    }
                }
                "res" => {
                    let _ = IamLogClient::add_ctx_task(
                        LogParamTag::IamRes,
                        Some(set_cate_id.to_string()),
                        "编辑目录".to_string(),
                        Some("ModifyContent".to_string()),
                        ctx,
                    )
                    .await;
                }
                _ => {}
            }
        }

        result
    }

    pub async fn move_set_cate(set_cate_id: &str, parent_set_cate_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        RbumSetCateServ::move_set_cate(set_cate_id, parent_set_cate_id, funs, ctx).await
    }

    pub async fn delete_set_cate(set_cate_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<u64> {
        let set_cate_item = RbumSetCateServ::get_rbum(
            set_cate_id,
            &RbumSetCateFilterReq {
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
        let item = RbumSetServ::get_rbum(
            &set_cate_item.rel_rbum_set_id,
            &RbumSetFilterReq {
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
        if item.kind == IamSetKind::Apps.to_string() {
            if IamSubDeployServ::exist_to_rel(&IamRelKind::IamSubDeployApps, set_cate_id, funs, ctx).await? {
                return Err(funs.err().conflict("iam_set", "delete_set_cate", &format!("apps is used by sub deploy"), "409-rbum-apps-is-used-by-sub-deploy"));
            }
        } else if item.kind == IamSetKind::Org.to_string() {
            if IamSubDeployServ::exist_to_rel(&IamRelKind::IamSubDeployOrg, set_cate_id, funs, ctx).await? {
                return Err(funs.err().conflict("iam_set", "delete_set_cate", &format!("org is used by sub deploy"), "409-rbum-org-is-used-by-sub-deploy"));
            }
        }

        let result = RbumSetCateServ::delete_rbum(set_cate_id, funs, ctx).await;

        if result.is_ok() {
            let mut kind = item.kind;
            if kind == IamSetKind::Org.to_string() {
                IamStatsClient::async_org_fact_record_remove(set_cate_id.to_owned(), funs, ctx).await?;
            }
            kind.make_ascii_lowercase();
            let (op_describe, tag, op_kind) = match kind.as_str() {
                "org" => ("删除部门".to_string(), Some(LogParamTag::IamOrg), Some("Delete".to_string())),
                "res" => ("删除目录".to_string(), Some(LogParamTag::IamRes), Some("Delete".to_string())),
                _ => (String::new(), None, None),
            };
            if let Some(tag) = tag {
                let _ = IamLogClient::add_ctx_task(tag, Some(set_cate_id.to_string()), op_describe, op_kind, ctx).await;
            }
        }

        result
    }
    pub async fn find_set_cate(
        filter: &RbumSetCateFilterReq,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<Vec<RbumSetCateSummaryResp>> {
        RbumSetCateServ::find_rbums(filter, desc_sort_by_create, desc_sort_by_update, funs, ctx).await
    }

    pub async fn get_tree(set_id: &str, filter: &mut RbumSetTreeFilterReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<RbumSetTreeResp> {
        filter.rel_rbum_item_domain_ids = Some(vec![funs.iam_basic_domain_iam_id()]);
        let resp = RbumSetServ::get_tree(set_id, filter, funs, ctx).await?;
        Self::find_rel_set_cate(resp, filter, funs, ctx).await
    }

    // find set_cate 对应的set_id,返回set_id下面set_cate
    // 返回的节点里面，如果有通过关联关系而来的cate（不属于此set_id的），会在ext中标识它真正的set_id
    async fn find_rel_set_cate(resp: RbumSetTreeResp, filter: &mut RbumSetTreeFilterReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<RbumSetTreeResp> {
        let mut result_main: Vec<RbumSetTreeNodeResp> = vec![];
        let mut resp_items = HashMap::new();
        let mut resp_item_domains = HashMap::new();
        let mut resp_item_kinds = HashMap::new();
        let mut resp_item_number_agg = HashMap::new();

        //from set_cate to find tenant_id (set_id)
        for mut r in resp.main.clone() {
            if let Some(set_rel) = RbumRelServ::find_one_rbum(
                &RbumRelFilterReq {
                    basic: RbumBasicFilterReq {
                        own_paths: Some(ctx.own_paths.clone()),
                        with_sub_own_paths: true,
                        ..Default::default()
                    },
                    tag: Some(IamRelKind::IamOrgRel.to_string()),
                    from_rbum_kind: Some(RbumRelFromKind::SetCate),
                    from_rbum_id: Some(r.id.to_string()),
                    ..Default::default()
                },
                funs,
                ctx,
            )
            .await?
            {
                let new_resp = RbumSetTreeNodeResp {
                    rel: Some(set_rel.to_rbum_item_id.clone()),
                    ..r.clone()
                };
                r = new_resp;

                let mock_ctx = TardisContext {
                    own_paths: set_rel.to_own_paths.clone(),
                    ..ctx.clone()
                };
                let mut set_filter = filter.clone();
                if set_filter.sys_codes.is_some() {
                    set_filter.sys_codes = Some(vec!["".to_string()]);
                }
                if set_filter.sys_codes.is_some() && set_filter.sys_code_query_depth == Some(1) && set_filter.sys_code_query_kind == Some(RbumSetCateLevelQueryKind::CurrentAndSub)
                {
                    //只获取一层，那么就不需要查询关联的
                    result_main.push(r);
                    continue;
                }
                let rel_set_id = IamSetServ::get_default_set_id_by_ctx(&IamSetKind::Org, funs, &mock_ctx).await?;
                let mut tenant_resp = RbumSetServ::get_tree(&rel_set_id, &set_filter, funs, &mock_ctx).await?;
                let mut resp_tenant_node: Vec<RbumSetTreeNodeResp> = tenant_resp
                    .main
                    .clone()
                    .iter()
                    .filter(|r_main| r_main.pid.is_none())
                    .map(|r_main| RbumSetTreeNodeResp {
                        pid: Some(r.id.clone()),
                        ..r_main.clone()
                    })
                    .collect();
                tenant_resp.main.retain(|r_main| r_main.pid.is_some());
                resp_tenant_node.extend(tenant_resp.main.clone());
                result_main.extend(
                    resp_tenant_node
                        .iter()
                        .map(|rm| {
                            let mut r = rm.clone();
                            r.ext = json!({"set_id":rel_set_id.clone(),"disable_import":true}).to_string();
                            r
                        })
                        .collect::<Vec<RbumSetTreeNodeResp>>(),
                );
                if set_filter.fetch_cate_item {
                    if let Some(ext_resp) = tenant_resp.ext {
                        resp_items.extend(ext_resp.items);
                        resp_item_domains.extend(ext_resp.item_domains);
                        resp_item_kinds.extend(ext_resp.item_kinds);
                        resp_item_number_agg.extend(ext_resp.item_number_agg);
                    }
                }
            }
            //把原来的resp.main完全拷贝到result_main中
            result_main.push(r);
        }
        // 向上查询 标识父级也不能显示绑定
        for rm in result_main.clone() {
            if rm.rel.is_some() {
                let mut pid = rm.pid;
                loop {
                    if pid.is_none() {
                        break;
                    }
                    if let Some(p_node) = result_main.iter_mut().find(|r| pid.is_some() && r.id == pid.clone().unwrap_or_default()) {
                        p_node.ext = json!({"disable_import":true}).to_string();
                        pid.clone_from(&p_node.pid);
                    } else {
                        break;
                    }
                }
            }
        }
        let mut result = RbumSetTreeResp { main: result_main, ext: None };
        if filter.fetch_cate_item {
            if let Some(mut ext_resp) = resp.ext.clone() {
                ext_resp.items.extend(resp_items);
                ext_resp.item_domains.extend(resp_item_domains);
                ext_resp.item_kinds.extend(resp_item_kinds);
                ext_resp.item_number_agg.extend(resp_item_number_agg);
                result.ext = Some(ext_resp);
            }
        }
        Ok(result)
    }

    pub async fn get_tree_with_auth_by_account(set_id: &str, account_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<RbumSetTreeResp> {
        let account_ids = vec![account_id.to_string()];
        let app_ids = IamRelServ::find_from_id_rels(&IamRelKind::IamAccountApp, true, account_id, None, None, funs, ctx).await?;

        let account_cate = RbumSetItemServ::find_detail_rbums(
            &RbumSetItemFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                rel_rbum_item_can_not_exist: None,
                rel_rbum_item_disabled: Some(false),
                rel_rbum_set_id: Some(set_id.to_string()),
                rel_rbum_item_ids: Some(account_ids.clone()),
                ..Default::default()
            },
            Some(true),
            None,
            funs,
            ctx,
        )
        .await?;
        let app_cate = RbumSetItemServ::find_detail_rbums(
            &RbumSetItemFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                rel_rbum_item_can_not_exist: None,
                rel_rbum_item_disabled: Some(false),
                rel_rbum_set_id: Some(set_id.to_string()),
                rel_rbum_item_ids: Some(app_ids),
                ..Default::default()
            },
            Some(true),
            None,
            funs,
            ctx,
        )
        .await?;
        let app_cate_sys_codes =
            app_cate.iter().filter(|r| r.rel_rbum_set_cate_sys_code.is_some()).map(|r| r.rel_rbum_set_cate_sys_code.clone().unwrap_or_default()).collect::<Vec<String>>();
        let account_cate_sys_codes =
            account_cate.iter().filter(|r| r.rel_rbum_set_cate_sys_code.is_some()).map(|r| r.rel_rbum_set_cate_sys_code.clone().unwrap_or_default()).collect::<Vec<String>>();
        if account_cate_sys_codes.is_empty() && app_cate_sys_codes.is_empty() {
            return Ok(RbumSetTreeResp { main: vec![], ext: None });
        }

        let account_tree_sub = if account_cate_sys_codes.is_empty() {
            RbumSetTreeResp { main: vec![], ext: None }
        } else {
            //  获取 account cate 的树
            Self::get_tree(
                set_id,
                &mut RbumSetTreeFilterReq {
                    fetch_cate_item: true,
                    sys_codes: Some(account_cate_sys_codes.clone()),
                    sys_code_query_kind: Some(RbumSetCateLevelQueryKind::CurrentAndSub),
                    ..Default::default()
                },
                funs,
                ctx,
            )
            .await?
        };
        //  补全 account cate 的 parent node
        let mut all_sys_codes = vec![];
        for cate in account_tree_sub.main.iter() {
            if cate.sys_code.is_empty() {
                continue;
            }
            let parent_sys_codes = RbumSetCateServ::get_parent_sys_codes(&cate.sys_code, funs)?;
            if !parent_sys_codes.is_empty() {
                all_sys_codes.extend(parent_sys_codes);
            }
            all_sys_codes.push(cate.sys_code.clone());
        }
        //  补全 app cate 的 parent node
        for cate_sys_code in app_cate_sys_codes.iter() {
            if cate_sys_code.is_empty() {
                continue;
            }
            let parent_sys_codes = RbumSetCateServ::get_parent_sys_codes(&cate_sys_code.clone(), funs)?;
            if !parent_sys_codes.is_empty() {
                all_sys_codes.extend(parent_sys_codes);
            }
            all_sys_codes.push(cate_sys_code.to_string());
        }
        if all_sys_codes.is_empty() {
            return Ok(RbumSetTreeResp { main: vec![], ext: None });
        }
        let all_sys_codes = all_sys_codes.iter().map(|r| r.to_string()).collect::<Vec<String>>().into_iter().collect::<HashSet<String>>().into_iter().collect::<Vec<String>>();

        Self::get_tree(
            set_id,
            &mut RbumSetTreeFilterReq {
                fetch_cate_item: true,
                sys_codes: Some(all_sys_codes),
                sys_code_query_kind: Some(RbumSetCateLevelQueryKind::Current),
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await
    }

    pub async fn get_app_with_auth_by_account(set_id: &str, account_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Vec<(String, String)>> {
        // 获取 account_id 对应的 set_cate
        let rbum_set_cate_code = RbumSetItemServ::find_detail_rbums(
            &RbumSetItemFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                rel_rbum_item_can_not_exist: None,
                rel_rbum_item_disabled: Some(false),
                rel_rbum_set_id: Some(set_id.to_string()),
                rel_rbum_item_ids: Some(vec![account_id.to_string()]),
                ..Default::default()
            },
            Some(true),
            None,
            funs,
            ctx,
        )
        .await?
        .iter()
        .filter(|r| r.rel_rbum_set_cate_sys_code.is_some())
        .map(|r| r.rel_rbum_set_cate_sys_code.clone().unwrap_or_default())
        .collect::<Vec<String>>();
        if rbum_set_cate_code.is_empty() {
            return Ok(vec![]);
        }
        let apps = RbumSetItemServ::find_detail_rbums(
            &RbumSetItemFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                rel_rbum_item_can_not_exist: None,
                rel_rbum_item_disabled: Some(false),
                rel_rbum_set_id: Some(set_id.to_string()),
                rel_rbum_item_kind_ids: Some(vec![funs.iam_basic_kind_app_id()]),
                sys_code_query_kind: Some(RbumSetCateLevelQueryKind::CurrentAndSub),
                rel_rbum_set_cate_sys_codes: Some(rbum_set_cate_code),
                ..Default::default()
            },
            Some(true),
            None,
            funs,
            ctx,
        )
        .await?
        .iter()
        .map(|r| (r.rel_rbum_item_id.clone(), r.rel_rbum_item_name.clone()))
        .collect::<Vec<(String, String)>>();
        Ok(apps)
    }

    pub async fn get_menu_tree_by_roles(set_id: &str, role_ids: &Vec<String>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<RbumSetTreeResp> {
        let set_cate_sys_code_node_len = funs.rbum_conf_set_cate_sys_code_node_len();
        let menu_sys_code = String::from_utf8(vec![b'0'; set_cate_sys_code_node_len])?;
        let mut res_ids = HashSet::new();
        let mut global_ctx = ctx.clone();
        global_ctx.own_paths = "".to_string();
        // TODO default empty res
        res_ids.insert("".to_string());
        for role_id in role_ids {
            let rel_res_ids = IamRelServ::find_to_id_rels(&IamRelKind::IamResRole, role_id, None, None, funs, &global_ctx).await?;
            res_ids.extend(rel_res_ids.into_iter());
        }
        let mut filter = RbumSetTreeFilterReq {
            fetch_cate_item: true,
            hide_cate_with_empty_item: true,
            hide_item_with_disabled: true,
            sys_codes: Some(vec![menu_sys_code]),
            sys_code_query_kind: Some(RbumSetCateLevelQueryKind::CurrentAndSub),
            ..Default::default()
        };
        if !res_ids.is_empty() {
            filter.rel_rbum_item_ids = Some(res_ids.into_iter().collect());
        }
        RbumSetServ::get_tree(set_id, &filter, funs, ctx).await
    }

    pub async fn get_menu_tree(set_id: &str, exts: Option<String>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<IamResSetTreeResp> {
        let cate_exts = exts.map(|exts| exts.split(',').map(|r| r.to_string()).collect());
        let set_cate_sys_code_node_len = funs.rbum_conf_set_cate_sys_code_node_len();
        let menu_sys_code = String::from_utf8(vec![b'0'; set_cate_sys_code_node_len])?;
        Self::transform_res_tree(
            Self::get_tree_with_sys_codes(set_id, Some(vec![menu_sys_code]), cate_exts, funs, ctx).await?,
            None,
            funs,
            ctx,
        )
        .await
    }

    pub async fn get_api_tree(set_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<RbumSetTreeResp> {
        if let Some(api_set_cate) = RbumSetCateServ::find_one_rbum(
            &RbumSetCateFilterReq {
                rel_rbum_set_id: Some(set_id.to_string()),
                bus_codes: Some(vec![API_ROOT_SET_BUS_CODE.to_string()]),
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?
        {
            Self::get_tree_with_sys_codes(set_id, Some(vec![api_set_cate.sys_code]), None, funs, ctx).await
        } else {
            Self::get_tree_with_sys_codes(set_id, None, None, funs, ctx).await
        }
    }

    pub async fn get_data_guard_tree(set_id: &str, exts: Option<String>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<RbumSetTreeResp> {
        let cate_exts = exts.map(|exts| exts.split(',').map(|r| r.to_string()).collect());
        if let Some(data_guard_set_cate) = RbumSetCateServ::find_one_rbum(
            &RbumSetCateFilterReq {
                rel_rbum_set_id: Some(set_id.to_string()),
                bus_codes: Some(vec![DATA_GUARD_ROOT_SET_BUS_CODE.to_string()]),
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?
        {
            Self::get_tree_with_sys_codes(set_id, Some(vec![data_guard_set_cate.sys_code]), cate_exts, funs, ctx).await
        } else {
            Self::get_tree_with_sys_codes(set_id, None, None, funs, ctx).await
        }
    }

    pub async fn get_cate_id_with_sys_code(set_id: &str, filter_sys_code: Option<Vec<String>>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<String> {
        let rbum_cate = RbumSetCateServ::find_one_rbum(
            &RbumSetCateFilterReq {
                rel_rbum_set_id: Some(set_id.to_string()),
                sys_codes: filter_sys_code,
                sys_code_query_kind: Some(RbumSetCateLevelQueryKind::Current),
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?;
        Ok(rbum_cate.as_ref().map(|r| r.id.clone()).unwrap_or_default())
    }

    async fn get_tree_with_sys_codes(
        set_id: &str,
        filter_sys_codes: Option<Vec<String>>,
        cate_exts: Option<Vec<String>>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<RbumSetTreeResp> {
        RbumSetServ::get_tree(
            set_id,
            &RbumSetTreeFilterReq {
                fetch_cate_item: true,
                sys_codes: filter_sys_codes,
                sys_code_query_kind: Some(RbumSetCateLevelQueryKind::CurrentAndSub),
                cate_exts,
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await
    }

    pub async fn add_set_item(add_req: &IamSetItemAddReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<String> {
        let result: Result<String, tardis::basic::error::TardisError> = RbumSetItemServ::add_rbum(
            &mut RbumSetItemAddReq {
                sort: add_req.sort,
                rel_rbum_set_id: add_req.set_id.clone(),
                rel_rbum_set_cate_id: add_req.set_cate_id.clone(),
                rel_rbum_item_id: add_req.rel_rbum_item_id.clone(),
            },
            funs,
            ctx,
        )
        .await;

        let set_cate_id = add_req.set_cate_id.clone();
        if let Ok(account) = IamAccountServ::get_item(add_req.rel_rbum_item_id.clone().as_str(), &IamAccountFilterReq::default(), funs, ctx).await {
            let _ = IamLogClient::add_ctx_task(
                LogParamTag::IamOrg,
                Some(set_cate_id.clone()),
                format!("添加部门人员{}", account.name.clone()),
                Some("AddAccount".to_string()),
                ctx,
            )
            .await;
            let _ = IamCertServ::package_tardis_account_context_and_resp(&add_req.rel_rbum_item_id.clone(), &ctx.own_paths, "".to_string(), None, funs, &ctx).await;
            let _ = IamSearchClient::async_add_or_modify_account_search(&add_req.rel_rbum_item_id, Box::new(true), "", funs, ctx).await;
            if !set_cate_id.is_empty() {
                IamStatsClient::async_org_fact_record_load(set_cate_id.clone(), funs, ctx).await?;
            }
        }

        result
    }

    pub async fn modify_set_item(set_item_id: &str, modify_req: &mut RbumSetItemModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        RbumSetItemServ::modify_rbum(set_item_id, modify_req, funs, ctx).await
    }

    pub async fn delete_set_item(set_item_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<u64> {
        let item: RbumSetItemDetailResp = RbumSetItemServ::get_rbum(
            set_item_id,
            &RbumSetItemFilterReq {
                basic: Default::default(),
                rel_rbum_item_disabled: Some(false),
                rel_rbum_item_can_not_exist: Some(true),
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?;

        let result = RbumSetItemServ::delete_rbum(set_item_id, funs, ctx).await;

        if result.is_ok() && item.rel_rbum_item_kind_id == funs.iam_basic_kind_account_id() {
            if let Some(cate_id) = item.rel_rbum_set_cate_id.clone() {
                IamStatsClient::async_org_fact_record_load(cate_id, funs, ctx).await?;
            }
            if let Ok(account) = IamAccountServ::get_item(item.rel_rbum_item_id.clone().as_str(), &IamAccountFilterReq::default(), funs, ctx).await {
                let _ = IamLogClient::add_ctx_task(
                    LogParamTag::IamOrg,
                    Some(item.rel_rbum_set_cate_id.unwrap_or_default().clone()),
                    format!("移除部门人员{}", account.name.clone()),
                    Some("RemoveAccount".to_string()),
                    ctx,
                )
                .await;
                let _ = IamCertServ::package_tardis_account_context_and_resp(&item.rel_rbum_item_id.clone(), &ctx.own_paths, "".to_string(), None, funs, &ctx).await;
                let _ = IamSearchClient::async_add_or_modify_account_search(&item.rel_rbum_item_id, Box::new(true), "", funs, ctx).await;
            }
        }

        result
    }

    pub async fn find_set_cate_name(filter_req: &RbumSetCateFilterReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Vec<String>> {
        RbumSetCateServ::find_detail_rbums(filter_req, None, None, funs, ctx).await.map(|r| r.into_iter().map(|r| format!("{},{}", r.id, r.name)).collect())
    }

    pub async fn paginate_set_items(
        set_id: Option<String>,
        set_cate_id: Option<String>,
        item_id: Option<String>,
        scope_level: Option<RbumScopeLevelKind>,
        with_sub: bool,
        table_rbum_set_cate_is_left: Option<bool>,
        page_number: u32,
        page_size: u32,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<TardisPage<RbumSetItemDetailResp>> {
        RbumSetItemServ::paginate_detail_rbums(
            &RbumSetItemFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: with_sub,
                    ..Default::default()
                },
                rel_rbum_item_can_not_exist: table_rbum_set_cate_is_left,
                rel_rbum_item_disabled: Some(false),
                rel_rbum_set_id: set_id.clone(),
                rel_rbum_set_cate_ids: set_cate_id.map(|r| vec![r]),
                rel_rbum_item_ids: item_id.map(|i| vec![i]),
                rel_rbum_item_scope_level: scope_level,
                ..Default::default()
            },
            page_number,
            page_size,
            None,
            None,
            funs,
            ctx,
        )
        .await
    }

    pub async fn find_set_items(
        set_id: Option<String>,
        set_cate_id: Option<String>,
        item_id: Option<String>,
        scope_level: Option<RbumScopeLevelKind>,
        with_sub: bool,
        table_rbum_set_cate_is_left: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<Vec<RbumSetItemDetailResp>> {
        RbumSetItemServ::find_detail_rbums(
            &RbumSetItemFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: with_sub,
                    ..Default::default()
                },
                rel_rbum_item_can_not_exist: table_rbum_set_cate_is_left,
                rel_rbum_item_disabled: Some(false),
                rel_rbum_set_id: set_id.clone(),
                rel_rbum_set_cate_ids: set_cate_id.map(|r| vec![r]),
                rel_rbum_item_ids: item_id.map(|i| vec![i]),
                rel_rbum_item_scope_level: scope_level,
                ..Default::default()
            },
            None,
            None,
            funs,
            ctx,
        )
        .await
    }
    /// 和find_set_items的区别是,对set_cate_id为None时候的处理不同
    pub async fn find_set_items_with_none_set_cate_id(
        set_id: Option<String>,
        set_cate_id: Option<String>,
        item_id: Option<String>,
        with_sub: bool,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<Vec<RbumSetItemDetailResp>> {
        if set_cate_id.is_none() {
            RbumSetItemServ::find_detail_rbums(
                &RbumSetItemFilterReq {
                    basic: RbumBasicFilterReq {
                        with_sub_own_paths: with_sub,
                        ..Default::default()
                    },
                    rel_rbum_item_disabled: Some(false),
                    rel_rbum_set_id: set_id.clone(),
                    rel_rbum_set_item_cate_code: Some("".to_string()),
                    rel_rbum_item_can_not_exist: Some(true),
                    rel_rbum_item_ids: item_id.map(|i| vec![i]),
                    ..Default::default()
                },
                None,
                None,
                funs,
                ctx,
            )
            .await
        } else {
            Self::find_set_items(set_id, set_cate_id, item_id, None, with_sub, None, funs, ctx).await
        }
    }

    pub async fn find_set_paths(set_item_id: &str, set_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Vec<Vec<RbumSetPathResp>>> {
        RbumSetItemServ::find_set_paths(set_item_id, set_id, funs, ctx).await
    }

    pub async fn find_flat_set_items(set_id: &str, item_id: &str, with_sub: bool, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<HashMap<String, String>> {
        let items = Self::find_set_items(Some(set_id.to_string()), None, Some(item_id.to_string()), None, with_sub, None, funs, ctx).await?;
        let items = items
            .into_iter()
            .map(|item| {
                (
                    format!("{}{}{}", item.rel_rbum_set_id, SET_AND_ITEM_SPLIT_FLAG, item.rel_rbum_set_cate_sys_code.unwrap_or_default()),
                    item.rel_rbum_set_cate_name.unwrap_or_default(),
                )
            })
            .collect();
        Ok(items)
    }

    pub async fn check_scope(app_id: &str, account_id: &str, set_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<bool> {
        RbumSetItemServ::check_a_is_parent_or_sibling_of_b(account_id, app_id, set_id, funs, ctx).await
    }

    pub async fn cut_tree_to_new_set<'a>(
        from_tree: &'a RbumSetTreeResp,
        target_set_id: &'a str,
        old_pid: Option<String>,
        target_pid: Option<String>,
        funs: &'a TardisFunsInst,
        from_ctx: &'a TardisContext,
        target_ctx: &'a TardisContext,
    ) -> TardisResult<()> {
        Self::copy_tree_to_new_set(from_tree, target_set_id, old_pid.clone(), target_pid, funs, target_ctx).await?;
        Self::delete_tree(from_tree, old_pid, funs, from_ctx).await
    }

    pub async fn delete_tree<'a>(delete_tree: &'a RbumSetTreeResp, pid: Option<String>, funs: &'a TardisFunsInst, ctx: &'a TardisContext) -> TardisResult<()> {
        let mut stack = vec![];
        stack.push(pid.clone());
        let mut cate_vec = delete_tree.main.to_owned();
        let mut cate_item_vec = if let Some(ext) = &delete_tree.ext { ext.items.to_owned() } else { HashMap::new() };
        while !stack.is_empty() {
            let mut loop_cate_vec = cate_vec.clone();
            let loop_pid = stack.pop().unwrap_or_default();
            loop_cate_vec.retain(|cate| cate.pid == loop_pid);
            //have sub tree?
            let have_next_node = !loop_cate_vec.is_empty();
            if have_next_node && loop_pid.is_some() {
                stack.push(loop_pid.clone());
            }
            for r in loop_cate_vec {
                if let Some(set_items) = cate_item_vec.get(&r.id) {
                    for set_item in set_items {
                        Self::delete_set_item(&set_item.id, funs, ctx).await?;
                    }
                    cate_item_vec.insert(r.id.clone(), vec![]);
                }

                stack.push(Some(r.id.clone()));
            }
            if !have_next_node && loop_pid.is_some() && loop_pid != pid {
                Self::delete_set_cate(&loop_pid.clone().unwrap_or_default(), funs, ctx).await?;
                cate_vec.retain(|c| c.id != loop_pid.clone().unwrap_or_default());
            }
        }

        Ok(())
    }

    pub async fn copy_tree_to_new_set<'a>(
        tree: &'a RbumSetTreeResp,
        target_set_id: &'a str,
        old_pid: Option<String>,
        target_pid: Option<String>,
        funs: &'a TardisFunsInst,
        target_ctx: &'a TardisContext,
    ) -> TardisResult<()> {
        let mut old_stack = vec![];
        let mut target_stack = vec![];
        old_stack.push(old_pid.clone());
        target_stack.push(target_pid);

        let cate_vec = tree.main.to_owned();
        let cate_item_vec = if let Some(ext) = &tree.ext { ext.items.to_owned() } else { HashMap::new() };

        while !old_stack.is_empty() {
            let mut loop_cate_vec = cate_vec.clone();
            let loop_pid = old_stack.pop().unwrap_or_default();
            let loop_target_pid = target_stack.pop().unwrap_or_default();
            loop_cate_vec.retain(|cate| cate.pid == loop_pid);
            for r in loop_cate_vec {
                let new_cate_id = Self::add_set_cate(
                    target_set_id,
                    &IamSetCateAddReq {
                        name: TrimString(r.name.clone()),
                        scope_level: Some(r.scope_level.clone()),
                        bus_code: None,
                        icon: None,
                        sort: None,
                        ext: None,
                        rbum_parent_cate_id: loop_target_pid.clone(),
                    },
                    funs,
                    target_ctx,
                )
                .await?;
                old_stack.push(Some(r.id.clone()));
                target_stack.push(Some(new_cate_id.clone()));
                if let Some(set_items) = cate_item_vec.get(&r.id) {
                    let mut sort = 1;
                    for set_item in set_items {
                        //只有全局账号可以跨租户
                        if set_item.rel_rbum_item_scope_level != RbumScopeLevelKind::Root {
                            continue;
                        }
                        Self::add_set_item(
                            &IamSetItemAddReq {
                                set_id: target_set_id.to_string(),
                                set_cate_id: new_cate_id.clone(),
                                sort,
                                rel_rbum_item_id: set_item.rel_rbum_item_id.clone(),
                            },
                            funs,
                            target_ctx,
                        )
                        .await?;
                        sort += 1;
                    }
                }
            }
        }
        Ok(())
    }

    // Transform RbumSetTree to IamResSetTreeResp
    pub async fn transform_res_tree(original_tree: RbumSetTreeResp, role_ids: Option<Vec<String>>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<IamResSetTreeResp> {
        let res_ids = if let Some(role_ids) = role_ids {
            let global_ctx = TardisContext {
                own_paths: "".to_string(),
                ..ctx.clone()
            };
            Some(
                IamRelServ::find_simple_rels(
                    &RbumRelFilterReq {
                        basic: RbumBasicFilterReq {
                            with_sub_own_paths: true,
                            ..Default::default()
                        },
                        tag: Some(IamRelKind::IamResRole.to_string()),
                        to_rbum_item_ids: Some(role_ids.clone()),
                        ..Default::default()
                    },
                    None,
                    None,
                    false,
                    funs,
                    &global_ctx,
                )
                .await?
                .into_iter()
                .map(|r| r.rel_id)
                .unique()
                .collect_vec(),
            )
        } else {
            None
        };

        let ext = if let Some(value_ext) = original_tree.ext.clone() {
            Some(IamResSetTreeExtResp {
                items: value_ext.items,
                item_number_agg: value_ext.item_number_agg,
                item_kinds: value_ext.item_kinds,
                item_domains: value_ext.item_domains,
                item_data_guards: HashMap::new(),
            })
        } else {
            None
        };
        let mut result = IamResSetTreeResp { main: original_tree.main, ext };
        if let Some(ext) = &mut result.ext {
            let res_set_item_ids = if let Some(ext) = original_tree.ext {
                let ext_values = ext.items.values().collect_vec().into_iter().cloned().collect_vec();
                ext_values.concat().into_iter().map(|rbum_set_item| rbum_set_item.rel_rbum_item_id).collect_vec()
            } else {
                vec![]
            };
            let global_ctx = TardisContext {
                own_paths: "".to_string(),
                ..ctx.clone()
            };
            let data_guard_set_id = IamSetServ::get_default_set_id_by_ctx(&IamSetKind::Res, funs, &global_ctx).await?;
            let data_guard_set_cate_id = RbumSetCateServ::find_one_rbum(
                &RbumSetCateFilterReq {
                    bus_codes: Some(vec![DATA_GUARD_ROOT_SET_BUS_CODE.to_string()]),
                    rel_rbum_set_id: Some(data_guard_set_id.clone()),
                    ..Default::default()
                },
                funs,
                ctx,
            )
            .await?
            .map(|s| s.id)
            .unwrap_or_default();
            let data_guard_set_items = RbumSetItemServ::find_detail_rbums(
                &RbumSetItemFilterReq {
                    rel_rbum_set_cate_ids: Some(vec![data_guard_set_cate_id]),
                    rel_rbum_item_ids: res_ids.map(|res_ids| res_ids.into_iter().collect()),
                    rel_rbum_set_id: Some(data_guard_set_id.clone()),
                    ..Default::default()
                },
                None,
                None,
                funs,
                &global_ctx,
            )
            .await?;
            let rel_data_guard = IamRelServ::find_rels(
                &RbumRelFilterReq {
                    basic: RbumBasicFilterReq {
                        with_sub_own_paths: true,
                        ..Default::default()
                    },
                    tag: Some(IamRelKind::IamResDataGuard.to_string()),
                    to_rbum_item_ids: Some(res_set_item_ids.clone()),
                    ..Default::default()
                },
                None,
                None,
                funs,
                &global_ctx,
            )
            .await?;
            let mut data_guard_map = HashMap::new();
            for res_set_item_id in res_set_item_ids {
                let rel_ids = rel_data_guard.iter().filter(|r| r.rel.to_rbum_item_id == res_set_item_id).map(|r| r.rel.from_rbum_id.clone()).collect_vec();
                let data_guard = data_guard_set_items
                    .iter()
                    .filter(|i| rel_ids.contains(&i.rel_rbum_item_id))
                    .map(|i| RbumSetItemRelInfoResp {
                        id: i.id.to_string(),
                        sort: i.sort,
                        rel_rbum_item_id: i.rel_rbum_item_id.to_string(),
                        rel_rbum_item_code: i.rel_rbum_item_code.to_string(),
                        rel_rbum_item_name: i.rel_rbum_item_name.to_string(),
                        rel_rbum_item_kind_id: i.rel_rbum_item_kind_id.to_string(),
                        rel_rbum_item_domain_id: i.rel_rbum_item_domain_id.to_string(),
                        rel_rbum_item_owner: i.rel_rbum_item_owner.to_string(),
                        rel_rbum_item_create_time: i.rel_rbum_item_create_time,
                        rel_rbum_item_update_time: i.rel_rbum_item_update_time,
                        rel_rbum_item_disabled: i.rel_rbum_item_disabled,
                        rel_rbum_item_scope_level: i.rel_rbum_item_scope_level.clone(),
                        own_paths: i.own_paths.to_string(),
                        owner: i.owner.to_string(),
                    })
                    .collect_vec();

                data_guard_map.insert(res_set_item_id.clone(), data_guard);
            }
            ext.item_data_guards = data_guard_map;
        }
        Ok(result)
    }
}
