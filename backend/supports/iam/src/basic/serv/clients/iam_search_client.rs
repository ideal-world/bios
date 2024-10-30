use std::collections::HashSet;

use bios_basic::rbum::{dto::rbum_filer_dto::RbumBasicFilterReq, serv::rbum_item_serv::RbumItemCrudOperation};
use bios_sdk_invoke::{
    clients::spi_search_client::SpiSearchClient,
    dto::search_item_dto::{SearchItemAddReq, SearchItemModifyReq, SearchItemVisitKeysReq},
};
use itertools::Itertools;

use tardis::{
    basic::{dto::TardisContext, field::TrimString, result::TardisResult},
    serde_json::json,
    tokio, TardisFunsInst,
};

use crate::{
    basic::{
        dto::{
            iam_account_dto::IamAccountDetailAggResp,
            iam_filer_dto::{IamAccountFilterReq, IamTenantFilterReq},
        },
        serv::{iam_account_serv::IamAccountServ, iam_role_serv::IamRoleServ, iam_set_serv::IamSetServ, iam_tenant_serv::IamTenantServ},
    },
    iam_config::IamConfig,
    iam_constants,
    iam_enumeration::IamSetKind,
};
pub struct IamSearchClient;

impl IamSearchClient {
    pub async fn async_add_or_modify_account_search(account_id: &str, is_modify: Box<bool>, logout_msg: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
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
            funs,
            &mock_ctx,
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
        let account_app_ids: Vec<String> = account_resp.apps.iter().map(|a| a.app_id.clone()).collect();
        let mut account_resp_dept_id = vec![];
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
        }

        let tag = funs.conf::<IamConfig>().spi.search_account_tag.clone();
        let key = account_id.to_string();
        let raw_roles = IamAccountServ::find_simple_rel_roles(&account_resp.id, true, Some(true), None, funs, ctx).await?;
        let mut roles_set = HashSet::new();
        for role in raw_roles {
            if !IamRoleServ::is_disabled(&role.rel_id, funs).await? {
                roles_set.insert(role.rel_id);
            }
        }
        let account_roles = roles_set.into_iter().collect_vec();
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
                ext: Some(json!({
                    "status": account_resp.status,
                    "temporary":account_resp.temporary,
                    "lock_status": account_resp.lock_status,
                    "role_id": account_roles,
                    "dept_id": account_resp_dept_id,
                    "project_id": account_app_ids,
                    "create_time": account_resp.create_time.to_rfc3339(),
                    "certs":account_resp.certs,
                    "icon":account_resp.icon,
                    "logout_msg":logout_msg,
                    "disabled":account_resp.disabled,
                    "logout_time":account_resp.logout_time,
                    "logout_type":account_resp.logout_type,
                    "labor_type":account_resp.labor_type,
                    "scope_level":account_resp.scope_level
                })),
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
                owner: Some(account_resp.owner),
                own_paths: if !account_resp.own_paths.is_empty() {
                    Some(account_resp.own_paths.clone())
                } else {
                    None
                },
                create_time: Some(account_resp.create_time),
                update_time: Some(account_resp.update_time),
                ext: Some(json!({
                    "status": account_resp.status,
                    "temporary":account_resp.temporary,
                    "lock_status": account_resp.lock_status,
                    "role_id": account_roles,
                    "dept_id": account_resp_dept_id,
                    "project_id": account_app_ids,
                    "create_time": account_resp.create_time.to_rfc3339(),
                    "certs":account_resp.certs,
                    "icon":account_resp.icon,
                    "logout_msg":logout_msg,
                    "disabled":account_resp.disabled,
                    "logout_time":account_resp.logout_time,
                    "logout_type":account_resp.logout_type,
                    "labor_type":account_resp.labor_type,
                    "scope_level":account_resp.scope_level
                })),
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
