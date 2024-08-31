use std::collections::HashMap;

use crate::basic::dto::iam_filer_dto::IamRoleFilterReq;
use crate::basic::dto::iam_res_dto::{IamResAddReq, IamResAggAddReq, IamResAppReq, IamResSummaryResp};
use crate::basic::dto::iam_role_dto::IamRoleAggModifyReq;
use crate::basic::dto::iam_set_dto::{IamSetCateAddReq, IamSetItemAggAddReq};
use crate::basic::serv::clients::iam_kv_client::IamKvClient;
use crate::basic::serv::iam_rel_serv::IamRelServ;
use crate::basic::serv::iam_res_serv::IamResServ;
use crate::basic::serv::iam_role_serv::IamRoleServ;
use crate::basic::serv::iam_set_serv::IamSetServ;
use crate::iam_constants;
use crate::iam_enumeration::{IamRelKind, IamResKind, IamSetKind};
use bios_basic::helper::request_helper::try_set_real_ip_from_req_to_ctx;
use bios_basic::rbum::dto::rbum_filer_dto::{RbumBasicFilterReq, RbumSetItemFilterReq};
use bios_basic::rbum::dto::rbum_set_dto::{RbumSetTreeCateResp, RbumSetTreeResp};
use bios_basic::rbum::rbum_enumeration::{RbumScopeLevelKind, RbumSetCateLevelQueryKind};
use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;
use bios_basic::rbum::serv::rbum_item_serv::{RbumItemCrudOperation, RbumItemServ};
use bios_basic::rbum::serv::rbum_set_serv::RbumSetItemServ;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use tardis::basic::dto::TardisContext;
use tardis::log::warn;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem::web::Json;
use tardis::web::poem::Request;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::param::Query;
use tardis::web::web_resp::{TardisApiResult, TardisResp, Void};
use tardis::TardisFuns;

#[derive(Clone, Default)]
pub struct IamCcResApi;

/// Common Console Res API
/// 通用控制台资源API
#[poem_openapi::OpenApi(prefix_path = "/cc/res", tag = "bios_basic::ApiTag::Common")]
impl IamCcResApi {
    /// Find Menu Tree
    /// 查找菜单树
    #[oai(path = "/tree", method = "get")]
    async fn get_menu_tree(&self, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<RbumSetTreeResp> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        let set_id = IamSetServ::get_set_id_by_code(&IamSetServ::get_default_code(&IamSetKind::Res, ""), true, &funs, &ctx.0).await?;
        let result = IamSetServ::get_menu_tree_by_roles(&set_id, &ctx.0.roles, &funs, &ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// build Menu Tree
    /// 构造菜单树
    #[oai(path = "/tree/build", method = "get")]
    async fn build_menu_tree(&self, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<RbumSetTreeCateResp> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        let set_id = IamSetServ::get_set_id_by_code(&IamSetServ::get_default_code(&IamSetKind::Res, ""), true, &funs, &ctx.0).await?;
        let result = IamSetServ::get_menu_tree_by_roles(&set_id, &ctx.0.roles, &funs, &ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result.to_trees())
    }

    /// Find res by apps
    /// 根据应用查找资源
    #[oai(path = "/res", method = "get")]
    async fn get_res_by_app(&self, app_ids: Query<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<HashMap<String, Vec<IamResSummaryResp>>> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        let ids = app_ids.0.split(',').map(|s| s.to_string()).collect();
        let result = IamResServ::get_res_by_app_code(ids, None, &funs, &ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Find res by apps and code
    /// 根据应用和资源编码查找资源
    #[oai(path = "/res", method = "put")]
    async fn get_res_by_app_code(&self, res_req: Json<IamResAppReq>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<HashMap<String, Vec<IamResSummaryResp>>> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        let result = IamResServ::get_res_by_app_code(res_req.0.app_ids, Some(res_req.0.res_codes), &funs, &ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    #[oai(path = "/rebuild_menu_res", method = "get")]
    async fn rebuild_menu_res(&self, _ctx: TardisContextExtractor, _request: &Request) -> TardisApiResult<Void> {
        #[derive(Serialize, Deserialize, Debug)]
        struct MenuConfig {
            name: String,
            title: String,
            ext: String,
            id: String,
            pid: String,
            res: Option<Vec<OpConfig>>,
            api: Option<Vec<String>>,
        }
        #[derive(Serialize, Deserialize, Debug)]
        struct OpConfig {
            code: String,
            name: String,
        }
        let mut funs = iam_constants::get_tardis_inst();
        let global_ctx = TardisContext::default();
        funs.begin().await?;
        warn!("rebuild_menu_res: begin task");
        let mut bind_res = vec![]; // 绑定的资源ID
                                   // find old menu res
        let set_id = IamSetServ::get_default_set_id_by_ctx(&IamSetKind::Res, &funs, &global_ctx).await?;
        let old_menus = IamSetServ::get_menu_tree(&set_id, None, &funs, &global_ctx).await?.main.into_iter().filter(|menu| menu.sys_code != "0000").collect_vec();
        for old_menu in &old_menus {
            // delete set cate item
            let set_items = RbumSetItemServ::find_detail_rbums(
                &RbumSetItemFilterReq {
                    rel_rbum_set_id: Some(set_id.clone()),
                    sys_code_query_kind: Some(RbumSetCateLevelQueryKind::CurrentAndSub),
                    rel_rbum_set_cate_sys_codes: Some(vec![old_menu.sys_code.clone()]),
                    ..Default::default()
                },
                None,
                None,
                &funs,
                &global_ctx,
            )
            .await?;
            for set_item in set_items {
                IamResServ::delete_res(&set_item.rel_rbum_item_id, &funs, &global_ctx).await?;
            }
        }
        warn!("rebuild_menu_res: delete res finished");
        // delete set cate
        let mut deleted_menus = vec![];
        while old_menus.len() != deleted_menus.len() {
            for old_menu in old_menus.iter() {
                if deleted_menus.contains(&old_menu.id) {
                    continue;
                }
                match IamSetServ::delete_set_cate(&old_menu.id, &funs, &global_ctx).await {
                    Err(_) => {}
                    Ok(_) => deleted_menus.push(old_menu.id.clone()),
                };
            }
            warn!("rebuild_menu_res: deleting set cate deleted_menus:{:?}", deleted_menus);
        }
        warn!("rebuild_menu_res: delete set cate finished");
        let menu_root_id = IamSetServ::get_cate_id_with_sys_code(set_id.as_str(), Some(vec!["0000".to_string()]), &funs, &global_ctx).await?;
        // init new menu
        let mut new_menu_map = HashMap::new();
        let menu_json = TardisFuns::json.json_to_obj::<Vec<MenuConfig>>(IamKvClient::get_item("basic:kv:init_menu", &funs, &global_ctx).await?.unwrap().value).unwrap();
        for new_menu in menu_json {
            // add set cate
            let set_cate_id = IamSetServ::add_set_cate(
                &set_id,
                &IamSetCateAddReq {
                    name: new_menu.title.clone().into(),
                    scope_level: Some(RbumScopeLevelKind::Root),
                    bus_code: None,
                    icon: None,
                    sort: None,
                    ext: Some(new_menu.ext.clone()),
                    rbum_parent_cate_id: if new_menu.pid.is_empty() {
                        Some(menu_root_id.clone())
                    } else {
                        new_menu_map.get(&new_menu.pid).cloned()
                    },
                },
                &funs,
                &global_ctx,
            )
            .await?;
            new_menu_map.insert(new_menu.id.clone(), set_cate_id.clone());
            // add page res
            let page_id = IamResServ::add_res_agg(
                &mut IamResAggAddReq {
                    res: IamResAddReq {
                        code: new_menu.name.into(),
                        hide: Some(false),
                        kind: IamResKind::Menu,
                        name: new_menu.title.into(),
                        scope_level: Some(RbumScopeLevelKind::Root),
                        ..Default::default()
                    },
                    set: IamSetItemAggAddReq { set_cate_id: set_cate_id.clone() },
                },
                &set_id,
                &funs,
                &global_ctx,
            )
            .await?;
            if new_menu.ext == "Root" || new_menu.ext == "System" {
                bind_res.push(page_id.clone());
            }
            // add ele res
            if let Some(res) = new_menu.res {
                for ele in res {
                    let ele_id = IamResServ::add_res_agg(
                        &mut IamResAggAddReq {
                            res: IamResAddReq {
                                code: ele.code.into(),
                                kind: IamResKind::Ele,
                                name: ele.name.into(),
                                scope_level: Some(RbumScopeLevelKind::Root),
                                ..Default::default()
                            },
                            set: IamSetItemAggAddReq { set_cate_id: set_cate_id.clone() },
                        },
                        &set_id,
                        &funs,
                        &global_ctx,
                    )
                    .await?;
                    if new_menu.ext == "Root" || new_menu.ext == "System" {
                        bind_res.push(ele_id);
                    }
                }
            }
            if let Some(apis) = new_menu.api {
                for api_code in apis {
                    if let Some(api) = RbumItemServ::find_one_rbum(
                        &RbumBasicFilterReq {
                            code: Some(api_code),
                            ..Default::default()
                        },
                        &funs,
                        &global_ctx,
                    )
                    .await?
                    {
                        IamRelServ::add_simple_rel(&IamRelKind::IamResApi, &api.id, &page_id, None, None, false, false, &funs, &global_ctx).await?;
                    }
                }
            }
        }
        warn!("rebuild_menu_res: add set cate finished");
        // bind res with sys_admin_role
        let sys_role_id = IamRoleServ::find_one_item(
            &IamRoleFilterReq {
                basic: RbumBasicFilterReq {
                    code: Some("sys_admin".to_string()),
                    ..Default::default()
                },
                ..Default::default()
            },
            &funs,
            &global_ctx,
        )
        .await?
        .map(|role| role.id)
        .unwrap_or_default();
        IamRoleServ::modify_role_agg(
            &sys_role_id,
            &mut IamRoleAggModifyReq {
                role: None,
                res_ids: Some(bind_res),
            },
            &funs,
            &global_ctx,
        )
        .await?;
        warn!("rebuild_menu_res: bind res with sys_admin_role finished");
        funs.commit().await?;
        TardisResp::ok(Void {})
    }
}
