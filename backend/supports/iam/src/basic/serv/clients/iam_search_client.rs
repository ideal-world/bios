use std::collections::{HashMap, HashSet};

use bios_basic::rbum::{
    dto::rbum_filer_dto::{RbumBasicFilterReq, RbumItemRelFilterReq, RbumSetFilterReq, RbumSetItemFilterReq},
    serv::{rbum_crud_serv::RbumCrudOperation, rbum_item_serv::RbumItemCrudOperation, rbum_set_serv::{RbumSetItemServ, RbumSetServ}},
};
use bios_sdk_invoke::{
    clients::spi_search_client::SpiSearchClient,
    dto::search_item_dto::{SearchItemAddReq, SearchItemModifyReq, SearchItemVisitKeysReq},
};
use itertools::Itertools;

use tardis::{
    TardisFunsInst, basic::{dto::TardisContext, field::TrimString, result::TardisResult}, serde_json::{self, Value, json}, tokio
};

use crate::{
    basic::{
        dto::{
            iam_account_dto::IamAccountDetailAggResp,
            iam_filer_dto::{IamAccountFilterReq, IamAppFilterReq, IamRoleFilterReq, IamTenantFilterReq},
        },
        serv::{
            clients::iam_kv_client::IamKvClient, iam_account_serv::IamAccountServ, iam_app_serv::IamAppServ, iam_role_serv::IamRoleServ, iam_set_serv::IamSetServ, iam_sub_deploy_serv::IamSubDeployServ, iam_tenant_serv::IamTenantServ
        },
    },
    iam_config::IamConfig,
    iam_constants,
    iam_enumeration::{IamRelKind, IamSetKind},
};
pub struct IamSearchClient;

impl IamSearchClient {
    pub async fn async_add_or_modify_account_search(account_id: &str, is_modify: Box<bool>, logout_msg: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let ctx_clone = ctx.clone();
        let account_resp = IamAccountServ::get_account_detail_aggs(
            account_id,
            &IamAccountFilterReq {
                basic: RbumBasicFilterReq {
                    ignore_scope: true,
                    own_paths: Some("".to_string()),
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                ..Default::default()
            },
            true,
            true,
            true,
            funs,
            ctx,
        )
        .await?;
        let logout_msg = logout_msg.to_string();
        ctx.add_async_task(Box::new(|| {
            Box::pin(async move {
                let task_handle = tokio::spawn(async move {
                    let funs = iam_constants::get_tardis_inst();
                    let _ = Self::add_or_modify_account_search(account_resp, is_modify, &logout_msg, &funs, &ctx_clone).await;
                });
                task_handle.await.unwrap();
                Ok(())
            })
        }))
        .await
    }

    pub async fn sync_add_or_modify_account_search(account_id: &str, is_modify: Box<bool>, logout_msg: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let ctx_clone = ctx.clone();
        let mock_ctx = TardisContext {
            own_paths: "".to_string(),
            ..ctx.clone()
        };
        let account_resp = IamAccountServ::get_account_detail_aggs(
            account_id,
            &IamAccountFilterReq {
                basic: RbumBasicFilterReq {
                    ignore_scope: true,
                    own_paths: Some("".to_string()),
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                ..Default::default()
            },
            true,
            true,
            true,
            funs,
            &mock_ctx,
        )
        .await?;
        let logout_msg = logout_msg.to_string();
        ctx.add_sync_task(Box::new(|| {
            Box::pin(async move {
                let task_handle = tokio::spawn(async move {
                    let funs = iam_constants::get_tardis_inst();
                    let _ = Self::add_or_modify_account_search(account_resp, is_modify, &logout_msg, &funs, &ctx_clone).await;
                });
                task_handle.await.unwrap();
                Ok(())
            })
        }))
        .await
    }

    pub async fn async_delete_account_search(account_id: String, _funs: &TardisFunsInst, ctx: TardisContext) -> TardisResult<()> {
        let ctx_clone = ctx.clone();
        ctx.add_async_task(Box::new(|| {
            Box::pin(async move {
                let task_handle = tokio::spawn(async move {
                    let funs = iam_constants::get_tardis_inst();
                    let _ = Self::delete_account_search(&account_id, &funs, &ctx_clone).await;
                });
                task_handle.await.unwrap();
                Ok(())
            })
        }))
        .await
    }

    // account 全局搜索埋点方法
    pub async fn add_or_modify_account_search(
        account_resp: IamAccountDetailAggResp,
        is_modify: Box<bool>,
        logout_msg: &str,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<()> {
        let account_id = account_resp.id.as_str();
        let account_certs = account_resp.certs.iter().map(|m| m.1.clone()).collect::<Vec<String>>();
        // let account_app_ids: Vec<String> = account_resp.apps.iter().map(|a| a.app_id.clone()).collect();
        let mut account_resp_dept_id = vec![];
        let mut account_resp_dept_map: HashMap<String, Value> = HashMap::new();
        let global_ctx = TardisContext {
            own_paths: "".to_owned(),
            ..ctx.clone()
        };
        let sub_deploy_ids = IamSubDeployServ::find_sub_deploy_id_by_rel_id(&IamRelKind::IamSubDeployAccount, account_id, &funs, &global_ctx).await?;
        let auth_sub_deploy_ids = IamSubDeployServ::find_sub_deploy_id_by_rel_id(&IamRelKind::IamSubDeployAuthAccount, account_id, &funs, &global_ctx).await?;
        let mock_ctx = TardisContext {
            own_paths: "".to_owned(),
            ..ctx.clone()
        };
        let mut set_ids = vec![];
        if account_resp.own_paths.is_empty() {
            let tenants = IamTenantServ::find_items(
                &IamTenantFilterReq {
                    basic: RbumBasicFilterReq {
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
                ctx,
            )
            .await?;
            if let Ok(set_id) = IamSetServ::get_set_id_by_code(&IamSetServ::get_default_code(&IamSetKind::Org, ""), true, funs, &mock_ctx).await {
                set_ids.push(set_id);
            }
            for t in tenants {
                if let Ok(set_id) = IamSetServ::get_set_id_by_code(&IamSetServ::get_default_code(&IamSetKind::Org, &t.id), true, funs, &mock_ctx).await {
                    set_ids.push(set_id);
                }
            }
        } else if let Ok(set_id) = IamSetServ::get_set_id_by_code(&IamSetServ::get_default_code(&IamSetKind::Org, &account_resp.own_paths), true, funs, &mock_ctx).await {
            set_ids.push(set_id);
        };
        for set_id in set_ids {
            let set_items = IamSetServ::find_set_items(Some(set_id), None, Some(account_id.to_string()), None, true, None, funs, &mock_ctx).await?;
            account_resp_dept_id.extend(set_items.iter().filter_map(|s| s.rel_rbum_set_cate_id.clone()).collect::<Vec<_>>());
            for set_item in set_items {
                account_resp_dept_map.insert(set_item.rel_rbum_set_cate_id.clone().unwrap_or_default(), json!({
                    "name": set_item.rel_rbum_set_cate_name,
                    "own_paths": set_item.own_paths,
                    "scope_level": set_item.rel_rbum_item_scope_level,
                }));
            }
        }

        let tag = funs.conf::<IamConfig>().spi.search_account_tag.clone();
        let key = account_id.to_string();
        let raw_account_apps = IamAppServ::find_items(
            &IamAppFilterReq {
                basic: RbumBasicFilterReq {
                    own_paths: Some("".to_string()),
                    with_sub_own_paths: true,
                    enabled: Some(true),
                    ..Default::default()
                },
                rel: Some(RbumItemRelFilterReq {
                    rel_by_from: false,
                    rel_item_id: Some(account_id.to_string()),
                    tag: Some(IamRelKind::IamAccountApp.to_string()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            None,
            None,
            funs,
            &mock_ctx,
        )
        .await?;
        let account_app_ids = raw_account_apps.iter().map(|app| app.id.clone()).collect_vec();
        let raw_account_apps_map = raw_account_apps
            .into_iter()
            .map(|app| {
                (
                    app.id.clone(),
                    json!({
                        "name": app.name,
                        "own_paths": app.own_paths,
                        "scope_level": app.scope_level,
                    }),
                )
            })
            .collect::<HashMap<String, serde_json::Value>>();
        let raw_roles = IamRoleServ::find_items(
            &IamRoleFilterReq {
                basic: RbumBasicFilterReq {
                    own_paths: Some("".to_string()),
                    with_sub_own_paths: true,
                    enabled: Some(true),
                    ..Default::default()
                },
                rel: Some(RbumItemRelFilterReq {
                    rel_by_from: false,
                    rel_item_id: Some(account_id.to_string()),
                    tag: Some(IamRelKind::IamAccountRole.to_string()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            None,
            None,
            funs,
            &mock_ctx,
        )
        .await?;
        let mut roles_set = HashSet::new();
        let mut raw_roles_map = HashMap::new();
        for role in raw_roles {
            roles_set.insert(role.id.clone());
            raw_roles_map.insert(
                role.id.clone(),
                json!({
                    "name": role.name,
                    "own_paths": role.own_paths,
                    "scope_level": role.scope_level,
                }),
            );
        }
        let account_roles = roles_set.into_iter().collect_vec();

        // 产品组
        let mut app_set = vec![];
        let set_cate = RbumSetItemServ::find_detail_rbums(&RbumSetItemFilterReq {
            basic: RbumBasicFilterReq {
                own_paths: Some("".to_string()),
                with_sub_own_paths: true,
                ..Default::default()
            },
            rel_rbum_item_ids: Some(vec![account_id.to_string()]),
            rel_rbum_set_id: Some(IamSetServ::get_set_id_by_code(&IamSetServ::get_default_code(&IamSetKind::Apps, ""), true, funs, &mock_ctx).await?),
            ..Default::default()
        }, None, None, funs, ctx).await?;
        let set_ids = RbumSetServ::find_id_rbums(&RbumSetFilterReq {
            basic: RbumBasicFilterReq {
                ids: Some(set_cate.iter().map(|cate| cate.rel_rbum_set_id.clone()).collect_vec()),
                own_paths: Some("".to_string()),
                with_sub_own_paths: true,
                ..Default::default()
            },
            ..Default::default()
        }, None, None, funs, ctx).await?;
        for set_cate in set_cate {
            if set_ids.contains(&set_cate.rel_rbum_set_id.clone()) {
                app_set.push(json!({
                    "name": set_cate.rel_rbum_set_cate_name,
                    "own_paths": set_cate.own_paths,
                    "scope_level": set_cate.rel_rbum_item_scope_level,
                }));
            }
        }
        // 岗位
        let primary_code = account_resp.exts.iter().find(|attr| attr.name == "primary").map(|attr| attr.value.clone());
        let secondary_code = account_resp.exts.iter().find(|attr| attr.name == "secondary").map(|attr| attr.value.clone());
        let standard_level_map = IamKvClient::get_item_value("__tag__:_:standardLevel", funs, ctx).await?.unwrap_or_default();
        let position_map = IamKvClient::get_item_value("__tag__:_:position:all", funs, ctx).await?.unwrap_or_default();
        let raw_primary = if let Some(pri) = primary_code.clone() {
            Some(
                json!({
                    "name": position_map.iter().find(|pos| pos.code == pri).map(|pos| pos.label.clone()).unwrap_or_default(),
                    "standard_level": standard_level_map.iter().find(|level| level.code == pri).map(|level| level.label.clone()).unwrap_or_default(),
                })
            )
        } else {
            None
        };
        let raw_secondary = if let Some(sec) = secondary_code.clone() {
            let sec_arr = sec.split(";").map(
                |s| json!({
                    "name": position_map.iter().find(|pos| pos.code == s).map(|pos| pos.label.clone()).unwrap_or_default(),
                    "standard_level": standard_level_map.iter().find(|level| level.code == s).map(|level| level.label.clone()).unwrap_or_default(),
                })
            ).collect_vec();
            Some(sec_arr)
        } else {
            None
        };
        let mut ext = json!({
            "status": account_resp.status,
            "temporary":account_resp.temporary,
            "lock_status": account_resp.lock_status,
            "role_id": account_roles,
            "role": raw_roles_map,
            "dept_id": account_resp_dept_id,
            "dept": account_resp_dept_map,
            "sub_deploy_ids": sub_deploy_ids,
            "auth_sub_deploy_ids": auth_sub_deploy_ids,
            "project_id": account_app_ids,
            "app": raw_account_apps_map,
            "create_time": account_resp.create_time.to_rfc3339(),
            "certs":account_resp.certs,
            "icon":account_resp.icon,
            "logout_msg":logout_msg,
            "disabled":account_resp.disabled,
            "logout_time":account_resp.logout_time,
            "logout_type":account_resp.logout_type,
            "labor_type":account_resp.labor_type,
            "primary": raw_primary,
            "primary_code": primary_code,
            "secondary": raw_secondary,
            "secondary_code": secondary_code,
            "app_set": app_set,
            "scope_level":account_resp.scope_level
        });
        //add or modify search
        if *is_modify {
            let modify_req = SearchItemModifyReq {
                kind: Some(funs.conf::<IamConfig>().spi.search_account_tag.clone()),
                title: Some(account_resp.name.clone()),
                name: Some(account_resp.name.clone()),
                content: Some(format!("{},{:?}", account_resp.name, account_certs,)),
                owner: Some(account_resp.owner),
                own_paths: if !account_resp.own_paths.is_empty() {
                    Some(account_resp.own_paths.clone())
                } else {
                    None
                },
                create_time: Some(account_resp.create_time),
                update_time: Some(account_resp.update_time),
                ext: Some(ext),
                ext_override: Some(true),
                visit_keys: Some(SearchItemVisitKeysReq {
                    accounts: None,
                    apps: Some(account_app_ids),
                    tenants: Some([account_resp.own_paths].to_vec()),
                    roles: Some(account_roles),
                    groups: Some(account_resp_dept_id),
                }),
                kv_disable: Some(account_resp.disabled),
            };
            SpiSearchClient::modify_item_and_name(&tag, &key, &modify_req, funs, ctx).await?;
        } else {
            let add_req = SearchItemAddReq {
                tag,
                kind: funs.conf::<IamConfig>().spi.search_account_tag.clone(),
                key: TrimString(key),
                title: account_resp.name.clone(),
                content: format!("{},{:?}", account_resp.name, account_certs,),
                data_source: None,
                owner: Some(account_resp.owner),
                own_paths: if !account_resp.own_paths.is_empty() {
                    Some(account_resp.own_paths.clone())
                } else {
                    None
                },
                create_time: Some(account_resp.create_time),
                update_time: Some(account_resp.update_time),
                ext: Some(ext),
                visit_keys: Some(SearchItemVisitKeysReq {
                    accounts: None,
                    apps: Some(account_app_ids),
                    tenants: Some([account_resp.own_paths].to_vec()),
                    roles: Some(account_roles),
                    groups: Some(account_resp_dept_id),
                }),
                kv_disable: Some(account_resp.disabled),
            };
            SpiSearchClient::add_item_and_name(&add_req, Some(account_resp.name), funs, ctx).await?
        }
        Ok(())
    }

    // account 全局搜索删除埋点方法
    pub async fn delete_account_search(account_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let tag = funs.conf::<IamConfig>().spi.search_account_tag.clone();
        SpiSearchClient::delete_item_and_name(&tag, account_id, funs, ctx).await?;
        Ok(())
    }
}
