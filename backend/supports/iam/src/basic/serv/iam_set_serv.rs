use std::collections::{HashMap, HashSet};

use bios_basic::rbum::dto::rbum_filer_dto::{RbumBasicFilterReq, RbumRelFilterReq, RbumSetCateFilterReq, RbumSetFilterReq, RbumSetItemFilterReq, RbumSetTreeFilterReq};
use bios_basic::rbum::dto::rbum_rel_agg_dto::RbumRelAggAddReq;
use bios_basic::rbum::dto::rbum_rel_dto::RbumRelAddReq;

use bios_basic::rbum::dto::rbum_set_cate_dto::{RbumSetCateAddReq, RbumSetCateModifyReq, RbumSetCateSummaryResp};
use bios_basic::rbum::dto::rbum_set_dto::{RbumSetAddReq, RbumSetPathResp, RbumSetSummaryResp, RbumSetTreeExtResp, RbumSetTreeNodeResp, RbumSetTreeResp};
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
use tardis::log::warn;

use tardis::chrono::Utc;
use tardis::serde_json::json;
use tardis::web::web_resp::TardisPage;
use tardis::{TardisFuns, TardisFunsInst};

use crate::basic::dto::iam_account_dto::IamAccountDetailResp;
use crate::basic::dto::iam_filer_dto::{IamAccountFilterReq, IamResFilterReq, IamRoleFilterReq};
use crate::basic::dto::iam_set_dto::{IamAppsSetAccountRoleReq, IamAppsSetAccountRoleResp, IamResSetTreeExtResp, IamResSetTreeResp, IamSetCateAddReq, IamSetCateModifyReq, IamSetItemAddReq};
use crate::basic::dto::iam_tenant_dto::IamTenantSummaryResp;
use crate::iam_config::{IamBasicConfigApi, IamConfig};
use crate::iam_constants::{RBUM_ITEM_NAME_APP_READ_ROLE, RBUM_SCOPE_LEVEL_APP, RBUM_SCOPE_LEVEL_TENANT};
use crate::iam_enumeration::{IamPermKind, IamRelKind, IamResKind, IamResShowScope, IamRoleKind, IamSetCateKind, IamSetKind};

use super::clients::iam_kv_client::IamKvClient;
use super::clients::iam_log_client::{IamLogClient, LogParamTag};
use super::clients::iam_search_client::IamSearchClient;
use super::clients::iam_stats_client::IamStatsClient;
use super::iam_account_serv::IamAccountServ;
use super::iam_cert_serv::IamCertServ;
use super::iam_rel_serv::IamRelServ;
use super::iam_res_serv::IamResServ;
use super::iam_role_serv::IamRoleServ;
use super::iam_sub_deploy_serv::IamSubDeployServ;

const SET_AND_ITEM_SPLIT_FLAG: &str = ":";

pub const MENU_ROOT_SET_BUS_CODE: &str = "__menus__";
pub const API_ROOT_SET_BUS_CODE: &str = "__apis__";
pub const DATA_GUARD_ROOT_SET_BUS_CODE: &str = "__data_guards__";

pub struct IamSetServ;

/// 账号在各 Apps set 上配置的上下文角色。set id -> (role id, role code)。
pub struct IamAppsSetAccountRoleMap {
    by_set_id: HashMap<String, (String, String)>,
    platform_set_id: Option<String>,
}

impl IamAppsSetAccountRoleMap {
    /// 租户 set 有配置时用租户配置，否则沿用平台根 set 的配置，两边都没有时回退 `app_read`。
    pub fn resolve(&self, set_id: &str, app_read_role_id: &str) -> HashMap<String, String> {
        let mut roles = HashMap::new();
        if let Some((role_id, role_code)) = self.by_set_id.get(set_id).or_else(|| self.platform_set_id.as_ref().and_then(|platform_set_id| self.by_set_id.get(platform_set_id))) {
            roles.insert(role_id.clone(), role_code.clone());
        } else {
            roles.insert(app_read_role_id.to_string(), RBUM_ITEM_NAME_APP_READ_ROLE.to_string());
        }
        roles
    }
}

impl IamSetServ {
    /// 根据 account_id 和 set_kind 获取对应的 set 列表
    ///
    /// Get set list by account_id and set_kind
    pub async fn find_sets_by_account_id_and_kind(account_id: &str, set_kind: &IamSetKind, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Vec<RbumSetSummaryResp>> {
        // 调用 RbumSetItemServ::find_detail_rbums 获取 account 关联的 set item
        let set_items = RbumSetItemServ::find_detail_rbums(
            &RbumSetItemFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                rel_rbum_item_can_not_exist: Some(true),
                rel_rbum_item_disabled: Some(false),
                rel_rbum_item_ids: Some(vec![account_id.to_string()]),
                ..Default::default()
            },
            None,
            None,
            funs,
            ctx,
        )
        .await?;

        if set_items.is_empty() {
            return Ok(vec![]);
        }

        // 去重并收集 rel_rbum_set_id
        let set_ids: Vec<String> = set_items.iter().map(|item| item.rel_rbum_set_id.clone()).collect::<HashSet<String>>().into_iter().collect();

        // 根据 set_ids 查询 set 信息，并过滤出指定 kind 的 set
        let sets = RbumSetServ::find_rbums(
            &RbumSetFilterReq {
                basic: RbumBasicFilterReq {
                    ids: Some(set_ids),
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                rel: None,
                kind: Some(set_kind.to_string()),
            },
            None,
            None,
            funs,
            ctx,
        )
        .await?;

        // 存在 own_paths 为空的 set，表示账号绑定了平台层根节点，等价于关联所有 set
        if sets.iter().any(|set| set.own_paths.is_empty()) {
            let sys_ctx = TardisContext { own_paths: "".to_string(), ..ctx.clone() };
            return RbumSetServ::find_rbums(
                &RbumSetFilterReq {
                    basic: RbumBasicFilterReq {
                        with_sub_own_paths: true,
                        ..Default::default()
                    },
                    rel: None,
                    kind: Some(set_kind.to_string()),
                },
                None,
                None,
                funs,
                &sys_ctx,
            )
            .await;
        }

        Ok(sets)
    }

    /// Whether the account is bound to the platform Apps root (grants full apps tree across all tenants).
    ///
    /// 账号是否挂载在平台层 Apps 根节点（等价于拥有全部租户应用树权限）。
    pub async fn account_has_platform_apps_auth(account_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<bool> {
        let sets = Self::find_sets_by_account_id_and_kind(account_id, &IamSetKind::Apps, funs, ctx).await?;
        Ok(sets.iter().any(|set| set.own_paths.is_empty()))
    }

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

    /// Ensure platform-level Apps set exists (for root item binding on platform tree).
    ///
    /// 确保平台层 Apps set 存在（用于平台树根节点挂载资源项）。
    pub async fn ensure_platform_apps_set(funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<String> {
        let code = Self::get_default_code(&IamSetKind::Apps, "");
        if let Some(set_id) = RbumSetServ::get_rbum_set_id_by_code(&code, true, funs, ctx).await? {
            return Ok(set_id);
        }
        let (set_id, _) = Self::init_set(IamSetKind::Apps, RbumScopeLevelKind::Root, funs, ctx).await?;
        Ok(set_id)
    }

    fn platform_apps_tree_root_id(funs: &TardisFunsInst) -> String {
        funs.conf::<IamConfig>().platform_apps_tree_root_id.clone()
    }

    fn platform_apps_tree_root_name(funs: &TardisFunsInst) -> String {
        funs.conf::<IamConfig>().platform_apps_tree_root_name.clone()
    }

    pub fn is_platform_apps_root_cate_id(cate_id: Option<&str>, funs: &TardisFunsInst) -> bool {
        let root_id = Self::platform_apps_tree_root_id(funs);
        match cate_id {
            None | Some("") => true,
            Some(s) => s == root_id.as_str(),
        }
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
                let querying_this_rel_cate = filter.sys_codes.as_ref().is_some_and(|codes| {
                    codes.iter().any(|code| !code.is_empty() && code == &r.sys_code)
                });
                let mut set_filter = filter.clone();
                if set_filter.sys_codes.is_some() {
                    set_filter.sys_codes = Some(vec!["".to_string()]);
                }
                // 懒加载平台层级时跳过跨租户展开；用户显式选中跨租户节点时展开关联租户的一级组织
                let skip_rel_tenant_expand = filter.sys_codes.is_some()
                    && filter.sys_code_query_depth == Some(1)
                    && filter.sys_code_query_kind == Some(RbumSetCateLevelQueryKind::CurrentAndSub)
                    && !querying_this_rel_cate;
                if skip_rel_tenant_expand {
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

    /// Assemble platform-level apps tree: platform virtual root -> tenant virtual nodes -> tenant app cates.
    ///
    /// 组装平台层应用树：平台虚拟根 -> 租户虚拟节点 -> 各租户产品组节点。
    ///
    /// * ``parent_sys_code`` is None: full merged tree
    /// * ``parent_sys_code`` is ``""`` or configured [`IamConfig::platform_apps_tree_root_id`]: platform root + tenant nodes only
    /// * ``parent_sys_code`` is a tenant id: first-level app cates under that tenant
    /// * ``parent_sys_code`` is a cate sys_code and ``tenant_id`` is set: next level under that cate in the tenant
    pub async fn get_platform_apps_tree(
        tenants: &[IamTenantSummaryResp],
        parent_sys_code: Option<String>,
        tenant_id: Option<String>,
        only_related: bool,
        funs: &TardisFunsInst,
        sys_ctx: &TardisContext,
    ) -> TardisResult<RbumSetTreeResp> {
        let root_id = Self::platform_apps_tree_root_id(funs);
        let root_name = Self::platform_apps_tree_root_name(funs);
        // Platform root binding implies full tree; align with get_account_apps_from_all_sets.
        let only_related = only_related && !Self::account_has_platform_apps_auth(&sys_ctx.owner, funs, sys_ctx).await?;
        let tenant_id_set: HashSet<String> = tenants.iter().map(|t| t.id.clone()).collect();

        if let Some(parent) = parent_sys_code.as_ref().filter(|p| !p.is_empty() && *p != &root_id) {
            if tenant_id_set.contains(parent) {
                let tenant = tenants.iter().find(|t| t.id == *parent).expect("ignore");
                return Self::get_platform_tenant_apps_subtree(tenant, Some("".to_string()), only_related, funs, sys_ctx).await;
            }
            if let Some(tid) = tenant_id.as_ref() {
                let tenant = tenants.iter().find(|t| t.id == *tid).ok_or_else(|| {
                    funs.err().not_found("iam_tenant", "get_platform_apps_tree", &format!("tenant {tid} not found"), "404-iam-tenant-not-exist")
                })?;
                return Self::get_platform_tenant_apps_subtree(tenant, Some(parent.clone()), only_related, funs, sys_ctx).await;
            }
            return Err(funs.err().bad_request(
                "iam_apps_tree",
                "get_platform_apps_tree",
                &format!("platform tree node {parent} requires tenant_id when expanding app cates"),
                "400-iam-apps-tree-tenant-id-required",
            ));
        }

        if parent_sys_code.as_ref().is_some_and(|p| p.is_empty() || p == &root_id) {
            let mut result = Self::build_platform_tenant_nodes_tree(tenants, sys_ctx, &root_id, &root_name);
            Self::attach_platform_root_ext(&mut result, Self::fetch_platform_apps_root_ext(funs, sys_ctx).await?, &root_id);
            return Ok(result);
        }

        let mut result = RbumSetTreeResp {
            main: vec![Self::build_platform_root_node(sys_ctx, &root_id, &root_name)],
            ext: None,
        };
        for tenant in tenants {
            if let Some((set_id, tenant_tree)) = Self::fetch_tenant_apps_tree(tenant, None, only_related, funs, sys_ctx).await? {
                Self::merge_tenant_apps_tree(&mut result, tenant, &set_id, tenant_tree, true, &root_id);
            }
        }
        Self::attach_platform_root_ext(&mut result, Self::fetch_platform_apps_root_ext(funs, sys_ctx).await?, &root_id);
        Ok(result)
    }

    async fn fetch_platform_apps_root_ext(funs: &TardisFunsInst, sys_ctx: &TardisContext) -> TardisResult<Option<RbumSetTreeExtResp>> {
        let set_id = Self::ensure_platform_apps_set(funs, sys_ctx).await?;
        let tree = Self::get_tree(
            &set_id,
            &mut RbumSetTreeFilterReq {
                fetch_cate_item: true,
                hide_item_with_disabled: true,
                ..Default::default()
            },
            funs,
            sys_ctx,
        )
        .await?;
        Ok(tree.ext)
    }

    fn attach_platform_root_ext(result: &mut RbumSetTreeResp, platform_ext: Option<RbumSetTreeExtResp>, root_id: &str) {
        let Some(mut src) = platform_ext else { return };
        let ext = result.ext.get_or_insert_with(|| RbumSetTreeExtResp {
            items: HashMap::new(),
            item_number_agg: HashMap::new(),
            item_kinds: HashMap::new(),
            item_domains: HashMap::new(),
        });
        if let Some(items) = src.items.remove("") {
            ext.items.insert(root_id.to_string(), items);
        }
        if let Some(agg) = src.item_number_agg.remove("") {
            ext.item_number_agg.insert(root_id.to_string(), agg);
        }
        ext.items.extend(src.items);
        ext.item_number_agg.extend(src.item_number_agg);
        ext.item_kinds.extend(src.item_kinds);
        ext.item_domains.extend(src.item_domains);
    }

    fn build_platform_root_node(ctx: &TardisContext, root_id: &str, root_name: &str) -> RbumSetTreeNodeResp {
        let now = Utc::now();
        RbumSetTreeNodeResp {
            id: root_id.to_string(),
            sys_code: String::new(),
            bus_code: String::new(),
            name: root_name.to_string(),
            icon: String::new(),
            sort: 0,
            ext: json!({"virtual":true,"scope":"platform"}).to_string(),
            pid: None,
            rel: None,
            own_paths: String::new(),
            owner: ctx.owner.clone(),
            create_time: now,
            update_time: now,
            scope_level: RbumScopeLevelKind::Root,
        }
    }

    fn build_tenant_virtual_node(tenant: &IamTenantSummaryResp, set_id: &str, root_id: &str) -> RbumSetTreeNodeResp {
        RbumSetTreeNodeResp {
            id: tenant.id.clone(),
            sys_code: tenant.id.clone(),
            bus_code: String::new(),
            name: tenant.name.clone(),
            icon: tenant.icon.clone(),
            sort: tenant.sort,
            ext: json!({"virtual":true,"scope":"tenant","tenant_id":tenant.id,"set_id":set_id}).to_string(),
            pid: Some(root_id.to_string()),
            rel: None,
            own_paths: tenant.own_paths.clone(),
            owner: tenant.owner.clone(),
            create_time: tenant.create_time,
            update_time: tenant.update_time,
            scope_level: tenant.scope_level.clone(),
        }
    }

    fn build_platform_tenant_nodes_tree(tenants: &[IamTenantSummaryResp], ctx: &TardisContext, root_id: &str, root_name: &str) -> RbumSetTreeResp {
        RbumSetTreeResp {
            main: std::iter::once(Self::build_platform_root_node(ctx, root_id, root_name))
                .chain(tenants.iter().map(|tenant| {
                    RbumSetTreeNodeResp {
                        ext: json!({"virtual":true,"scope":"tenant","tenant_id":tenant.id}).to_string(),
                        ..Self::build_tenant_virtual_node(tenant, "", root_id)
                    }
                }))
                .collect(),
            ext: None,
        }
    }

    async fn get_platform_tenant_apps_subtree(
        tenant: &IamTenantSummaryResp,
        parent_sys_code: Option<String>,
        only_related: bool,
        funs: &TardisFunsInst,
        sys_ctx: &TardisContext,
    ) -> TardisResult<RbumSetTreeResp> {
        let include_tenant_node = parent_sys_code.as_ref().is_some_and(|c| c.is_empty());
        let Some((set_id, tenant_tree)) = Self::fetch_tenant_apps_tree(tenant, parent_sys_code, only_related, funs, sys_ctx).await? else {
            return Ok(RbumSetTreeResp { main: vec![], ext: None });
        };
        let mut result = RbumSetTreeResp { main: vec![], ext: None };
        Self::merge_tenant_apps_tree(&mut result, tenant, &set_id, tenant_tree, include_tenant_node, &Self::platform_apps_tree_root_id(funs));
        Ok(result)
    }

    async fn fetch_tenant_apps_tree(
        tenant: &IamTenantSummaryResp,
        parent_sys_code: Option<String>,
        only_related: bool,
        funs: &TardisFunsInst,
        sys_ctx: &TardisContext,
    ) -> TardisResult<Option<(String, RbumSetTreeResp)>> {
        let t_ctx = IamCertServ::use_tenant_ctx(sys_ctx.clone(), &tenant.id)?;
        let set_id = Self::get_default_set_id_by_ctx(&IamSetKind::Apps, funs, &t_ctx).await?;
        let mut tree_filter = RbumSetTreeFilterReq {
            fetch_cate_item: true,
            hide_item_with_disabled: true,
            ..Default::default()
        };
        if let Some(code) = parent_sys_code {
            tree_filter.sys_codes = Some(vec![code]);
            tree_filter.sys_code_query_kind = Some(RbumSetCateLevelQueryKind::Sub);
            tree_filter.sys_code_query_depth = Some(1);
        }
        let tree = if only_related {
            Self::get_apps_tree_with_auth_by_account_opt(&set_id, &t_ctx.owner, funs, &t_ctx).await?
        } else {
            Some(Self::get_tree(&set_id, &mut tree_filter, funs, &t_ctx).await?)
        };
        Ok(tree.map(|tree| (set_id, tree)))
    }

    fn merge_tenant_apps_tree(result: &mut RbumSetTreeResp, tenant: &IamTenantSummaryResp, set_id: &str, tenant_tree: RbumSetTreeResp, include_tenant_node: bool, root_id: &str) {
        if include_tenant_node {
            result.main.push(Self::build_tenant_virtual_node(tenant, set_id, root_id));
        }
        for mut node in tenant_tree.main {
            if node.pid.is_none() {
                node.pid = Some(tenant.id.clone());
            }
            if node.ext.is_empty() {
                node.ext = json!({"tenant_id": tenant.id}).to_string();
            }
            result.main.push(node);
        }
        Self::merge_tree_ext(&mut result.ext, &tenant.id, tenant_tree.ext);
    }

    fn merge_tree_ext(merged: &mut Option<RbumSetTreeExtResp>, tenant_id: &str, tree_ext: Option<RbumSetTreeExtResp>) {
        let Some(src) = tree_ext else { return };
        let ext = merged.get_or_insert_with(|| RbumSetTreeExtResp {
            items: HashMap::new(),
            item_number_agg: HashMap::new(),
            item_kinds: HashMap::new(),
            item_domains: HashMap::new(),
        });
        for (k, v) in src.items {
            let key = if k.is_empty() { tenant_id.to_string() } else { k };
            ext.items.entry(key).or_default().extend(v);
        }
        for (k, v) in src.item_number_agg {
            let key = if k.is_empty() { tenant_id.to_string() } else { k };
            ext.item_number_agg.insert(key, v);
        }
        ext.item_kinds.extend(src.item_kinds);
        ext.item_domains.extend(src.item_domains);
    }

    /// 按账号权限过滤产品组树：仅返回账号所属节点及其关联应用所在节点（含父级补全）。
    ///
    /// 账号挂载在平台 Apps set（根产品组）时，等价于拥有该 set 下的全部产品组树，直接返回完整树。
    pub async fn get_apps_tree_with_auth_by_account(set_id: &str, account_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<RbumSetTreeResp> {
        // 挂在平台根产品组时，根节点等价于拥有该 set 下的全部产品组（含全部应用）
        if Self::account_bound_on_platform_apps_set(account_id, funs, ctx).await? {
            return Self::get_tree(
                set_id,
                &mut RbumSetTreeFilterReq {
                    fetch_cate_item: true,
                    hide_item_with_disabled: true,
                    ..Default::default()
                },
                funs,
                ctx,
            )
            .await;
        }
        let account_ids = vec![account_id.to_string()];
        let app_ids = IamRelServ::find_from_id_rels(&IamRelKind::IamAccountApp, true, account_id, None, None, funs, ctx).await?;

        let account_cate = RbumSetItemServ::find_detail_rbums(
            &RbumSetItemFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                rel_rbum_item_can_not_exist: Some(true),
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
                rel_rbum_item_can_not_exist: Some(true),
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
        let app_cate_sys_codes = app_cate.iter().map(|r| r.rel_rbum_set_cate_sys_code.clone().unwrap_or_default()).collect::<Vec<String>>();
        let account_cate_sys_codes = account_cate.iter().map(|r| r.rel_rbum_set_cate_sys_code.clone().unwrap_or_default()).collect::<Vec<String>>();
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

    /// 按账号权限过滤组织树：根据上下文 groups 对应的组织节点，补全父节点与子节点。
    /// 若 ``filter.sys_codes`` 有值，则按该条件做逐级查询，并与有权限节点取交集。
    pub async fn get_org_tree_with_auth_by_account(
        set_id: &str,
        _account_id: &str,
        filter: &mut RbumSetTreeFilterReq,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<RbumSetTreeResp> {
        let org_cate_sys_codes = ctx
            .groups
            .iter()
            .filter_map(|group| {
                group.split_once(SET_AND_ITEM_SPLIT_FLAG).and_then(|(group_set_id, sys_code)| {
                    if group_set_id == set_id && !sys_code.is_empty() {
                        Some(sys_code.to_string())
                    } else {
                        None
                    }
                })
            })
            .collect::<Vec<String>>();
        if org_cate_sys_codes.is_empty() {
            return Ok(RbumSetTreeResp { main: vec![], ext: None });
        }

        // 补全子节点
        let org_tree_sub = Self::get_tree(
            set_id,
            &mut RbumSetTreeFilterReq {
                fetch_cate_item: true,
                sys_codes: Some(org_cate_sys_codes.clone()),
                sys_code_query_kind: Some(RbumSetCateLevelQueryKind::CurrentAndSub),
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?;

        let mut all_sys_codes = org_tree_sub.main.iter().filter(|cate| !cate.sys_code.is_empty()).map(|cate| cate.sys_code.clone()).collect::<Vec<String>>();
        // 仅对 groups 对应节点补全父节点
        for cate_sys_code in org_cate_sys_codes.iter() {
            if cate_sys_code.is_empty() {
                continue;
            }
            let parent_sys_codes = RbumSetCateServ::get_parent_sys_codes(cate_sys_code, funs)?;
            if !parent_sys_codes.is_empty() {
                all_sys_codes.extend(parent_sys_codes);
            }
        }
        if all_sys_codes.is_empty() {
            return Ok(RbumSetTreeResp { main: vec![], ext: None });
        }
        let all_sys_codes = all_sys_codes.into_iter().collect::<HashSet<String>>();
        let has_parent_sys_code = filter.sys_codes.as_ref().is_some_and(|codes| !codes.is_empty());

        let mut tree = if has_parent_sys_code {
            Self::get_tree(set_id, filter, funs, ctx).await?
        } else {
            Self::get_tree(
                set_id,
                &mut RbumSetTreeFilterReq {
                    fetch_cate_item: filter.fetch_cate_item,
                    hide_item_with_disabled: filter.hide_item_with_disabled,
                    sys_codes: Some(all_sys_codes.iter().cloned().collect()),
                    sys_code_query_kind: Some(RbumSetCateLevelQueryKind::Current),
                    ..Default::default()
                },
                funs,
                ctx,
            )
            .await?
        };
        if has_parent_sys_code {
            tree.main.retain(|node| all_sys_codes.contains(&node.sys_code));
            if let Some(ext) = tree.ext.as_mut() {
                let remain_ids = tree.main.iter().map(|node| node.id.clone()).collect::<HashSet<String>>();
                ext.items.retain(|id, _| remain_ids.contains(id));
                ext.item_number_agg.retain(|id, _| remain_ids.contains(id));
            }
        }
        Ok(tree)
    }

    /// 按账号权限过滤产品组树；无相关节点时返回 `None`。
    ///
    /// 账号挂载在平台 Apps set（根产品组）时，等价于拥有该 set 下的全部产品组树，直接返回完整树。
    pub async fn get_apps_tree_with_auth_by_account_opt(set_id: &str, account_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Option<RbumSetTreeResp>> {
        // 挂在平台根产品组时，根节点等价于拥有该 set 下的全部产品组（含全部应用）
        if Self::account_bound_on_platform_apps_set(account_id, funs, ctx).await? {
            let tree = Self::get_tree(
                set_id,
                &mut RbumSetTreeFilterReq {
                    fetch_cate_item: true,
                    hide_item_with_disabled: true,
                    ..Default::default()
                },
                funs,
                ctx,
            )
            .await?;
            return Ok(Some(tree));
        }
        let account_ids = vec![account_id.to_string()];
        let app_ids = IamRelServ::find_from_id_rels(&IamRelKind::IamAccountApp, true, account_id, None, None, funs, ctx).await?;

        let account_cate = RbumSetItemServ::find_detail_rbums(
            &RbumSetItemFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                rel_rbum_item_can_not_exist: Some(true),
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
                rel_rbum_item_can_not_exist: Some(true),
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
        if account_cate.is_empty() && app_cate.is_empty() {
            return Ok(None);
        }
        let app_cate_sys_codes = app_cate.iter().map(|r| r.rel_rbum_set_cate_sys_code.clone().unwrap_or_default()).collect::<Vec<String>>();
        let account_cate_sys_codes = account_cate.iter().map(|r| r.rel_rbum_set_cate_sys_code.clone().unwrap_or_default()).collect::<Vec<String>>();
        if account_cate_sys_codes.is_empty() && app_cate_sys_codes.is_empty() {
            return Ok(Some(RbumSetTreeResp { main: vec![], ext: None }));
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
            return Ok(None);
        }
        let all_sys_codes = all_sys_codes.iter().map(|r| r.to_string()).collect::<Vec<String>>().into_iter().collect::<HashSet<String>>().into_iter().collect::<Vec<String>>();

        let result = Self::get_tree(
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
        .await?;
        Ok(Some(result))
    }

    /// 账号是否挂在平台层 Apps set（根产品组）。
    ///
    /// set item 没有 scope_level，租户上下文会按 `own_paths LIKE '{tenant}%'` 过滤，看不到 `own_paths` 为空的根节点挂载。
    async fn account_bound_on_platform_apps_set(account_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<bool> {
        let sys_ctx = TardisContext {
            own_paths: String::new(),
            ..ctx.clone()
        };
        let code = Self::get_default_code(&IamSetKind::Apps, "");
        if let Some(platform_set_id) = RbumSetServ::get_rbum_set_id_by_code(&code, true, funs, &sys_ctx).await? {
            let items = Self::find_set_items(Some(platform_set_id), None, Some(account_id.to_string()), None, true, Some(true), funs, &sys_ctx).await?;
            Ok(!items.is_empty())
        } else {
            Ok(false)
        }
    }

    pub async fn get_app_with_auth_by_account(set_id: &str, account_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Vec<(String, String)>> {
        // 挂在平台根产品组时，登录传入的是租户 set。根节点等价于拥有该 set 下全部应用。
        if Self::account_bound_on_platform_apps_set(account_id, funs, ctx).await? {
            return Self::get_all_apps_in_set(set_id, funs, ctx).await;
        }
        // 获取 account_id 对应的 set_cate
        let rbum_set_cate_code = RbumSetItemServ::find_detail_rbums(
            &RbumSetItemFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                rel_rbum_item_can_not_exist: Some(true),
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
        // .filter(|r| r.rel_rbum_set_cate_sys_code.is_some())
        .map(|r| r.rel_rbum_set_cate_sys_code.clone().unwrap_or("".to_string()))
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
                rel_rbum_item_can_not_exist: Some(true),
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

    /// 获取 Set 下的全部应用（不做账号分类权限过滤）
    pub async fn get_all_apps_in_set(set_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Vec<(String, String)>> {
        Ok(RbumSetItemServ::find_detail_rbums(
            &RbumSetItemFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                rel_rbum_item_can_not_exist: Some(true),
                rel_rbum_item_disabled: Some(false),
                rel_rbum_set_id: Some(set_id.to_string()),
                rel_rbum_item_kind_ids: Some(vec![funs.iam_basic_kind_app_id()]),
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
        .collect())
    }

    pub async fn load_apps_set_account_roles(account_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<IamAppsSetAccountRoleMap> {
        let sys_ctx = IamCertServ::use_sys_ctx_unsafe(ctx.clone())?;
        let platform_set_id = RbumSetServ::get_rbum_set_id_by_code(&Self::get_default_code(&IamSetKind::Apps, ""), true, funs, &sys_ctx).await?;
        let rels = RbumRelServ::find_rbums(
            &RbumRelFilterReq {
                basic: RbumBasicFilterReq {
                    own_paths: Some("".to_string()),
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                tag: Some(IamRelKind::IamAccountAppsSetRole.to_string()),
                from_rbum_kind: Some(RbumRelFromKind::Item),
                from_rbum_id: Some(account_id.to_string()),
                disabled: Some(false),
                ..Default::default()
            },
            None,
            Some(true),
            funs,
            &sys_ctx,
        )
        .await?;
        let role_ids = rels.iter().map(|rel| rel.to_rbum_item_id.clone()).collect::<Vec<_>>();
        let role_codes = if role_ids.is_empty() {
            HashMap::new()
        } else {
            IamRoleServ::find_items(
                &IamRoleFilterReq {
                    basic: RbumBasicFilterReq {
                        ids: Some(role_ids),
                        ignore_scope: true,
                        own_paths: Some("".to_string()),
                        with_sub_own_paths: true,
                        enabled: Some(true),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                None,
                None,
                funs,
                &sys_ctx,
            )
            .await?
            .into_iter()
            .map(|role| (role.id, role.code))
            .collect::<HashMap<_, _>>()
        };
        let mut by_set_id = HashMap::new();
        for rel in rels {
            if rel.ext.is_empty() {
                continue;
            }
            if let Some(role_code) = role_codes.get(&rel.to_rbum_item_id) {
                by_set_id.entry(rel.ext.clone()).or_insert_with(|| (rel.to_rbum_item_id.clone(), role_code.clone()));
            }
        }
        Ok(IamAppsSetAccountRoleMap { by_set_id, platform_set_id })
    }

    /// 查询产品所属产品组上，指定角色通过 `IamAccountAppsSetRole` 关联的账号。
    ///
    /// 先取产品挂载的产品组 set。未挂载或 set id 为空时，回退到平台根产品组。
    /// 关联的 `ext` 为 set id，`to` 为角色，`from` 为账号。多个角色共用一次当前产品组查询和一次平台根产品组查询。
    ///
    /// 组装规则：
    /// 1. 优先返回当前产品组 set 下这些角色关联的账号；
    /// 2. 再取平台根产品组 set 下这些角色关联的账号，剔除其中在当前产品组 set 下已绑定任意角色的账号；
    /// 3. 两部分合并去重后返回。
    ///
    /// 产品挂在租户下时，只保留平台账号（关系 `own_paths` 为空）以及该租户及其下级的账号。
    /// 关系写入时 `own_paths` 取账号的 `own_paths`。
    pub async fn find_apps_set_role_account_ids(
        product_id: &str,
        product_own_paths: &str,
        role_ids: &[String],
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<Vec<String>> {
        if role_ids.is_empty() {
            return Ok(vec![]);
        }
        let sys_ctx = IamCertServ::use_sys_ctx_unsafe(ctx.clone())?;
        let Some(set_id) = Self::resolve_product_apps_set_id(product_id, product_own_paths, funs, &sys_ctx).await? else {
            return Ok(vec![]);
        };
        let role_id_set = role_ids.iter().map(String::as_str).collect::<HashSet<_>>();
        // 当前产品组 set 下所有角色（不限本次角色）的关联关系，用于识别当前角色，以及排除已被当前产品组覆盖的平台层账号
        let current_set_rels = RbumRelServ::find_rbums(
            &RbumRelFilterReq {
                basic: RbumBasicFilterReq {
                    own_paths: Some("".to_string()),
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                tag: Some(IamRelKind::IamAccountAppsSetRole.to_string()),
                from_rbum_kind: Some(RbumRelFromKind::Item),
                ext_eq: Some(set_id.clone()),
                disabled: Some(false),
                ..Default::default()
            },
            None,
            None,
            funs,
            &sys_ctx,
        )
        .await?;
        let mut result: HashSet<String> = current_set_rels
            .iter()
            .filter(|rel| role_id_set.contains(rel.to_rbum_item_id.as_str()))
            .filter(|rel| Self::account_own_paths_visible_to_product(&rel.own_paths, product_own_paths))
            .map(|rel| rel.from_rbum_id.clone())
            .collect();
        // 当前产品组与平台根产品组为同一个 set 时，无需再合并平台层关联
        let platform_set_id = RbumSetServ::get_rbum_set_id_by_code(&Self::get_default_code(&IamSetKind::Apps, ""), true, funs, &sys_ctx).await?;
        let Some(platform_set_id) = platform_set_id.filter(|platform_set_id| platform_set_id != &set_id) else {
            return Ok(result.into_iter().collect());
        };
        let platform_set_rels = RbumRelServ::find_rbums(
            &RbumRelFilterReq {
                basic: RbumBasicFilterReq {
                    own_paths: Some("".to_string()),
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                tag: Some(IamRelKind::IamAccountAppsSetRole.to_string()),
                from_rbum_kind: Some(RbumRelFromKind::Item),
                to_rbum_item_ids: Some(role_ids.to_vec()),
                ext_eq: Some(platform_set_id),
                disabled: Some(false),
                ..Default::default()
            },
            None,
            None,
            funs,
            &sys_ctx,
        )
        .await?;
        let current_set_account_ids = current_set_rels.iter().map(|rel| rel.from_rbum_id.clone()).collect::<HashSet<_>>();
        result.extend(
            platform_set_rels
                .into_iter()
                .filter(|rel| Self::account_own_paths_visible_to_product(&rel.own_paths, product_own_paths))
                .map(|rel| rel.from_rbum_id)
                .filter(|account_id| !current_set_account_ids.contains(account_id)),
        );
        Ok(result.into_iter().collect())
    }

    /// 产品租户可见的账号路径。
    ///
    /// 产品在平台层时不收口。产品在租户下时，保留平台账号，以及该租户路径本身和它的下级路径。
    pub fn account_own_paths_visible_to_product(account_own_paths: &str, product_own_paths: &str) -> bool {
        let tenant_own_paths = rbum_scope_helper::get_path_item(RBUM_SCOPE_LEVEL_TENANT.to_int(), product_own_paths).unwrap_or_default();
        if tenant_own_paths.is_empty() {
            return true;
        }
        account_own_paths.is_empty() || account_own_paths == tenant_own_paths || account_own_paths.starts_with(&format!("{tenant_own_paths}/"))
    }

    /// 按产品租户范围补账号详情。候选 id 已经过关系过滤，这里再用 `own_paths` 限制查询范围。
    ///
    /// 平台产品一次查出全部候选。租户产品分两次：平台账号精确匹配空路径，租户账号匹配该租户前缀，再按路径边界丢掉前缀相撞的记录。
    pub async fn find_product_visible_account_details(
        account_ids: &[String],
        product_own_paths: &str,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<Vec<IamAccountDetailResp>> {
        if account_ids.is_empty() {
            return Ok(vec![]);
        }
        let sys_ctx = IamCertServ::use_sys_ctx_unsafe(ctx.clone())?;
        let tenant_own_paths = rbum_scope_helper::get_path_item(RBUM_SCOPE_LEVEL_TENANT.to_int(), product_own_paths).unwrap_or_default();
        let ids = account_ids.to_vec();
        if tenant_own_paths.is_empty() {
            return Self::find_account_details_by_own_paths(ids, "", true, funs, &sys_ctx).await;
        }
        let mut accounts = Self::find_account_details_by_own_paths(ids.clone(), "", false, funs, &sys_ctx).await?;
        accounts.extend(Self::find_account_details_by_own_paths(ids, &tenant_own_paths, true, funs, &sys_ctx).await?);
        accounts.retain(|account| Self::account_own_paths_visible_to_product(&account.own_paths, product_own_paths));
        Ok(accounts)
    }

    async fn find_account_details_by_own_paths(
        account_ids: Vec<String>,
        own_paths: &str,
        with_sub_own_paths: bool,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<Vec<IamAccountDetailResp>> {
        IamAccountServ::find_detail_items(
            &IamAccountFilterReq {
                basic: RbumBasicFilterReq {
                    ids: Some(account_ids),
                    ignore_scope: true,
                    own_paths: Some(own_paths.to_string()),
                    with_sub_own_paths,
                    ..Default::default()
                },
                ..Default::default()
            },
            None,
            None,
            funs,
            ctx,
        )
        .await
    }

    /// 产品挂载的 Apps set。未挂到产品组时使用平台根产品组。
    async fn resolve_product_apps_set_id(product_id: &str, product_own_paths: &str, funs: &TardisFunsInst, sys_ctx: &TardisContext) -> TardisResult<Option<String>> {
        let platform_set_id = RbumSetServ::get_rbum_set_id_by_code(&Self::get_default_code(&IamSetKind::Apps, ""), true, funs, sys_ctx).await?;
        let tenant_own_paths = rbum_scope_helper::get_path_item(RBUM_SCOPE_LEVEL_TENANT.to_int(), product_own_paths).unwrap_or_default();
        let tenant_set_id = if tenant_own_paths.is_empty() {
            None
        } else {
            RbumSetServ::get_rbum_set_id_by_code(&Self::get_default_code(&IamSetKind::Apps, &tenant_own_paths), true, funs, sys_ctx).await?
        };
        let mounted_set_ids = Self::find_set_items(None, None, Some(product_id.to_string()), None, true, Some(true), funs, sys_ctx)
            .await?
            .into_iter()
            .map(|item| item.rel_rbum_set_id)
            .filter(|set_id| !set_id.is_empty())
            .collect::<HashSet<_>>();
        if let Some(tenant_set_id) = tenant_set_id.filter(|set_id| mounted_set_ids.contains(set_id)) {
            return Ok(Some(tenant_set_id));
        }
        Ok(platform_set_id)
    }

    /// 查询产品组账号按实际规则生效的角色。`account_id` 必填，支持逗号分隔的多个账号。
    /// 当前 set 有配置时用当前配置；没有时沿用平台根 set 的配置；两边都没有则不返回该账号，不含 app_read 回退。
    pub async fn find_apps_set_account_roles(
        tenant_id: Option<String>,
        account_id: Option<String>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<Vec<IamAppsSetAccountRoleResp>> {
        let tenant_ctx = TardisContext {
            own_paths: tenant_id.clone().unwrap_or_default(),
            ..ctx.clone()
        };
        let set_id = IamSetServ::get_default_set_id_by_ctx(&IamSetKind::Apps, &funs, &tenant_ctx).await?;
        let account_ids = account_id.map(|ids| ids.split(',').map(|id| id.trim().to_string()).filter(|id| !id.is_empty()).collect::<Vec<_>>()).filter(|ids| !ids.is_empty());
        let Some(account_ids) = account_ids else {
            return Err(funs.err().bad_request(
                "iam_apps_set_account_role",
                "find",
                "account_id is required",
                "400-iam-apps-set-account-role-filter-required",
            ));
        };
        let sys_ctx = IamCertServ::use_sys_ctx_unsafe(ctx.clone())?;
        let platform_set_id = RbumSetServ::get_rbum_set_id_by_code(&Self::get_default_code(&IamSetKind::Apps, ""), true, funs, &sys_ctx).await?;
        let rels = RbumRelServ::find_rbums(
            &RbumRelFilterReq {
                basic: RbumBasicFilterReq {
                    own_paths: Some("".to_string()),
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                tag: Some(IamRelKind::IamAccountAppsSetRole.to_string()),
                from_rbum_kind: Some(RbumRelFromKind::Item),
                from_rbum_ids: Some(account_ids.clone()),
                disabled: Some(false),
                ..Default::default()
            },
            None,
            Some(true),
            funs,
            &sys_ctx,
        )
        .await?;
        let mut rel_by_account_set = HashMap::new();
        for rel in rels {
            if rel.ext.is_empty() {
                continue;
            }
            rel_by_account_set.entry((rel.from_rbum_id.clone(), rel.ext.clone())).or_insert(rel);
        }
        let mut selected = Vec::new();
        for account_id in account_ids {
            if let Some(rel) = rel_by_account_set.remove(&(account_id.clone(), set_id.clone())) {
                selected.push(rel);
                continue;
            }
            let Some(platform_set_id) = platform_set_id.as_ref() else {
                continue;
            };
            if platform_set_id == &set_id {
                continue;
            }
            if let Some(rel) = rel_by_account_set.remove(&(account_id, platform_set_id.clone())) {
                selected.push(rel);
            }
        }
        let role_ids = selected.iter().map(|rel| rel.to_rbum_item_id.clone()).collect::<Vec<_>>();
        let role_codes = if role_ids.is_empty() {
            HashMap::new()
        } else {
            IamRoleServ::find_items(
                &IamRoleFilterReq {
                    basic: RbumBasicFilterReq {
                        ids: Some(role_ids),
                        ignore_scope: true,
                        own_paths: Some("".to_string()),
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
            .await?
            .into_iter()
            .map(|role| (role.id, role.code))
            .collect::<HashMap<_, _>>()
        };
        let mut result = Vec::new();
        for rel in selected {
            result.push(IamAppsSetAccountRoleResp {
                set_id: rel.ext,
                account_id: rel.from_rbum_id.clone(),
                account_name: rel.from_rbum_item_name,
                role_id: rel.to_rbum_item_id.clone(),
                role_code: role_codes.get(&rel.to_rbum_item_id).cloned().unwrap_or_default(),
                role_name: rel.to_rbum_item_name,
            });
        }
        Ok(result)
    }

    /// 保存产品组 set 与账号对应的上下文角色，并重刷该账号的应用上下文。
    pub async fn save_apps_set_account_role(req: &IamAppsSetAccountRoleReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let tenant_ctx = TardisContext {
            own_paths: req.tenant_id.clone().unwrap_or_default(),
            ..ctx.clone()
        };
        let set_id = IamSetServ::get_default_set_id_by_ctx(&IamSetKind::Apps, &funs, &tenant_ctx).await?;
        let sys_ctx = IamCertServ::use_sys_ctx_unsafe(ctx.clone())?;
        let set = RbumSetServ::get_rbum(
            &set_id,
            &RbumSetFilterReq {
                basic: RbumBasicFilterReq {
                    own_paths: Some("".to_string()),
                    with_sub_own_paths: true,
                    ignore_scope: true,
                    ..Default::default()
                },
                ..Default::default()
            },
            funs,
            &sys_ctx,
        )
        .await?;
        if set.kind != IamSetKind::Apps.to_string() {
            return Err(funs.err().bad_request("iam_apps_set_account_role", "save", "set is not an apps set", "400-iam-apps-set-kind-illegal"));
        }
        let account = IamAccountServ::get_item(
            &req.account_id,
            &IamAccountFilterReq {
                basic: RbumBasicFilterReq {
                    ignore_scope: true,
                    own_paths: Some("".to_string()),
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                ..Default::default()
            },
            funs,
            &sys_ctx,
        )
        .await?;
        if account.disabled {
            return Err(funs.err().bad_request("iam_apps_set_account_role", "save", "account is disabled", "400-iam-account-disabled"));
        }
        let role = IamRoleServ::get_item(
            &req.role_id,
            &IamRoleFilterReq {
                basic: RbumBasicFilterReq {
                    ignore_scope: true,
                    own_paths: Some("".to_string()),
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                ..Default::default()
            },
            funs,
            &sys_ctx,
        )
        .await?;
        if role.disabled || !role.in_base || !role.own_paths.is_empty() {
            return Err(funs.err().bad_request(
                "iam_apps_set_account_role",
                "save",
                "role must be an enabled global base app role",
                "400-iam-apps-set-role-not-base-app",
            ));
        }
        let existing = RbumRelServ::find_rbums(
            &RbumRelFilterReq {
                basic: RbumBasicFilterReq {
                    own_paths: Some("".to_string()),
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                tag: Some(IamRelKind::IamAccountAppsSetRole.to_string()),
                from_rbum_kind: Some(RbumRelFromKind::Item),
                from_rbum_id: Some(req.account_id.to_string()),
                ext_eq: Some(set_id.to_string()),
                disabled: Some(false),
                ..Default::default()
            },
            None,
            None,
            funs,
            &sys_ctx,
        )
        .await?;
        if existing.len() == 1 && existing[0].to_rbum_item_id == req.role_id {
            return Ok(());
        }
        for rel in &existing {
            RbumRelServ::delete_rbum(&rel.id, funs, &sys_ctx).await?;
        }
        let write_ctx = TardisContext {
            own_paths: account.own_paths.clone(),
            ..ctx.clone()
        };
        RbumRelServ::add_rel(
            &mut RbumRelAggAddReq {
                rel: RbumRelAddReq {
                    tag: IamRelKind::IamAccountAppsSetRole.to_string(),
                    note: None,
                    from_rbum_kind: RbumRelFromKind::Item,
                    from_rbum_id: req.account_id.clone(),
                    to_rbum_item_id: req.role_id.clone(),
                    to_own_paths: role.own_paths.clone(),
                    ext: Some(set_id.to_string()),
                    to_is_outside: false,
                    disabled: None,
                },
                attrs: vec![],
                envs: vec![],
            },
            funs,
            &write_ctx,
        )
        .await?;
        Self::refresh_apps_set_account_context(&req.account_id, &set.own_paths, funs, ctx).await;
        Ok(())
    }

    async fn refresh_apps_set_account_context(account_id: &str, set_own_paths: &str, funs: &TardisFunsInst, ctx: &TardisContext) {
        let refresh_ctx = TardisContext {
            own_paths: set_own_paths.to_string(),
            ..ctx.clone()
        };
        let refresh_result = if set_own_paths.is_empty() {
            IamCertServ::add_all_porject_auth(account_id, String::new(), None, funs, &refresh_ctx).await
        } else {
            IamCertServ::package_tardis_account_context_and_resp(account_id, set_own_paths, String::new(), None, funs, &refresh_ctx)
                .await
                .map(|_| ())
        };
        if let Err(err) = refresh_result {
            warn!("refresh apps set account role context failed, account_id={account_id}, set_own_paths={set_own_paths}, err={err}");
        }
    }

    pub async fn get_menu_tree_by_roles(set_id: &str, role_ids: &Vec<String>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<RbumSetTreeResp> {
        let mut role_ids = role_ids.clone();
        let set_cate_sys_code_node_len = funs.rbum_conf_set_cate_sys_code_node_len();
        let menu_sys_code = String::from_utf8(vec![b'0'; set_cate_sys_code_node_len])?;
        let mut res_ids = HashSet::new();
        let mut global_ctx = ctx.clone();
        global_ctx.own_paths = "".to_string();
        // TODO default empty res
        res_ids.insert("".to_string());
        // 按角色权限类型收集菜单资源：
        // - perm_kind = all：保留该角色关联的全部资源
        // - perm_kind = read：只保留该角色关联资源中 perm_kind = read 的只读资源
        // 只读角色明确裁掉的资源不再返回，即使默认角色也绑定了同一资源
        let read_role_ids = if role_ids.is_empty() {
            HashSet::new()
        } else {
            // 先筛出本次角色中的只读角色，避免对每个角色单独查 perm_kind
            IamRoleServ::find_id_items(
                &IamRoleFilterReq {
                    basic: RbumBasicFilterReq {
                        ids: Some(role_ids.clone()),
                        with_sub_own_paths: true,
                        own_paths: Some("".to_string()),
                        ignore_scope: true,
                        ..Default::default()
                    },
                    perm_kind: Some(IamPermKind::Read),
                    ..Default::default()
                },
                None,
                None,
                funs,
                &global_ctx,
            )
            .await?
            .into_iter()
            .collect::<HashSet<String>>()
        };
        // 如果只读角色存在，则过滤掉role_ids中只读角色的父级角色
        if !read_role_ids.is_empty() {
            for read_role_id in &read_role_ids {
                let read_role = IamRoleServ::get_item(
                    read_role_id,
                    &IamRoleFilterReq {
                        basic: RbumBasicFilterReq {
                            own_paths: Some("".to_string()),
                            with_sub_own_paths: true,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    funs,
                    ctx,
                )
                .await?;
                if !read_role.extend_role_id.is_empty() {
                    role_ids.retain(|id| *id != read_role.extend_role_id);
                }
            }
        }
        // 只读角色的关联资源先单独收集，后续再按资源 perm_kind 过滤
        let mut read_role_res_ids = HashSet::new();
        for role_id in role_ids {
            let rel_res_ids = IamRoleServ::find_simple_rels(
                &role_id,
                None,
                None,
                Some(vec![
                    RbumScopeLevelKind::Root.to_int(),
                    RbumScopeLevelKind::L1.to_int(),
                    RbumScopeLevelKind::L2.to_int(),
                    RbumScopeLevelKind::L3.to_int(),
                ]),
                Some(vec![
                    RbumScopeLevelKind::Private.to_int(),
                    RbumScopeLevelKind::Root.to_int(),
                    RbumScopeLevelKind::L1.to_int(),
                    RbumScopeLevelKind::L2.to_int(),
                    RbumScopeLevelKind::L3.to_int(),
                ]),
                funs,
                &global_ctx,
            )
            .await?.into_iter().map(|r| r.rel_id.clone()).collect_vec();
            if read_role_ids.contains(&role_id) {
                read_role_res_ids.extend(rel_res_ids.into_iter());
            } else {
                // 非只读角色：关联资源先全部收入，随后再减去只读角色裁掉的资源
                res_ids.extend(rel_res_ids.into_iter());
            }
        }
        if !read_role_res_ids.is_empty() {
            // 只读角色仅能看到其关联资源中标记为只读的资源
            let read_res_ids = IamResServ::find_id_items(
                &IamResFilterReq {
                    basic: RbumBasicFilterReq {
                        ids: Some(read_role_res_ids.iter().cloned().collect()),
                        with_sub_own_paths: true,
                        own_paths: Some("".to_string()),
                        ignore_scope: true,
                        ..Default::default()
                    },
                    perm_kind: Some(IamPermKind::Read),
                    ..Default::default()
                },
                None,
                None,
                funs,
                &global_ctx,
            )
            .await?
            .into_iter()
            .collect::<HashSet<String>>();
            // 只读角色关联、但资源本身不是只读的，视为明确裁掉
            let denied_res_ids = read_role_res_ids.difference(&read_res_ids).cloned().collect::<HashSet<String>>();
            res_ids.extend(read_res_ids);
            // 被只读角色裁掉的资源不再返回，即使默认角色也绑定了它
            res_ids.retain(|id| !denied_res_ids.contains(id));
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

    /// 按按钮展示范围过滤菜单树中的按钮。
    /// 平台层（own_paths 为空）保留全部按钮；租户层和应用层去掉 show_scope = platform 的按钮。
    pub async fn filter_button_show_scope(tree: &mut IamResSetTreeResp, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        if ctx.own_paths.is_empty() {
            return Ok(());
        }
        let Some(ext) = tree.ext.as_ref() else {
            return Ok(());
        };
        let res_ids = ext.items.values().flatten().map(|item| item.rel_rbum_item_id.clone()).unique().collect_vec();
        if res_ids.is_empty() {
            return Ok(());
        }
        let global_ctx = TardisContext {
            own_paths: "".to_string(),
            ..ctx.clone()
        };
        let platform_button_ids = IamResServ::find_items(
            &IamResFilterReq {
                basic: RbumBasicFilterReq {
                    ids: Some(res_ids),
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                kind: Some(IamResKind::Ele),
                ..Default::default()
            },
            None,
            None,
            funs,
            &global_ctx,
        )
        .await?
        .into_iter()
        .filter(|res| res.show_scope == IamResShowScope::Platform)
        .map(|res| res.id)
        .collect::<HashSet<String>>();
        if platform_button_ids.is_empty() {
            return Ok(());
        }
        if let Some(ext) = tree.ext.as_mut() {
            for items in ext.items.values_mut() {
                items.retain(|item| !platform_button_ids.contains(&item.rel_rbum_item_id));
            }
            ext.item_perm_kinds.retain(|id, _| !platform_button_ids.contains(id));
            ext.item_data_guards.retain(|id, _| !platform_button_ids.contains(id));
        }
        if let Some(ext) = tree.ext.as_mut() {
            ext.item_number_agg = Self::aggregate_item_numbers(&tree.main, &ext.items);
        }
        Ok(())
    }

    fn aggregate_item_numbers(main: &[RbumSetTreeNodeResp], items: &HashMap<String, Vec<RbumSetItemRelInfoResp>>) -> HashMap<String, HashMap<String, u64>> {
        let count_items = |node_items: &[&RbumSetItemRelInfoResp]| -> HashMap<String, u64> {
            let mut grouped: HashMap<String, HashSet<String>> = HashMap::new();
            for item in node_items {
                grouped.entry(item.rel_rbum_item_kind_id.clone()).or_default().insert(item.rel_rbum_item_id.clone());
            }
            grouped.into_iter().map(|(kind_id, ids)| (kind_id, ids.len() as u64)).collect()
        };
        let mut item_number_agg = main
            .iter()
            .map(|cate| {
                let cate_items = main
                    .iter()
                    .filter(|node| node.sys_code.starts_with(&cate.sys_code))
                    .flat_map(|node| items.get(&node.id).into_iter().flatten())
                    .collect_vec();
                (cate.id.clone(), count_items(&cate_items))
            })
            .collect::<HashMap<String, HashMap<String, u64>>>();
        let root_items = items.values().flatten().collect_vec();
        item_number_agg.insert("".to_string(), count_items(&root_items));
        item_number_agg
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
        sys_codes: Option<Vec<String>>,
        sys_code_query_kind: Option<RbumSetCateLevelQueryKind>,
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
                rel_rbum_set_cate_sys_codes: sys_codes,
                sys_code_query_kind,
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

    async fn resolve_apps_set_item_ctx(
        set_cate_id: Option<String>,
        tenant_id: Option<String>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<(TardisContext, String, Option<String>)> {
        if !ctx.own_paths.is_empty() {
            let set_id = Self::get_default_set_id_by_ctx(&IamSetKind::Apps, funs, ctx).await?;
            return Ok((ctx.clone(), set_id, set_cate_id));
        }
        if Self::is_platform_apps_root_cate_id(set_cate_id.as_deref(), funs) {
            let set_id = Self::ensure_platform_apps_set(funs, ctx).await?;
            return Ok((ctx.clone(), set_id, None));
        }
        if let Some(tid) = tenant_id.clone().or_else(|| set_cate_id.clone()) {
            if let Ok(t_ctx) = IamCertServ::use_tenant_ctx(ctx.clone(), &tid) {
                if let Ok(set_id) = Self::get_default_set_id_by_ctx(&IamSetKind::Apps, funs, &t_ctx).await {
                    let is_tenant_virtual_root = set_cate_id.as_ref().is_none_or(|c| c.is_empty() || c == &tid);
                    if is_tenant_virtual_root {
                        return Ok((t_ctx, set_id, None));
                    }
                    if tenant_id.is_some() {
                        return Ok((t_ctx, set_id, set_cate_id));
                    }
                }
            }
        }
        Err(funs.err().bad_request(
            "iam_apps_set_item",
            "resolve",
            "tenant_id is required when binding to tenant app cates on platform context",
            "400-iam-apps-tree-tenant-id-required",
        ))
    }

    /// Add app set item with platform / tenant virtual root cate id mapping.
    pub async fn add_apps_set_item(
        set_cate_id: Option<String>,
        tenant_id: Option<String>,
        sort: i64,
        rel_rbum_item_id: String,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<String> {
        let (item_ctx, set_id, cate_id) = Self::resolve_apps_set_item_ctx(set_cate_id, tenant_id, funs, ctx).await?;
        Self::add_set_item(
            &IamSetItemAddReq {
                set_id,
                set_cate_id: cate_id.unwrap_or_default(),
                sort,
                rel_rbum_item_id,
            },
            funs,
            &item_ctx,
        )
        .await
    }

    /// Find app set items with platform / tenant virtual root cate id mapping.
    pub async fn find_apps_set_items(
        set_cate_id: Option<String>,
        item_id: Option<String>,
        tenant_id: Option<String>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<Vec<RbumSetItemDetailResp>> {
        let (item_ctx, set_id, cate_id) = Self::resolve_apps_set_item_ctx(set_cate_id, tenant_id, funs, ctx).await?;
        if cate_id.is_none() {
            Self::find_set_items_with_none_set_cate_id(Some(set_id), None, item_id, true, funs, &item_ctx).await
        } else {
            Self::find_set_items(Some(set_id), cate_id, item_id, None, true, None, funs, &item_ctx).await
        }
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
                item_perm_kinds: HashMap::new(),
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
            let unique_res_ids = res_set_item_ids.iter().unique().cloned().collect_vec();
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

            if !unique_res_ids.is_empty() {
                ext.item_perm_kinds = IamResServ::find_items(
                    &IamResFilterReq {
                        basic: RbumBasicFilterReq {
                            ids: Some(unique_res_ids),
                            with_sub_own_paths: true,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    None,
                    None,
                    funs,
                    &global_ctx,
                )
                .await?
                .into_iter()
                .map(|res| (res.id, res.perm_kind))
                .collect();
            }
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::IamSetServ;

    #[test]
    fn product_tenant_account_paths_stay_in_tenant() {
        assert!(IamSetServ::account_own_paths_visible_to_product("", "tenant/app"));
        assert!(IamSetServ::account_own_paths_visible_to_product("tenant", "tenant/app"));
        assert!(IamSetServ::account_own_paths_visible_to_product("tenant/app", "tenant/app"));
        assert!(!IamSetServ::account_own_paths_visible_to_product("tenant2", "tenant/app"));
        assert!(!IamSetServ::account_own_paths_visible_to_product("other", "tenant/app"));
        assert!(IamSetServ::account_own_paths_visible_to_product("other", ""));
    }
}
