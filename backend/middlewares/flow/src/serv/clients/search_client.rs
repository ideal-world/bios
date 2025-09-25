use std::{collections::HashMap, fmt::Display, str::FromStr, vec};

use bios_basic::rbum::{dto::rbum_filer_dto::RbumBasicFilterReq, helper::rbum_scope_helper, rbum_enumeration::RbumScopeLevelKind, serv::rbum_item_serv::RbumItemCrudOperation};
use bios_sdk_invoke::{
    clients::spi_search_client::SpiSearchClient,
    dto::search_item_dto::{
        AdvSearchItemQueryReq, BasicQueryCondInfo, BasicQueryOpKind, SearchItemAddReq, SearchItemModifyReq, SearchItemQueryReq, SearchItemSearchCtxReq, SearchItemSearchPageReq,
        SearchItemSearchReq, SearchItemSearchResp, SearchItemVisitKeysReq,
    },
};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tardis::{
    basic::{dto::TardisContext, error::TardisError, field::TrimString, result::TardisResult},
    tokio,
    web::{poem_openapi::types::ToJSON, web_resp::TardisPage},
    TardisFuns, TardisFunsInst,
};

use crate::{
    dto::{
        flow_inst_dto::{FlowInstFilterReq, ModifyObjSearchExtReq},
        flow_model_dto::{FlowModelDetailResp, FlowModelFilterReq},
        flow_state_dto::FlowGuardConf,
    },
    flow_constants,
    serv::{flow_inst_serv::FlowInstServ, flow_model_serv::FlowModelServ},
};

const SEARCH_MODEL_TAG: &str = "flow_model";
const SEARCH_INSTANCE_TAG: &str = "flow_approve_inst";

/// 日志任务类型
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum FlowSearchTaskKind {
    /// 修改业务
    ModifyBusinessObj,
    /// 添加工作流实例
    AddInstance,
    /// 修改工作流实例
    ModifyInstance,
    /// 添加或修改工作流模型
    ModifyModel,
    /// 删除工作流模型
    DeleteModel,
}

impl Display for FlowSearchTaskKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FlowSearchTaskKind::ModifyBusinessObj => write!(f, "ModifyBusinessObj"),
            FlowSearchTaskKind::AddInstance => write!(f, "AddInstance"),
            FlowSearchTaskKind::ModifyInstance => write!(f, "ModifyInstance"),
            FlowSearchTaskKind::ModifyModel => write!(f, "ModifyModel"),
            FlowSearchTaskKind::DeleteModel => write!(f, "DeleteModel"),
        }
    }
}

impl FromStr for FlowSearchTaskKind {
    type Err = TardisError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ModifyBusinessObj" => Ok(Self::ModifyBusinessObj),
            "AddInstance" => Ok(Self::AddInstance),
            "ModifyInstance" => Ok(Self::ModifyInstance),
            "ModifyModel" => Ok(Self::ModifyModel),
            "DeleteModel" => Ok(Self::DeleteModel),
            _ => Err(TardisError::bad_request(&format!("invalid FlowSearchTaskKind: {}", s), "400-operator-invalid-param")),
        }
    }
}

pub struct FlowSearchClient;

impl FlowSearchClient {
    pub async fn add_search_task(kind: &FlowSearchTaskKind, key: &str, val: &str, _funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let task_key = format!("search_{}_{}", kind, key);
        let val = match *kind {
            FlowSearchTaskKind::ModifyInstance => val.to_string(),
            FlowSearchTaskKind::AddInstance => val.to_string(),
            FlowSearchTaskKind::ModifyModel => val.to_string(),
            FlowSearchTaskKind::DeleteModel => val.to_string(),
            FlowSearchTaskKind::ModifyBusinessObj => {
                let mut req = ctx.get_ext(&task_key).await?.map_or(ModifyObjSearchExtReq::default(), |s| TardisFuns::json.str_to_obj(&s).unwrap_or_default());
                let modify_req = TardisFuns::json.str_to_obj::<ModifyObjSearchExtReq>(val)?;
                req.tag = modify_req.tag;
                if modify_req.status.is_some() {
                    req.status = modify_req.status;
                }
                if modify_req.rel_state.is_some() {
                    req.rel_state = modify_req.rel_state;
                }
                if modify_req.rel_transition_state_name.is_some() {
                    req.rel_transition_state_name = modify_req.rel_transition_state_name;
                }
                if modify_req.current_state_color.is_some() {
                    req.current_state_color = modify_req.current_state_color;
                }
                TardisFuns::json.obj_to_string(&req)?
            }
        };
        ctx.remove_ext(&task_key).await?;
        ctx.add_ext(&task_key, &val).await?;
        Ok(())
    }

    pub async fn execute_async_task(key: &str, val: &str, ctx: &TardisContext) -> TardisResult<()> {
        let funs = flow_constants::get_tardis_inst();
        let (kind, id) = key.split_once('_').unwrap_or_default();
        match FlowSearchTaskKind::from_str(kind)? {
            FlowSearchTaskKind::AddInstance => {
                Self::async_add_or_modify_instance_search(id, Box::new(false), &funs, ctx).await?;
            }
            FlowSearchTaskKind::ModifyInstance => {
                Self::async_add_or_modify_instance_search(id, Box::new(true), &funs, ctx).await?;
            }
            FlowSearchTaskKind::ModifyModel => {
                Self::async_add_or_modify_model_search(id, Box::new(true), &funs, ctx).await?;
            }
            FlowSearchTaskKind::DeleteModel => {
                Self::async_delete_model_search(id, &funs, ctx).await?;
            }
            FlowSearchTaskKind::ModifyBusinessObj => {
                let req = TardisFuns::json.str_to_obj::<ModifyObjSearchExtReq>(val)?;
                Self::async_modify_business_obj_search_ext(id, &req, &funs, ctx).await?;
            }
        }
        Ok(())
    }
    pub async fn refresh_business_obj_search(rel_business_obj_id: &str, tag: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let (rel_state, rel_transition_state_name) = FlowInstServ::find_detail_items(
            &FlowInstFilterReq {
                rel_business_obj_ids: Some(vec![rel_business_obj_id.to_string()]),
                main: Some(false),
                finish: Some(false),
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?
        .pop()
        .map(|inst| (inst.artifacts.unwrap_or_default().state, inst.current_state_name))
        .unwrap_or_default();
        let req = ModifyObjSearchExtReq {
            tag: tag.to_string(),
            rel_state: Some(rel_state.map_or("".to_string(), |s| s.to_string())),
            rel_transition_state_name: Some(rel_transition_state_name.unwrap_or_default()),
            ..Default::default()
        };
        let val = TardisFuns::json.obj_to_string(&req)?;
        Self::add_search_task(&FlowSearchTaskKind::ModifyBusinessObj, rel_business_obj_id, &val, funs, ctx).await
    }

    pub async fn modify_business_obj_search_ext_status(rel_business_obj_id: &str, tag: &str, status: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let tag_search_map = Self::get_tag_search_map();
        if let Some((table, _kind)) = tag_search_map.get(tag) {
            let ext = json!({
                "status": status,
            });
            SpiSearchClient::modify_item_and_name(
                table,
                rel_business_obj_id,
                &SearchItemModifyReq {
                    kind: None,
                    title: None,
                    name: None,
                    content: None,
                    owner: None,
                    own_paths: None,
                    create_time: None,
                    update_time: None,
                    ext: Some(ext),
                    ext_override: Some(false),
                    visit_keys: None,
                    kv_disable: None,
                },
                funs,
                ctx,
            )
            .await
            .unwrap_or_default();
        }

        Ok(())
    }

    pub async fn async_modify_business_obj_search_ext(rel_business_obj_id: &str, req: &ModifyObjSearchExtReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let ctx_clone = ctx.clone();
        let rel_business_obj_id_cp = rel_business_obj_id.to_string();
        let mut req_cp = req.clone();
        // 获取当前对象的状态信息
        if let Some(inst_id) = FlowInstServ::get_inst_ids_by_rel_business_obj_id(vec![rel_business_obj_id.to_string()], Some(true), funs, ctx).await?.pop() {
            let inst = FlowInstServ::get(&inst_id, funs, ctx).await?;
            if req_cp.status.is_none() {
                req_cp.status = inst.current_state_name.clone();
            }
            if req_cp.current_state_color.is_none() {
                req_cp.current_state_color = inst.current_state_color.clone();
            }
        }
        ctx.add_async_task(Box::new(|| {
            Box::pin(async move {
                let rel_business_obj_id_cp2 = rel_business_obj_id_cp.clone();
                let task_handle = tokio::spawn(async move {
                    let funs = flow_constants::get_tardis_inst();
                    Self::modify_business_obj_search_ext(&rel_business_obj_id_cp, &req_cp, &funs, &ctx_clone).await
                });
                match task_handle.await {
                    Ok(_) => {}
                    Err(e) => tardis::log::error!("Flow Instance {} async_modify_business_obj_search_ext error:{:?}", rel_business_obj_id_cp2, e),
                }
                Ok(())
            })
        }))
        .await
    }

    pub async fn modify_business_obj_search_ext(rel_business_obj_id: &str, req: &ModifyObjSearchExtReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let tag_search_map = Self::get_tag_search_map();
        if let Some((table, _kind)) = tag_search_map.get(&req.tag) {
            let mut ext = json!({});
            if let Some(status) = &req.status {
                if let Some(ext_mut) = ext.as_object_mut() {
                    ext_mut.insert("status".to_string(), status.to_json().unwrap_or_default());
                }
            }
            if let Some(rel_state) = &req.rel_state {
                if let Some(ext_mut) = ext.as_object_mut() {
                    ext_mut.insert("rel_state".to_string(), rel_state.to_json().unwrap_or_default());
                }
            }
            if let Some(rel_transition_state_name) = &req.rel_transition_state_name {
                if let Some(ext_mut) = ext.as_object_mut() {
                    ext_mut.insert("rel_transition_state_name".to_string(), rel_transition_state_name.to_json().unwrap_or_default());
                }
            }
            if let Some(current_state_color) = &req.current_state_color {
                if let Some(ext_mut) = ext.as_object_mut() {
                    ext_mut.insert("current_state_color".to_string(), current_state_color.to_json().unwrap_or_default());
                }
            }
            return SpiSearchClient::modify_item_and_name(
                table,
                rel_business_obj_id,
                &SearchItemModifyReq {
                    kind: None,
                    title: None,
                    name: None,
                    content: None,
                    owner: None,
                    own_paths: None,
                    create_time: None,
                    update_time: None,
                    ext: Some(ext),
                    ext_override: Some(false),
                    visit_keys: None,
                    kv_disable: None,
                },
                funs,
                ctx,
            )
            .await;
        }

        Ok(())
    }

    pub async fn async_add_or_modify_model_search(model_id: &str, is_modify: Box<bool>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let ctx_clone = ctx.clone();
        let mock_ctx = TardisContext {
            own_paths: "".to_string(),
            ..ctx.clone()
        };
        let model_resp = FlowModelServ::get_item(
            model_id,
            &FlowModelFilterReq {
                basic: RbumBasicFilterReq {
                    own_paths: Some("".to_string()),
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                ..Default::default()
            },
            funs,
            &mock_ctx,
        )
        .await?;
        let model_id_cp = model_id.to_string();
        ctx.add_async_task(Box::new(|| {
            Box::pin(async move {
                let task_handle = tokio::spawn(async move {
                    let funs = flow_constants::get_tardis_inst();
                    let _ = Self::add_or_modify_model_search(&model_resp, is_modify, &funs, &ctx_clone).await;
                });
                match task_handle.await {
                    Ok(_) => {}
                    Err(e) => tardis::log::error!("Flow model {} async_add_or_modify_model_search error:{:?}", model_id_cp, e),
                }
                Ok(())
            })
        }))
        .await
    }

    pub async fn async_delete_model_search(model_id: &str, _funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let ctx_clone = ctx.clone();
        let model_id_cp = model_id.to_string();
        ctx.add_async_task(Box::new(|| {
            Box::pin(async move {
                let model_id_cp2 = model_id_cp.clone();
                let task_handle = tokio::spawn(async move {
                    let funs = flow_constants::get_tardis_inst();
                    let _ = Self::delete_model_search(&model_id_cp, &funs, &ctx_clone).await;
                });
                match task_handle.await {
                    Ok(_) => {}
                    Err(e) => tardis::log::error!("Flow model {} async_delete_model_search error:{:?}", model_id_cp2, e),
                }
                Ok(())
            })
        }))
        .await
    }

    // flow model 全局搜索埋点方法
    async fn add_or_modify_model_search(model_resp: &FlowModelDetailResp, is_modify: Box<bool>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let model_id = &model_resp.id;
        // 数据共享权限处理
        let mut visit_tenants = vec![rbum_scope_helper::get_path_item(RbumScopeLevelKind::L1.to_int(), &model_resp.own_paths).unwrap_or_default()];
        let mut visit_apps = rbum_scope_helper::get_path_item(RbumScopeLevelKind::L2.to_int(), &model_resp.own_paths).map(|app| vec![app]).unwrap_or_default();
        let mut own_paths = Some(model_resp.own_paths.clone());
        if model_resp.scope_level == RbumScopeLevelKind::Root {
            visit_apps.push("".to_string());
            visit_tenants.push("".to_string());
            own_paths = Some("".to_string());
        }
        let key = model_id.clone();
        if *is_modify {
            let modify_req = SearchItemModifyReq {
                kind: Some(SEARCH_MODEL_TAG.to_string()),
                title: Some(model_resp.name.clone()),
                name: Some(model_resp.name.clone()),
                content: Some(model_resp.name.clone()),
                owner: if model_resp.owner.is_empty() { None } else { Some(model_resp.owner.clone()) },
                own_paths,
                create_time: Some(model_resp.create_time),
                update_time: Some(model_resp.update_time),
                ext: Some(json!({
                    "default": model_resp.default,
                    "tag": model_resp.tag,
                    "icon": model_resp.icon,
                    "info": model_resp.info,
                    "rel_template_ids": model_resp.rel_template_ids,
                    "scope_level": model_resp.scope_level,
                    "tenant_id": model_resp.own_paths.clone(),
                })),
                ext_override: Some(true),
                visit_keys: Some(SearchItemVisitKeysReq {
                    accounts: None,
                    apps: Some(visit_apps),
                    tenants: Some(visit_tenants),
                    roles: None,
                    groups: None,
                }),
                kv_disable: None,
            };
            SpiSearchClient::modify_item_and_name(SEARCH_MODEL_TAG, &key, &modify_req, funs, ctx).await?;
        } else {
            let add_req = SearchItemAddReq {
                tag: SEARCH_MODEL_TAG.to_string(),
                kind: SEARCH_MODEL_TAG.to_string(),
                key: TrimString(key),
                title: model_resp.name.clone(),
                content: model_resp.name.clone(),
                owner: if model_resp.owner.is_empty() { None } else { Some(model_resp.owner.clone()) },
                own_paths,
                create_time: Some(model_resp.create_time),
                update_time: Some(model_resp.update_time),
                ext: Some(json!({
                    "default": model_resp.default,
                    "tag": model_resp.tag,
                    "icon": model_resp.icon,
                    "info": model_resp.info,
                    "rel_template_ids": model_resp.rel_template_ids,
                    "scope_level": model_resp.scope_level,
                    "tenant_id": model_resp.own_paths.clone(),
                })),
                visit_keys: Some(SearchItemVisitKeysReq {
                    accounts: None,
                    apps: Some(visit_apps),
                    tenants: Some(visit_tenants),
                    roles: None,
                    groups: None,
                }),
                kv_disable: None,
                data_source: None,
            };
            SpiSearchClient::add_item_and_name(&add_req, Some(model_resp.name.clone()), funs, ctx).await?;
        }
        Ok(())
    }

    // model 全局搜索删除埋点方法
    async fn delete_model_search(model_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        SpiSearchClient::delete_item_and_name(SEARCH_MODEL_TAG, model_id, funs, ctx).await?;
        Ok(())
    }

    pub async fn async_add_or_modify_instance_search(inst_id: &str, is_modify: Box<bool>, _funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let ctx_clone = ctx.clone();
        let inst_id_cp = inst_id.to_string();
        ctx.add_async_task(Box::new(|| {
            Box::pin(async move {
                let inst_id_cp2 = inst_id_cp.clone();
                let task_handle = tokio::spawn(async move {
                    let funs = flow_constants::get_tardis_inst();
                    let _ = Self::add_or_modify_instance_search(&inst_id_cp, is_modify, &funs, &ctx_clone).await;
                });
                match task_handle.await {
                    Ok(_) => {}
                    Err(e) => tardis::log::error!("Flow Instance {} add_or_modify_instance_search error:{:?}", inst_id_cp2, e),
                }
                Ok(())
            })
        }))
        .await
    }

    // flow inst 全局搜索埋点方法
    pub async fn add_or_modify_instance_search(inst_id: &str, is_modify: Box<bool>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let mock_ctx = TardisContext {
            own_paths: "".to_string(),
            ..ctx.clone()
        };
        let inst_resp = FlowInstServ::get_search_item(inst_id, funs, &mock_ctx).await?;
        if !inst_resp.rel_inst_id.clone().is_none_or(|id| id.is_empty()) {
            return Ok(());
        }
        // 数据共享权限处理
        let tenant = rbum_scope_helper::get_path_item(RbumScopeLevelKind::L1.to_int(), &inst_resp.own_paths).unwrap_or_default();
        let app = rbum_scope_helper::get_path_item(RbumScopeLevelKind::L2.to_int(), &inst_resp.own_paths).unwrap_or_default();
        let visit_tenants = vec![tenant.clone()];
        let visit_apps = vec![app.clone()];
        let own_paths = Some(inst_resp.own_paths.clone());
        let key = inst_id;
        if *is_modify {
            let modify_req = SearchItemModifyReq {
                kind: Some(SEARCH_INSTANCE_TAG.to_string()),
                title: inst_resp.title.clone(),
                name: inst_resp.name.clone(),
                content: inst_resp.content.clone(),
                owner: Some(inst_resp.owner.clone()),
                own_paths,
                create_time: inst_resp.create_time,
                update_time: inst_resp.update_time,
                ext: Some(json!({
                    "code": inst_resp.code,
                    "tag": inst_resp.tag,
                    "current_state_id": &inst_resp.current_state_id,
                    "rel_business_obj_name": inst_resp.rel_business_obj_name,
                    "current_state_name": inst_resp.current_state_name,
                    "current_state_kind": inst_resp.current_state_kind,
                    "rel_business_obj_id": inst_resp.rel_business_obj_id,
                    "finish_time": inst_resp.finish_time,
                    "op_time": inst_resp.update_time,
                    "state": inst_resp.state,
                    "rel_transition": inst_resp.rel_transition.clone().unwrap_or_default().to_string(),
                    "his_operators": inst_resp.his_operators,
                    "curr_operators": inst_resp.curr_operators,
                    "curr_referral": inst_resp.curr_referral,
                    "tenant_id": tenant.clone(),
                    "app_id": app.clone(),
                })),
                ext_override: Some(true),
                visit_keys: Some(SearchItemVisitKeysReq {
                    accounts: None,
                    apps: Some(visit_apps),
                    tenants: Some(visit_tenants),
                    roles: None,
                    groups: None,
                }),
                kv_disable: None,
            };
            SpiSearchClient::modify_item_and_name(SEARCH_INSTANCE_TAG, key, &modify_req, funs, ctx).await?;
        } else {
            let add_req = SearchItemAddReq {
                tag: SEARCH_INSTANCE_TAG.to_string(),
                kind: SEARCH_INSTANCE_TAG.to_string(),
                key: TrimString(key),
                title: inst_resp.title.clone().unwrap_or_default(),
                content: inst_resp.content.clone().unwrap_or_default(),
                owner: Some(inst_resp.owner.clone()),
                own_paths,
                create_time: inst_resp.create_time,
                update_time: inst_resp.update_time,
                ext: Some(json!({
                    "tag": inst_resp.tag,
                    "current_state_id": inst_resp.current_state_id,
                    "current_state_name": inst_resp.current_state_name,
                    "current_state_kind": inst_resp.current_state_kind,
                    "rel_business_obj_name": inst_resp.rel_business_obj_name,
                    "rel_business_obj_id": inst_resp.rel_business_obj_id,
                    "finish_time": inst_resp.finish_time,
                    "op_time": inst_resp.update_time,
                    "state": inst_resp.state,
                    "rel_transition": inst_resp.rel_transition.clone().unwrap_or_default().to_string(),
                    "his_operators": inst_resp.his_operators,
                    "curr_operators": inst_resp.curr_operators,
                    "tenant_id": tenant.clone(),
                    "app_id": app.clone(),
                })),
                visit_keys: Some(SearchItemVisitKeysReq {
                    accounts: None,
                    apps: Some(visit_apps),
                    tenants: Some(visit_tenants),
                    roles: None,
                    groups: None,
                }),
                kv_disable: None,
                data_source: None,
            };
            SpiSearchClient::add_item_and_name(&add_req, inst_resp.title.clone(), funs, ctx).await?;
        }
        Ok(())
    }

    pub async fn search(search_req: &SearchItemSearchReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Option<TardisPage<SearchItemSearchResp>>> {
        SpiSearchClient::search(search_req, funs, ctx).await
    }

    pub async fn search_guard_accounts(guard_conf: &FlowGuardConf, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Vec<String>> {
        if guard_conf.guard_by_spec_account_ids.is_empty() && guard_conf.guard_by_spec_org_ids.is_empty() && guard_conf.guard_by_spec_role_ids.is_empty() {
            return Ok(vec![]);
        }
        let mut query = SearchItemQueryReq::default();
        let mut adv_query = vec![];

        if !guard_conf.guard_by_spec_account_ids.is_empty() {
            query.keys = Some(guard_conf.guard_by_spec_account_ids.clone().into_iter().map(|account_id| account_id.into()).collect_vec());
        }

        if !guard_conf.guard_by_spec_org_ids.is_empty() {
            adv_query.push(AdvSearchItemQueryReq {
                group_by_or: Some(true),
                ext_by_or: Some(true),
                ext: Some(
                    guard_conf
                        .guard_by_spec_org_ids
                        .clone()
                        .into_iter()
                        .map(|org_id| BasicQueryCondInfo {
                            field: "dept_id".to_string(),
                            op: BasicQueryOpKind::In,
                            value: org_id.to_json().unwrap_or(json!("")),
                        })
                        .collect_vec(),
                ),
            });
        }
        if !guard_conf.guard_by_spec_role_ids.is_empty() {
            adv_query.push(AdvSearchItemQueryReq {
                group_by_or: Some(true),
                ext_by_or: Some(true),
                ext: Some(
                    guard_conf
                        .guard_by_spec_role_ids
                        .clone()
                        .into_iter()
                        .map(|role_id| BasicQueryCondInfo {
                            field: "role_id".to_string(),
                            op: BasicQueryOpKind::In,
                            value: role_id.to_json().unwrap_or(json!("")),
                        })
                        .collect_vec(),
                ),
            });
        }
        if !adv_query.is_empty() {
            adv_query[0].group_by_or = Some(false);
        }
        let result = SpiSearchClient::search(
            &SearchItemSearchReq {
                tag: "iam_account".to_string(),
                ctx: SearchItemSearchCtxReq::default(),
                query,
                adv_by_or: Some(!guard_conf.guard_by_spec_account_ids.is_empty()),
                adv_query: Some(adv_query),
                sort: None,
                page: SearchItemSearchPageReq {
                    number: 1,
                    size: 999,
                    fetch_total: false,
                },
            },
            funs,
            ctx,
        )
        .await?
        .map(|result| result.records.into_iter().map(|record| record.key).collect_vec())
        .unwrap_or_default();
        Ok(result)
    }

    pub fn get_tag_search_map() -> HashMap<String, (String, String)> {
        HashMap::from([
            ("CTS".to_string(), ("idp_test".to_string(), "idp_test_cts".to_string())),
            ("ISSUE".to_string(), ("idp_test".to_string(), "idp_test_issue".to_string())),
            ("ITER".to_string(), ("idp_product".to_string(), "idp_feed_iter".to_string())),
            ("MS".to_string(), ("idp_product".to_string(), "idp_feed_ms".to_string())),
            ("PRODUCT".to_string(), ("idp_product".to_string(), "idp_product".to_string())),
            ("PROJECT_MS".to_string(), ("idp_product".to_string(), "idp_feed_project_ms".to_string())),
            ("PROJ".to_string(), ("idp_product".to_string(), "idp_project".to_string())),
            ("REQ".to_string(), ("idp_product".to_string(), "idp_feed_req".to_string())),
            ("TASK".to_string(), ("idp_product".to_string(), "idp_feed_task".to_string())),
            ("TICKET".to_string(), ("ticket".to_string(), "ticket_inst".to_string())),
            ("TP".to_string(), ("idp_test".to_string(), "idp_test_plan".to_string())),
            ("TS".to_string(), ("idp_test".to_string(), "idp_test_stage".to_string())),
            ("TC".to_string(), ("idp_test".to_string(), "idp_test_case".to_string())),
            ("REVIEW".to_string(), ("idp_product".to_string(), "idp_feed_review".to_string())),
        ])
    }
}
