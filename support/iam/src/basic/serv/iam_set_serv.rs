use std::collections::{HashMap, HashSet};

use bios_basic::rbum::dto::rbum_filer_dto::{RbumBasicFilterReq, RbumRelFilterReq, RbumSetCateFilterReq, RbumSetFilterReq, RbumSetItemFilterReq, RbumSetTreeFilterReq};
use bios_basic::rbum::dto::rbum_rel_agg_dto::RbumRelAggAddReq;
use bios_basic::rbum::dto::rbum_rel_dto::{RbumRelAddReq, RbumRelDetailResp};
use bios_basic::rbum::dto::rbum_set_cate_dto::{RbumSetCateAddReq, RbumSetCateModifyReq, RbumSetCateSummaryResp};
use bios_basic::rbum::dto::rbum_set_dto::{RbumSetAddReq, RbumSetPathResp, RbumSetTreeMainResp, RbumSetTreeResp};
use bios_basic::rbum::dto::rbum_set_item_dto::{RbumSetItemAddReq, RbumSetItemDetailResp, RbumSetItemModifyReq};
use bios_basic::rbum::helper::rbum_scope_helper;
use bios_basic::rbum::rbum_config::RbumConfigApi;
use bios_basic::rbum::rbum_enumeration::{RbumRelFromKind, RbumScopeLevelKind, RbumSetCateLevelQueryKind};
use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;
use bios_basic::rbum::serv::rbum_rel_serv::RbumRelServ;
use bios_basic::rbum::serv::rbum_set_serv::{RbumSetCateServ, RbumSetItemServ, RbumSetServ};
use tardis::basic::dto::TardisContext;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::futures::future::BoxFuture;
use tardis::futures::FutureExt;
use tardis::serde_json::json;
use tardis::{TardisFuns, TardisFunsInst};

use crate::basic::dto::iam_set_dto::{IamSetCateAddReq, IamSetCateModifyReq, IamSetItemAddReq, IamSetTreeResp};
use crate::iam_config::IamBasicConfigApi;
use crate::iam_constants::{RBUM_SCOPE_LEVEL_APP, RBUM_SCOPE_LEVEL_TENANT};
use crate::iam_enumeration::{IamRelKind, IamSetCateKind, IamSetKind};

use super::iam_rel_serv::IamRelServ;

const SET_AND_ITEM_SPLIT_FLAG: &str = ":";

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
            },
            funs,
            ctx,
        )
        .await?;
        let cates = if set_kind == IamSetKind::Res {
            let cate_menu_id = RbumSetCateServ::add_rbum(
                &mut RbumSetCateAddReq {
                    name: TrimString("Menus".to_string()),
                    bus_code: TrimString("__menus__".to_string()),
                    icon: None,
                    sort: None,
                    ext: Some(IamSetCateKind::Root.to_string()),
                    rbum_parent_cate_id: None,
                    rel_rbum_set_id: set_id.clone(),
                    scope_level: Some(scope_level.clone()),
                },
                funs,
                ctx,
            )
            .await?;
            let cate_api_id = RbumSetCateServ::add_rbum(
                &mut RbumSetCateAddReq {
                    name: TrimString("Apis".to_string()),
                    bus_code: TrimString("__apis__".to_string()),
                    icon: None,
                    sort: None,
                    ext: None,
                    rbum_parent_cate_id: None,
                    rel_rbum_set_id: set_id.clone(),
                    scope_level: Some(scope_level.clone()),
                },
                funs,
                ctx,
            )
            .await?;
            Some((cate_menu_id, cate_api_id))
        } else {
            None
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
        RbumSetCateServ::add_rbum(
            &mut RbumSetCateAddReq {
                name: add_req.name.clone(),
                bus_code: add_req.bus_code.as_ref().unwrap_or(&TrimString("".to_string())).clone(),
                icon: add_req.icon.clone(),
                sort: add_req.sort,
                ext: add_req.ext.clone(),
                rbum_parent_cate_id: add_req.rbum_parent_cate_id.clone(),
                rel_rbum_set_id: set_id.to_string(),
                scope_level: add_req.scope_level.clone(),
            },
            funs,
            ctx,
        )
        .await
    }

    pub async fn modify_set_cate(set_cate_id: &str, modify_req: &IamSetCateModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        RbumSetCateServ::modify_rbum(
            set_cate_id,
            &mut RbumSetCateModifyReq {
                bus_code: modify_req.bus_code.clone(),
                name: modify_req.name.clone(),
                icon: modify_req.icon.clone(),
                sort: modify_req.sort,
                ext: modify_req.ext.clone(),
                scope_level: modify_req.scope_level.clone(),
            },
            funs,
            ctx,
        )
        .await
    }

    pub async fn delete_set_cate(set_cate_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<u64> {
        RbumSetCateServ::delete_rbum(set_cate_id, funs, ctx).await
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

    pub async fn get_tree(set_id: &str, filter: &mut RbumSetTreeFilterReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<IamSetTreeResp> {
        filter.rel_rbum_item_domain_ids = Some(vec![funs.iam_basic_domain_iam_id()]);
        let resp = RbumSetServ::get_tree(set_id, filter, funs, ctx).await?;
        Self::find_rel_set_cate(set_id, resp, filter, funs, ctx).await
    }
    //todo bugfix
    // find set_cate 对应的set_id,返回set_id下面set_cate
    // 返回的节点里面，如果有通过关联关系而来的cate（不属于此set_id的），会在ext中标识它真正的set_id
    #[allow(dead_code)]
    async fn find_rel_set_cate(set_id: &str, resp: RbumSetTreeResp, filter: &mut RbumSetTreeFilterReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<IamSetTreeResp> {
        let mut result_main: Vec<RbumSetTreeMainResp> = vec![];
        let mut resp_items = HashMap::new();
        let mut resp_item_domains = HashMap::new();
        let mut resp_item_kinds = HashMap::new();
        let mut resp_item_number_agg = HashMap::new();
        let mut resp_rel = None;

        //from set to find set_cate
        let set_cate_rel = RbumRelServ::find_one_rbum(
            &RbumRelFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                tag: Some(IamRelKind::IamOrgRel.to_string()),
                from_rbum_kind: Some(RbumRelFromKind::Set),
                from_rbum_id: Some(set_id.to_string()),
                from_rbum_scope_levels: None,
                to_rbum_item_scope_levels: None,
                to_rbum_item_id: None,
                to_own_paths: Some("".to_string()),
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?;
        if let Some(set_cate_rel) = set_cate_rel {
            resp_rel = Some(set_cate_rel.to_rbum_item_id.clone());
            if let Some(set_cate_resp) = RbumSetCateServ::find_one_rbum(
                &RbumSetCateFilterReq {
                    basic: RbumBasicFilterReq {
                        with_sub_own_paths: true,
                        own_paths:Some(set_cate_rel.to_own_paths.clone()),
                        ids: Some(vec![set_cate_rel.to_rbum_item_id.clone()]),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                funs,
                ctx,
            )
            .await?
            {
                let mut mock_filter = filter.clone();
                mock_filter.sys_codes = Some(vec![set_cate_resp.sys_code.clone()]);
                mock_filter.sys_code_query_kind = Some(RbumSetCateLevelQueryKind::Sub);
                mock_filter.sys_code_query_depth = Some(99);
                let mock_ctx = TardisContext {
                    own_paths: set_cate_rel.to_own_paths.clone(),
                    ..ctx.clone()
                };
                let set_id = Self::get_default_set_id_by_ctx(&IamSetKind::Org, funs, &mock_ctx).await.unwrap();
                let sys_resp = RbumSetServ::get_tree(&set_id, &mock_filter, funs, &mock_ctx).await.unwrap();

                result_main.extend(
                    sys_resp
                        .main
                        .iter()
                        .map(|rm| {
                            let mut r = rm.clone();
                            r.ext = json!({"set_id":set_id.clone(),"own_paths":set_cate_rel.to_own_paths}).to_string();
                            r
                        })
                        .collect::<Vec<RbumSetTreeMainResp>>(),
                );
                if mock_filter.fetch_cate_item {
                    let ext_resp = resp.ext.clone().unwrap();
                    resp_items.extend(ext_resp.items);
                    resp_item_domains.extend(ext_resp.item_domains);
                    resp_item_kinds.extend(ext_resp.item_kinds);
                    resp_item_number_agg.extend(ext_resp.item_number_agg);
                }
            };
        }

        //from set_cate to find set
        for mut r in resp.main.clone() {
            if let Some(set_rel) = RbumRelServ::find_one_rbum(
                &RbumRelFilterReq {
                    basic: RbumBasicFilterReq {
                        own_paths: Some("".to_string()),
                        with_sub_own_paths: true,
                        ..Default::default()
                    },
                    tag: Some(IamRelKind::IamOrgRel.to_string()),
                    from_rbum_kind: Some(RbumRelFromKind::Set),
                    to_rbum_item_id: Some(r.id.to_string()),
                    to_own_paths: Some("".to_string()),
                    ..Default::default()
                },
                funs,
                ctx,
            )
            .await?
            {
                let new_resp = RbumSetTreeMainResp {
                    rel: Some(set_rel.from_rbum_id.clone()),
                    ..r.clone()
                };
                r = new_resp
            };
            let set_rel = RbumRelServ::find_one_rbum(
                &RbumRelFilterReq {
                    basic: RbumBasicFilterReq {
                        with_sub_own_paths: true,
                        ..Default::default()
                    },
                    tag: Some(IamRelKind::IamOrgRel.to_string()),
                    from_rbum_kind: Some(RbumRelFromKind::Set),
                    to_rbum_item_id: Some(r.id.to_string()),
                    to_own_paths: Some("".to_string()),
                    ..Default::default()
                },
                funs,
                ctx,
            )
            .await?;
            if let Some(set_rel) = set_rel {
                let mock_ctx = TardisContext {
                    own_paths: set_rel.own_paths.clone(),
                    ..ctx.clone()
                };
                let mut set_filter = filter.clone();
                set_filter.sys_codes = None;
                let mut tenant_resp = RbumSetServ::get_tree(&set_rel.from_rbum_id, &set_filter, funs, &mock_ctx).await?;
                let mut resp_tenant_node: Vec<RbumSetTreeMainResp> = tenant_resp
                    .main
                    .clone()
                    .iter()
                    .filter(|r_main| r_main.pid.is_none())
                    .map(|r_main| RbumSetTreeMainResp {
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
                            r.ext = json!({"set_id":set_rel.from_rbum_id.clone(),"own_paths":set_rel.own_paths}).to_string();
                            r
                        })
                        .collect::<Vec<RbumSetTreeMainResp>>(),
                );
                if set_filter.fetch_cate_item {
                    let ext_resp = tenant_resp.ext.unwrap();
                    resp_items.extend(ext_resp.items);
                    resp_item_domains.extend(ext_resp.item_domains);
                    resp_item_kinds.extend(ext_resp.item_kinds);
                    resp_item_number_agg.extend(ext_resp.item_number_agg);
                }
            }
            //把原来的resp.main完全拷贝到result_main中
            result_main.push(r);
        }
        let mut result = IamSetTreeResp {
            main: result_main,
            ext: None,
            rel: resp_rel,
        };
        if filter.fetch_cate_item {
            let mut ext_resp = resp.ext.clone().unwrap();
            ext_resp.items.extend(resp_items);
            ext_resp.item_domains.extend(resp_item_domains);
            ext_resp.item_kinds.extend(resp_item_kinds);
            ext_resp.item_number_agg.extend(resp_item_number_agg);
            result.ext = Some(ext_resp);
        }
        Ok(result)
    }
    pub async fn get_tree_with_auth_by_account(set_id: &str, account_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<IamSetTreeResp> {
        let tree_with_account = Self::get_tree(
            set_id,
            &mut RbumSetTreeFilterReq {
                fetch_cate_item: true,
                hide_item_with_disabled: true,
                rel_rbum_item_ids: Some(vec![account_id.to_string()]),
                rel_rbum_item_kind_ids: Some(vec![funs.iam_basic_kind_account_id()]),
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?;
        let tree_ext = tree_with_account.ext.as_ref().unwrap();
        let account_rel_sys_codes = tree_with_account.main.into_iter().filter(|cate| !tree_ext.items[&cate.id].is_empty()).map(|cate| cate.sys_code).collect::<Vec<String>>();
        if account_rel_sys_codes.is_empty() {
            return Ok(IamSetTreeResp {
                main: vec![],
                ext: None,
                rel: None,
            });
        }
        Self::get_tree(
            set_id,
            &mut RbumSetTreeFilterReq {
                fetch_cate_item: true,
                sys_codes: Some(account_rel_sys_codes),
                sys_code_query_kind: Some(RbumSetCateLevelQueryKind::CurrentAndSub),
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await
    }

    pub async fn get_menu_tree_by_roles(set_id: &str, role_ids: &Vec<String>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<RbumSetTreeResp> {
        let set_cate_sys_code_node_len = funs.rbum_conf_set_cate_sys_code_node_len();
        let menu_sys_code = String::from_utf8(vec![b'0'; set_cate_sys_code_node_len])?;
        let mut res_ids = HashSet::new();
        let mut global_ctx = ctx.clone();
        global_ctx.own_paths = "".to_string();
        // todo default empty res
        res_ids.insert("".to_string());
        for role_id in role_ids {
            let rel_res_ids = IamRelServ::find_to_id_rels(&IamRelKind::IamResRole, role_id, None, None, funs, &global_ctx).await?;
            res_ids.extend(rel_res_ids.into_iter());
        }
        let mut filter = RbumSetTreeFilterReq {
            fetch_cate_item: true,
            hide_cate_with_empty_item: true,
            sys_codes: Some(vec![menu_sys_code]),
            sys_code_query_kind: Some(RbumSetCateLevelQueryKind::CurrentAndSub),
            ..Default::default()
        };
        if !res_ids.is_empty() {
            filter.rel_rbum_item_ids = Some(res_ids.into_iter().collect());
        }
        RbumSetServ::get_tree(set_id, &filter, funs, ctx).await
    }

    pub async fn get_menu_tree(set_id: &str, exts: Option<String>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<RbumSetTreeResp> {
        let cate_exts = exts.map(|exts| exts.split(',').map(|r| r.to_string()).collect());
        let set_cate_sys_code_node_len = funs.rbum_conf_set_cate_sys_code_node_len();
        let menu_sys_code = String::from_utf8(vec![b'0'; set_cate_sys_code_node_len])?;
        Self::get_tree_with_sys_code(set_id, &menu_sys_code, cate_exts, funs, ctx).await
    }

    pub async fn get_api_tree(set_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<RbumSetTreeResp> {
        let set_cate_sys_code_node_len = funs.rbum_conf_set_cate_sys_code_node_len();
        let api_sys_code = TardisFuns::field.incr_by_base36(&String::from_utf8(vec![b'0'; set_cate_sys_code_node_len])?).unwrap();
        Self::get_tree_with_sys_code(set_id, &api_sys_code, None, funs, ctx).await
    }

    pub async fn get_cate_id_with_sys_code(set_id: &str, filter_sys_code: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<String> {
        let rbum_cate = RbumSetCateServ::find_one_rbum(
            &RbumSetCateFilterReq {
                rel_rbum_set_id: Some(set_id.to_string()),
                sys_codes: Some(vec![filter_sys_code.to_string()]),
                sys_code_query_kind: Some(RbumSetCateLevelQueryKind::Current),
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?;
        Ok(rbum_cate.as_ref().map(|r| r.id.clone()).unwrap_or_default())
    }

    async fn get_tree_with_sys_code(
        set_id: &str,
        filter_sys_code: &str,
        cate_exts: Option<Vec<String>>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<RbumSetTreeResp> {
        RbumSetServ::get_tree(
            set_id,
            &RbumSetTreeFilterReq {
                fetch_cate_item: true,
                sys_codes: Some(vec![filter_sys_code.to_string()]),
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
        RbumSetItemServ::add_rbum(
            &mut RbumSetItemAddReq {
                sort: add_req.sort,
                rel_rbum_set_id: add_req.set_id.clone(),
                rel_rbum_set_cate_id: add_req.set_cate_id.clone(),
                rel_rbum_item_id: add_req.rel_rbum_item_id.clone(),
            },
            funs,
            ctx,
        )
        .await
    }

    pub async fn modify_set_item(set_item_id: &str, modify_req: &mut RbumSetItemModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        RbumSetItemServ::modify_rbum(set_item_id, modify_req, funs, ctx).await
    }

    pub async fn delete_set_item(set_item_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<u64> {
        RbumSetItemServ::delete_rbum(set_item_id, funs, ctx).await
    }

    pub async fn find_set_items(
        set_id: Option<String>,
        set_cate_id: Option<String>,
        item_id: Option<String>,
        with_sub: bool,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<Vec<RbumSetItemDetailResp>> {
        RbumSetItemServ::find_detail_rbums(
            &RbumSetItemFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: with_sub,
                    ..Default::default()
                },
                rel_rbum_item_disabled: Some(false),
                rel_rbum_set_id: set_id.clone(),
                rel_rbum_set_cate_ids: set_cate_id.map(|r| vec![r]),
                rel_rbum_item_ids: item_id.map(|i| vec![i]),
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
                    table_rbum_set_cate_is_left: Some(true),
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
            Self::find_set_items(set_id, set_cate_id, item_id, with_sub, funs, ctx).await
        }
    }

    pub async fn find_set_paths(set_item_id: &str, set_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Vec<Vec<RbumSetPathResp>>> {
        RbumSetItemServ::find_set_paths(set_item_id, set_id, funs, ctx).await
    }

    pub async fn find_flat_set_items(set_id: &str, item_id: &str, with_sub: bool, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<HashMap<String, String>> {
        let items = Self::find_set_items(Some(set_id.to_string()), None, Some(item_id.to_string()), with_sub, funs, ctx).await?;
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

    /// 绑定租户的set_id to 平台 set_cate_id
    pub async fn bind_cate_with_platform(cate_id: &String, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let set_id = IamSetServ::get_default_set_id_by_ctx(&IamSetKind::Org, funs, ctx).await?;
        //删除原来的关联
        let old_rel = RbumRelServ::find_one_rbum(
            &RbumRelFilterReq {
                basic: Default::default(),
                tag: Some(IamRelKind::IamOrgRel.to_string()),
                from_rbum_kind: Some(RbumRelFromKind::Set),
                from_rbum_id: Some(set_id.clone()),
                from_rbum_scope_levels: None,
                to_rbum_item_id: None,
                to_rbum_item_scope_levels: None,
                to_own_paths: Some("".to_string()),
                ext_eq: None,
                ext_like: None,
            },
            funs,
            ctx,
        )
        .await?;
        if let Some(old_rel) = old_rel {
            RbumRelServ::delete_rbum(&old_rel.id, funs, ctx).await?;
        };
        RbumRelServ::add_rel(
            &mut RbumRelAggAddReq {
                rel: RbumRelAddReq {
                    tag: IamRelKind::IamOrgRel.to_string(),
                    note: None,
                    from_rbum_kind: RbumRelFromKind::Set,
                    from_rbum_id: set_id,
                    to_rbum_item_id: cate_id.to_string(),
                    to_own_paths: "".to_string(),
                    to_is_outside: true,
                    ext: None,
                },
                attrs: vec![],
                envs: vec![],
            },
            funs,
            ctx,
        )
        .await?;
        Ok(())
    }

    /// 解绑 租户的set_id to 平台 set_cate_id
    pub async fn unbind_cate_with_platform(old_rel: RbumRelDetailResp, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let mock_ctx = TardisContext {
            own_paths: old_rel.to_own_paths,
            ..ctx.clone()
        };
        let platform_set_id = IamSetServ::get_default_set_id_by_ctx(&IamSetKind::Org, funs, &mock_ctx).await?;
        let tenant_set_id = IamSetServ::get_default_set_id_by_ctx(&IamSetKind::Org, funs, ctx).await?;
        if let Some(set_cate_resp) = RbumSetCateServ::find_one_rbum(
            &RbumSetCateFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
                    ids: Some(vec![old_rel.to_rbum_item_id.clone()]),
                    ..Default::default()
                },
                ..Default::default()
            },
            funs,
            &mock_ctx,
        )
        .await?
        {
            let platform_cates = RbumSetServ::get_tree(
                &platform_set_id,
                &RbumSetTreeFilterReq {
                    sys_codes: Some(vec![set_cate_resp.sys_code.clone()]),
                    sys_code_query_kind: Some(RbumSetCateLevelQueryKind::Sub),
                    sys_code_query_depth: Some(99),
                    ..Default::default()
                },
                funs,
                &mock_ctx,
            )
            .await?;
            Self::copy_tree_to_new_set(&platform_cates.main.iter().collect::<Vec<&RbumSetTreeMainResp>>(), &tenant_set_id, None, None, funs, ctx).await?;
        }
        RbumRelServ::delete_rbum(&old_rel.id, funs, ctx).await?;
        Ok(())
    }

    fn copy_tree_to_new_set<'a>(
        tree: &'a [&RbumSetTreeMainResp],
        set_id: &'a str,
        old_pid: Option<String>,
        new_pid: Option<String>,
        funs: &'a TardisFunsInst,
        ctx: &'a TardisContext,
    ) -> BoxFuture<'a, TardisResult<String>> {
        async move {
            let mut vec1 = tree.to_owned();
            vec1.retain(|cate| cate.pid == old_pid);
            for r in vec1 {
                let new_cate_id = Self::add_set_cate(
                    set_id,
                    &IamSetCateAddReq {
                        name: TrimString(r.name.clone()),
                        scope_level: Some(r.scope_level.clone()),
                        bus_code: None,
                        icon: None,
                        sort: None,
                        ext: None,
                        rbum_parent_cate_id: new_pid.clone(),
                    },
                    funs,
                    ctx,
                )
                .await?;
                let mut vec = tree.to_owned();
                vec.retain(|c| c.sys_code.starts_with(&r.sys_code) && c.sys_code != r.sys_code);
                Self::copy_tree_to_new_set(&vec, set_id, r.pid.clone(), Some(new_cate_id), funs, ctx).await?;
            }
            Ok("".to_string())
        }
        .boxed()
    }
}
