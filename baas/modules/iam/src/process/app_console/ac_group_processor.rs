/*
 * Copyright 2021. gudaoxuri
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use actix_web::{delete, get, post, put, HttpRequest};
use sea_query::{Alias, Cond, Expr, Func, JoinType, Order, Query};
use sqlx::Connection;
use strum::IntoEnumIterator;

use bios::basic::error::BIOSError;
use bios::db::reldb_client::SqlBuilderProcess;
use bios::web::basic_processor::get_ident_account_info;
use bios::web::resp_handler::{BIOSResp, BIOSRespHelper};
use bios::web::validate::json::Json;
use bios::web::validate::query::Query as VQuery;
use bios::BIOSFuns;

use crate::domain::auth_domain::{IamAccountGroup, IamAuthPolicyObject, IamGroup, IamGroupNode};
use crate::domain::ident_domain::IamAccount;
use crate::process::app_console::ac_group_dto::{
    GroupAddReq, GroupDetailResp, GroupModifyReq, GroupNodeAddReq, GroupNodeDetailResp, GroupNodeModifyReq, GroupNodeOverviewResp, GroupQueryReq,
};
use crate::process::basic_dto::AuthObjectKind;

#[post("/console/app/group")]
pub async fn add_group(group_add_req: Json<GroupAddReq>, req: HttpRequest) -> BIOSResp {
    let ident_info = get_ident_account_info(&req)?;
    let id = bios::basic::field::uuid();

    if BIOSFuns::reldb()
        .exists(
            &Query::select()
                .columns(vec![IamGroup::Id])
                .from(IamGroup::Table)
                .and_where(Expr::col(IamGroup::Code).eq(group_add_req.code.as_str().to_lowercase()))
                .and_where(Expr::col(IamGroup::RelAppId).eq(ident_info.app_id.as_str()))
                .done(),
            None,
        )
        .await?
    {
        return BIOSRespHelper::bus_error(BIOSError::Conflict("Group [code] already exists".to_string()));
    }

    BIOSFuns::reldb()
        .exec(
            &Query::insert()
                .into_table(IamGroup::Table)
                .columns(vec![
                    IamGroup::Id,
                    IamGroup::CreateUser,
                    IamGroup::UpdateUser,
                    IamGroup::Code,
                    IamGroup::Name,
                    IamGroup::Kind,
                    IamGroup::Icon,
                    IamGroup::Sort,
                    IamGroup::RelGroupId,
                    IamGroup::RelGroupNodeId,
                    IamGroup::RelAppId,
                    IamGroup::RelTenantId,
                    IamGroup::ExposeKind,
                ])
                .values_panic(vec![
                    id.as_str().into(),
                    ident_info.account_id.as_str().into(),
                    ident_info.account_id.as_str().into(),
                    group_add_req.code.as_str().to_lowercase().into(),
                    group_add_req.name.as_str().into(),
                    group_add_req.kind.to_string().to_lowercase().into(),
                    group_add_req.icon.as_deref().unwrap_or(&"").into(),
                    group_add_req.sort.into(),
                    group_add_req.rel_group_id.as_deref().unwrap_or(&"").into(),
                    group_add_req.rel_group_node_id.as_deref().unwrap_or(&"").into(),
                    ident_info.app_id.as_str().into(),
                    ident_info.tenant_id.as_str().into(),
                    group_add_req.expose_kind.as_ref().unwrap_or(&crate::process::basic_dto::ExposeKind::App).to_string().to_lowercase().into(),
                ])
                .done(),
            None,
        )
        .await?;
    BIOSRespHelper::ok(id)
}

#[put("/console/app/group/{id}")]
pub async fn modify_group(group_modify_req: Json<GroupModifyReq>, req: HttpRequest) -> BIOSResp {
    let ident_info = get_ident_account_info(&req)?;
    let id: String = req.match_info().get("id").unwrap().parse()?;

    if !BIOSFuns::reldb()
        .exists(
            &Query::select()
                .columns(vec![IamGroup::Id])
                .from(IamGroup::Table)
                .and_where(Expr::col(IamGroup::Id).eq(id.as_str()))
                .and_where(Expr::col(IamGroup::RelAppId).eq(ident_info.app_id.as_str()))
                .done(),
            None,
        )
        .await?
    {
        return BIOSRespHelper::bus_error(BIOSError::NotFound("IamGroup not exists".to_string()));
    }
    let mut values = Vec::new();
    if let Some(name) = &group_modify_req.name {
        values.push((IamGroup::Name, name.to_string().into()));
    }
    if let Some(kind) = &group_modify_req.kind {
        values.push((IamGroup::Kind, kind.to_string().to_lowercase().into()));
    }
    if let Some(sort) = group_modify_req.sort {
        values.push((IamGroup::Sort, sort.into()));
    }
    if let Some(icon) = &group_modify_req.icon {
        values.push((IamGroup::Icon, icon.to_string().into()));
    }
    if let Some(rel_group_id) = &group_modify_req.rel_group_id {
        values.push((IamGroup::RelGroupId, rel_group_id.to_string().into()));
    }
    if let Some(rel_group_node_id) = &group_modify_req.rel_group_node_id {
        values.push((IamGroup::RelGroupNodeId, rel_group_node_id.to_string().into()));
    }
    if let Some(expose_kind) = &group_modify_req.expose_kind {
        values.push((IamGroup::ExposeKind, expose_kind.to_string().to_lowercase().into()));
    }
    values.push((IamGroup::UpdateUser, ident_info.account_id.as_str().into()));
    BIOSFuns::reldb()
        .exec(
            &Query::update()
                .table(IamGroup::Table)
                .values(values)
                .and_where(Expr::col(IamGroup::Id).eq(id.as_str()))
                .and_where(Expr::col(IamGroup::RelAppId).eq(ident_info.app_id.as_str()))
                .done(),
            None,
        )
        .await?;
    BIOSRespHelper::ok("")
}

#[get("/console/app/group")]
pub async fn list_group(query: VQuery<GroupQueryReq>, req: HttpRequest) -> BIOSResp {
    let ident_info = get_ident_account_info(&req)?;

    let create_user_table = Alias::new("create");
    let update_user_table = Alias::new("update");
    let sql_builder = Query::select()
        .columns(vec![
            (IamGroup::Table, IamGroup::Id),
            (IamGroup::Table, IamGroup::CreateTime),
            (IamGroup::Table, IamGroup::UpdateTime),
            (IamGroup::Table, IamGroup::Code),
            (IamGroup::Table, IamGroup::Name),
            (IamGroup::Table, IamGroup::Kind),
            (IamGroup::Table, IamGroup::Icon),
            (IamGroup::Table, IamGroup::Sort),
            (IamGroup::Table, IamGroup::RelGroupId),
            (IamGroup::Table, IamGroup::RelGroupNodeId),
            (IamGroup::Table, IamGroup::RelAppId),
            (IamGroup::Table, IamGroup::RelTenantId),
            (IamGroup::Table, IamGroup::ExposeKind),
        ])
        .expr_as(Expr::tbl(create_user_table.clone(), IamAccount::Name), Alias::new("create_user"))
        .expr_as(Expr::tbl(update_user_table.clone(), IamAccount::Name), Alias::new("update_user"))
        .from(IamGroup::Table)
        .join_as(
            JoinType::InnerJoin,
            IamAccount::Table,
            create_user_table.clone(),
            Expr::tbl(create_user_table, IamAccount::Id).equals(IamGroup::Table, IamGroup::CreateUser),
        )
        .join_as(
            JoinType::InnerJoin,
            IamAccount::Table,
            update_user_table.clone(),
            Expr::tbl(update_user_table, IamAccount::Id).equals(IamGroup::Table, IamGroup::UpdateUser),
        )
        .and_where_option(if let Some(code) = &query.code {
            Some(Expr::tbl(IamGroup::Table, IamGroup::Code).like(format!("%{}%", code).as_str()))
        } else {
            None
        })
        .and_where_option(if let Some(name) = &query.name {
            Some(Expr::tbl(IamGroup::Table, IamGroup::Name).like(format!("%{}%", name).as_str()))
        } else {
            None
        })
        .conditions(
            !query.expose,
            |x| {
                x.and_where(Expr::tbl(IamGroup::Table, IamGroup::RelAppId).eq(ident_info.app_id.as_str()));
            },
            |x| {
                x.cond_where(
                    Cond::any().add(Expr::tbl(IamGroup::Table, IamGroup::ExposeKind).eq(crate::process::basic_dto::ExposeKind::Global.to_string().to_lowercase())).add(
                        Cond::all()
                            .add(Expr::tbl(IamGroup::Table, IamGroup::RelTenantId).eq(ident_info.tenant_id.as_str()))
                            .add(Expr::tbl(IamGroup::Table, IamGroup::ExposeKind).eq(crate::process::basic_dto::ExposeKind::Tenant.to_string().to_lowercase())),
                    ),
                );
            },
        )
        .and_where(Expr::tbl(IamGroup::Table, IamGroup::RelAppId).eq(ident_info.app_id.as_str()))
        .order_by(IamGroup::UpdateTime, Order::Desc)
        .done();
    let items = BIOSFuns::reldb().pagination::<GroupDetailResp>(&sql_builder, query.page_number, query.page_size, None).await?;
    BIOSRespHelper::ok(items)
}

#[delete("/console/app/group/{id}")]
pub async fn delete_group(req: HttpRequest) -> BIOSResp {
    let ident_info = get_ident_account_info(&req)?;
    let id: String = req.match_info().get("id").unwrap().parse()?;

    if !BIOSFuns::reldb()
        .exists(
            &Query::select()
                .columns(vec![IamGroup::Id])
                .from(IamGroup::Table)
                .and_where(Expr::col(IamGroup::Id).eq(id.as_str()))
                .and_where(Expr::col(IamGroup::RelAppId).eq(ident_info.app_id.as_str()))
                .done(),
            None,
        )
        .await?
    {
        return BIOSRespHelper::bus_error(BIOSError::NotFound("IamGroup not exists".to_string()));
    }
    if BIOSFuns::reldb()
        .exists(
            &Query::select().columns(vec![IamGroupNode::Id]).from(IamGroupNode::Table).and_where(Expr::col(IamGroupNode::RelGroupId).eq(id.as_str())).done(),
            None,
        )
        .await?
    {
        return BIOSRespHelper::bus_error(BIOSError::Conflict("Please delete the associated [group_node] data first".to_owned()));
    }

    let mut conn = BIOSFuns::reldb().conn().await;
    let mut tx = conn.begin().await?;

    let sql_builder = Query::select()
        .columns(IamGroup::iter().filter(|i| *i != IamGroup::Table))
        .from(IamGroup::Table)
        .and_where(Expr::col(IamGroup::Id).eq(id.as_str()))
        .and_where(Expr::col(IamGroup::RelAppId).eq(ident_info.app_id.as_str()))
        .done();
    BIOSFuns::reldb().soft_del(IamGroup::Table, IamGroup::Id, &ident_info.account_id, &sql_builder, &mut tx).await?;
    tx.commit().await?;
    BIOSRespHelper::ok("")
}

// ------------------------------------

#[post("/console/app/group/{group_id}/node")]
pub async fn add_group_node(group_node_add_req: Json<GroupNodeAddReq>, req: HttpRequest) -> BIOSResp {
    let ident_info = get_ident_account_info(&req)?;
    let group_id: String = req.match_info().get("group_id").unwrap().parse()?;
    let id = bios::basic::field::uuid();

    if !BIOSFuns::reldb()
        .exists(
            &Query::select()
                .columns(vec![IamGroup::Id])
                .from(IamGroup::Table)
                .and_where(Expr::col(IamGroup::Id).eq(group_id.as_str()))
                .and_where(Expr::col(IamGroup::RelAppId).eq(ident_info.app_id.as_str()))
                .done(),
            None,
        )
        .await?
    {
        return BIOSRespHelper::bus_error(BIOSError::NotFound("GroupNode [rel_group_id] not exists".to_string()));
    }

    let last_group_node = BIOSFuns::reldb()
        .fetch_optional_json(
            &Query::select()
                .column(IamGroupNode::Code)
                .from(IamGroupNode::Table)
                .and_where(Expr::col(IamGroupNode::RelGroupId).eq(group_id.as_str()))
                .and_where(Expr::col(IamGroupNode::Code).like(format!("{}%", group_node_add_req.parent_code.as_str()).as_str()))
                .and_where(
                    Expr::expr(Func::char_length(Expr::col(IamGroupNode::Code))).eq(if group_node_add_req.parent_code.is_empty() {
                        4
                    } else {
                        group_node_add_req.parent_code.len() as i32 + 5
                    }),
                )
                .order_by(IamGroupNode::UpdateTime, Order::Desc)
                .done(),
            None,
        )
        .await?;

    let code = match last_group_node {
        Some(node) => {
            if group_node_add_req.parent_code.is_empty() {
                bios::basic::field::incr_by_base36(node["code"].as_str().unwrap()).expect("Group node code exceeds maximum limit")
            } else {
                let code = node["code"].as_str().unwrap().to_string();
                let last_split_idx = code.as_str().rfind(".").unwrap();
                let parent_code = &code.as_str()[..last_split_idx];
                let current_code = &code.as_str()[last_split_idx + 1..];
                format!(
                    "{}.{}",
                    parent_code,
                    bios::basic::field::incr_by_base36(current_code).expect("Group node code exceeds maximum limit")
                )
            }
        }
        None => {
            // TODO
            if group_node_add_req.parent_code.is_empty() {
                "aaaa".to_string()
            } else {
                format!("{}.{}", group_node_add_req.parent_code, "aaaa")
            }
        }
    };
    BIOSFuns::reldb()
        .exec(
            &Query::insert()
                .into_table(IamGroupNode::Table)
                .columns(vec![
                    IamGroupNode::Id,
                    IamGroupNode::CreateUser,
                    IamGroupNode::UpdateUser,
                    IamGroupNode::Code,
                    IamGroupNode::BusCode,
                    IamGroupNode::Name,
                    IamGroupNode::Sort,
                    IamGroupNode::Parameters,
                    IamGroupNode::RelGroupId,
                ])
                .values_panic(vec![
                    id.as_str().into(),
                    ident_info.account_id.as_str().into(),
                    ident_info.account_id.as_str().into(),
                    code.as_str().into(),
                    group_node_add_req.bus_code.as_deref().unwrap_or(&"").into(),
                    group_node_add_req.name.as_str().into(),
                    group_node_add_req.sort.into(),
                    group_node_add_req.parameters.as_deref().unwrap_or(&"").into(),
                    group_id.into(),
                ])
                .done(),
            None,
        )
        .await?;
    BIOSRespHelper::ok(GroupNodeOverviewResp { id, code })
}

#[put("/console/app/group/{group_id}/node/{id}")]
pub async fn modify_group_node(group_node_modify_req: Json<GroupNodeModifyReq>, req: HttpRequest) -> BIOSResp {
    let ident_info = get_ident_account_info(&req)?;
    let group_id: String = req.match_info().get("group_id").unwrap().parse()?;
    let id: String = req.match_info().get("id").unwrap().parse()?;

    if !BIOSFuns::reldb()
        .exists(
            &Query::select()
                .columns(vec![IamGroup::Id])
                .from(IamGroup::Table)
                .and_where(Expr::col(IamGroup::Id).eq(group_id.as_str()))
                .and_where(Expr::col(IamGroup::RelAppId).eq(ident_info.app_id.as_str()))
                .done(),
            None,
        )
        .await?
    {
        return BIOSRespHelper::bus_error(BIOSError::NotFound("Group not exists".to_string()));
    }
    if !BIOSFuns::reldb()
        .exists(
            &Query::select()
                .columns(vec![IamGroupNode::Id])
                .from(IamGroupNode::Table)
                .and_where(Expr::col(IamGroupNode::Id).eq(id.as_str()))
                .and_where(Expr::col(IamGroupNode::RelGroupId).eq(group_id.as_str()))
                .done(),
            None,
        )
        .await?
    {
        return BIOSRespHelper::bus_error(BIOSError::NotFound("GroupNode not exists".to_string()));
    }

    let mut values = Vec::new();
    if let Some(bus_code) = &group_node_modify_req.bus_code {
        values.push((IamGroupNode::BusCode, bus_code.to_string().into()));
    }
    if let Some(name) = &group_node_modify_req.name {
        values.push((IamGroupNode::Name, name.to_string().into()));
    }
    if let Some(sort) = group_node_modify_req.sort {
        values.push((IamGroupNode::Sort, sort.to_string().into()));
    }
    if let Some(parameters) = &group_node_modify_req.parameters {
        values.push((IamGroupNode::Parameters, parameters.to_string().into()));
    }
    BIOSFuns::reldb()
        .exec(
            &Query::update()
                .table(IamGroupNode::Table)
                .values(values)
                .and_where(Expr::col(IamGroupNode::Id).eq(id.as_str()))
                .and_where(Expr::col(IamGroupNode::RelGroupId).eq(group_id))
                .done(),
            None,
        )
        .await?;
    BIOSRespHelper::ok("")
}

#[get("/console/app/group/{group_id}/node")]
pub async fn list_group_node(req: HttpRequest) -> BIOSResp {
    let ident_info = get_ident_account_info(&req)?;
    let group_id: String = req.match_info().get("group_id").unwrap().parse()?;

    if !BIOSFuns::reldb()
        .exists(
            &Query::select()
                .columns(vec![IamGroup::Id])
                .from(IamGroup::Table)
                .and_where(Expr::col(IamGroup::Id).eq(group_id.as_str()))
                .and_where(Expr::col(IamGroup::RelAppId).eq(ident_info.app_id.as_str()))
                .done(),
            None,
        )
        .await?
    {
        return BIOSRespHelper::bus_error(BIOSError::NotFound("GroupNode [rel_group_id] not exists".to_string()));
    }

    let create_user_table = Alias::new("create");
    let update_user_table = Alias::new("update");
    let sql_builder = Query::select()
        .columns(vec![
            (IamGroupNode::Table, IamGroupNode::Id),
            (IamGroupNode::Table, IamGroupNode::CreateTime),
            (IamGroupNode::Table, IamGroupNode::UpdateTime),
            (IamGroupNode::Table, IamGroupNode::Code),
            (IamGroupNode::Table, IamGroupNode::BusCode),
            (IamGroupNode::Table, IamGroupNode::Name),
            (IamGroupNode::Table, IamGroupNode::Sort),
            (IamGroupNode::Table, IamGroupNode::Parameters),
            (IamGroupNode::Table, IamGroupNode::RelGroupId),
        ])
        .expr_as(Expr::tbl(create_user_table.clone(), IamAccount::Name), Alias::new("create_user"))
        .expr_as(Expr::tbl(update_user_table.clone(), IamAccount::Name), Alias::new("update_user"))
        .from(IamGroupNode::Table)
        .join_as(
            JoinType::InnerJoin,
            IamAccount::Table,
            create_user_table.clone(),
            Expr::tbl(create_user_table, IamAccount::Id).equals(IamGroupNode::Table, IamGroupNode::CreateUser),
        )
        .join_as(
            JoinType::InnerJoin,
            IamAccount::Table,
            update_user_table.clone(),
            Expr::tbl(update_user_table, IamAccount::Id).equals(IamGroupNode::Table, IamGroupNode::UpdateUser),
        )
        .and_where(Expr::tbl(IamGroupNode::Table, IamGroupNode::RelGroupId).eq(group_id))
        .order_by(IamGroupNode::UpdateTime, Order::Desc)
        .done();
    let items = BIOSFuns::reldb().fetch_all::<GroupNodeDetailResp>(&sql_builder, None).await?;
    BIOSRespHelper::ok(items)
}

#[delete("/console/app/group/{group_id}/node/{id}")]
pub async fn delete_group_node(req: HttpRequest) -> BIOSResp {
    let ident_info = get_ident_account_info(&req)?;
    let group_id: String = req.match_info().get("group_id").unwrap().parse()?;
    let id: String = req.match_info().get("id").unwrap().parse()?;

    if !BIOSFuns::reldb()
        .exists(
            &Query::select()
                .columns(vec![IamGroup::Id])
                .from(IamGroup::Table)
                .and_where(Expr::col(IamGroup::Id).eq(group_id.as_str()))
                .and_where(Expr::col(IamGroup::RelAppId).eq(ident_info.app_id.as_str()))
                .done(),
            None,
        )
        .await?
    {
        return BIOSRespHelper::bus_error(BIOSError::NotFound("Group not exists".to_string()));
    }
    if !BIOSFuns::reldb()
        .exists(
            &Query::select()
                .columns(vec![IamGroupNode::Id])
                .from(IamGroupNode::Table)
                .and_where(Expr::col(IamGroupNode::Id).eq(id.as_str()))
                .and_where(Expr::col(IamGroupNode::RelGroupId).eq(group_id.as_str()))
                .done(),
            None,
        )
        .await?
    {
        return BIOSRespHelper::bus_error(BIOSError::NotFound("GroupNode not exists".to_string()));
    }

    let result = BIOSFuns::reldb()
        .fetch_one_json(
            &Query::select().column(IamGroupNode::Code).from(IamGroupNode::Table).and_where(Expr::col(IamGroupNode::Id).eq(id.as_str())).done(),
            None,
        )
        .await?;
    let code = result["code"].as_str().unwrap();

    if BIOSFuns::reldb()
        .exists(
            &Query::select()
                .columns(vec![IamAuthPolicyObject::Id])
                .from(IamAuthPolicyObject::Table)
                .and_where(Expr::col(IamAuthPolicyObject::ObjectKind).eq(AuthObjectKind::GroupNode.to_string().to_lowercase()))
                .and_where(Expr::col(IamAuthPolicyObject::ObjectId).like(format!("{}-{}", id.as_str(), code).as_str()))
                .done(),
            None,
        )
        .await?
    {
        return BIOSRespHelper::bus_error(BIOSError::Conflict("Please delete the associated [auth_policy] data first".to_owned()));
    }
    if BIOSFuns::reldb()
        .exists(
            &Query::select().columns(vec![IamAccountGroup::Id]).from(IamAccountGroup::Table).and_where(Expr::col(IamAccountGroup::RelGroupNodeId).eq(id.as_str())).done(),
            None,
        )
        .await?
    {
        return BIOSRespHelper::bus_error(BIOSError::Conflict("Please delete the associated [account_group] data first".to_owned()));
    }

    let mut conn = BIOSFuns::reldb().conn().await;
    let mut tx = conn.begin().await?;

    let sql_builder = Query::select()
        .columns(IamGroupNode::iter().filter(|i| *i != IamGroupNode::Table))
        .from(IamGroupNode::Table)
        .and_where(Expr::col(IamGroupNode::Id).eq(id.as_str()))
        .and_where(Expr::col(IamGroupNode::RelGroupId).eq(group_id))
        .done();
    BIOSFuns::reldb().soft_del(IamGroupNode::Table, IamGroupNode::Id, &ident_info.account_id, &sql_builder, &mut tx).await?;
    tx.commit().await?;
    BIOSRespHelper::ok("")
}
