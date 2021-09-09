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
use chrono::Utc;
use itertools::Itertools;
use sea_query::{Alias, Cond, Expr, JoinType, Order, Query};
use sqlx::{Connection, MySql, Transaction};
use strum::IntoEnumIterator;

use bios::basic::error::{BIOSError, BIOSResult};
use bios::db::reldb_client::SqlBuilderProcess;
use bios::web::basic_processor::get_ident_account_info;
use bios::web::resp_handler::{BIOSResp, BIOSRespHelper};
use bios::web::validate::json::Json;
use bios::web::validate::query::Query as VQuery;
use bios::BIOSFuns;

use crate::domain::auth_domain::{IamAuthPolicy, IamAuthPolicySubject, IamGroup, IamGroupNode, IamResource, IamResourceSubject, IamRole};
use crate::domain::ident_domain::{IamAccount, IamAccountApp};
use crate::iam_config::WorkSpaceConfig;
use crate::process::app_console::ac_auth_policy_dto::{
    AuthPolicyAddReq, AuthPolicyDetailResp, AuthPolicyModifyReq, AuthPolicyQueryReq, AuthPolicySubjectAddReq, AuthPolicySubjectDetailResp,
};
use crate::process::basic_dto::AuthSubjectKind;

#[post("/console/app/auth-policy")]
pub async fn add_auth_policy(auth_policy_add_req: Json<AuthPolicyAddReq>, req: HttpRequest) -> BIOSResp {
    let ident_info = get_ident_account_info(&req)?;
    let id = bios::basic::field::uuid();

    if !BIOSFuns::reldb()
        .exists(
            &Query::select()
                .columns(vec![IamResource::Id])
                .from(IamResource::Table)
                .cond_where(
                    Cond::any()
                        .add(Expr::tbl(IamResource::Table, IamResource::ExposeKind).eq(crate::process::basic_dto::ExposeKind::Global.to_string().to_lowercase()))
                        .add(
                            Cond::all()
                                .add(Expr::tbl(IamResource::Table, IamResource::RelTenantId).eq(ident_info.tenant_id.clone()))
                                .add(Expr::tbl(IamResource::Table, IamResource::ExposeKind).eq(crate::process::basic_dto::ExposeKind::Tenant.to_string().to_lowercase())),
                        ),
                )
                .and_where(Expr::col(IamResource::Id).eq(auth_policy_add_req.rel_resource_id.clone()))
                .done(),
            None,
        )
        .await?
    {
        return BIOSRespHelper::bus_error(BIOSError::NotFound("AuthPolicy [rel_resource_id] not exists".to_string()));
    }

    if BIOSFuns::reldb()
        .exists(
            &Query::select()
                .columns(vec![IamAuthPolicy::Id])
                .from(IamAuthPolicy::Table)
                .and_where(Expr::col(IamAuthPolicy::ActionKind).eq(auth_policy_add_req.action_kind.to_string().to_lowercase()))
                .and_where(Expr::col(IamAuthPolicy::RelResourceId).eq(auth_policy_add_req.rel_resource_id.clone()))
                .and_where(Expr::col(IamAuthPolicy::ValidStartTime).lte(auth_policy_add_req.valid_start_time.clone()))
                .and_where(Expr::col(IamAuthPolicy::ValidEndTime).gte(auth_policy_add_req.valid_end_time.clone()))
                .and_where(Expr::col(IamAuthPolicy::ResultKind).eq(auth_policy_add_req.result_kind.to_string().to_lowercase()))
                .done(),
            None,
        )
        .await?
    {
        return BIOSRespHelper::bus_error(BIOSError::Conflict("AuthPolicy already exists".to_string()));
    }

    let mut conn = BIOSFuns::reldb().conn().await;
    let mut tx = conn.begin().await?;

    BIOSFuns::reldb()
        .exec(
            &Query::insert()
                .into_table(IamAuthPolicy::Table)
                .columns(vec![
                    IamAuthPolicy::Id,
                    IamAuthPolicy::CreateUser,
                    IamAuthPolicy::UpdateUser,
                    IamAuthPolicy::Name,
                    IamAuthPolicy::ValidStartTime,
                    IamAuthPolicy::ValidEndTime,
                    IamAuthPolicy::ActionKind,
                    IamAuthPolicy::RelResourceId,
                    IamAuthPolicy::ResultKind,
                    IamAuthPolicy::RelAppId,
                    IamAuthPolicy::RelTenantId,
                ])
                .values_panic(vec![
                    id.clone().into(),
                    ident_info.account_id.clone().into(),
                    ident_info.account_id.clone().into(),
                    auth_policy_add_req.name.clone().into(),
                    auth_policy_add_req.valid_start_time.into(),
                    auth_policy_add_req.valid_end_time.into(),
                    auth_policy_add_req.action_kind.to_string().to_lowercase().into(),
                    auth_policy_add_req.rel_resource_id.clone().into(),
                    auth_policy_add_req.result_kind.to_string().to_lowercase().into(),
                    ident_info.app_id.clone().into(),
                    ident_info.tenant_id.clone().into(),
                ])
                .done(),
            Some(&mut tx),
        )
        .await?;
    rebuild_cache(&id, &mut tx).await?;
    tx.commit().await?;
    BIOSRespHelper::ok(id)
}

#[put("/console/app/auth-policy/{id}")]
pub async fn modify_auth_policy(auth_policy_modify_req: Json<AuthPolicyModifyReq>, req: HttpRequest) -> BIOSResp {
    let ident_info = get_ident_account_info(&req)?;
    let id: String = req.match_info().get("id").unwrap().parse()?;

    if auth_policy_modify_req.valid_start_time.is_some() && auth_policy_modify_req.valid_end_time.is_none()
        || auth_policy_modify_req.valid_start_time.is_none() && auth_policy_modify_req.valid_end_time.is_some()
    {
        return BIOSRespHelper::bus_error(BIOSError::BadRequest(
            "AuthPolicy [valid_start_time] and [valid_end_time] must exist at the same time".to_string(),
        ));
    }
    if !BIOSFuns::reldb()
        .exists(
            &Query::select()
                .columns(vec![IamAuthPolicy::Id])
                .from(IamAuthPolicy::Table)
                .and_where(Expr::col(IamAuthPolicy::Id).eq(id.clone()))
                .and_where(Expr::col(IamAuthPolicy::RelAppId).eq(ident_info.app_id.clone().to_lowercase()))
                .done(),
            None,
        )
        .await?
    {
        return BIOSRespHelper::bus_error(BIOSError::NotFound("AuthPolicy not exists".to_string()));
    }
    if auth_policy_modify_req.valid_start_time.is_some() && auth_policy_modify_req.valid_end_time.is_some() {
        if BIOSFuns::reldb()
            .exists(
                &Query::select()
                    .columns(vec![IamAuthPolicy::Id])
                    .from(IamAuthPolicy::Table)
                    .and_where(Expr::col(IamAuthPolicy::Id).ne(id.clone()))
                    .and_where(Expr::col(IamAuthPolicy::ValidStartTime).lte(auth_policy_modify_req.valid_start_time.unwrap()))
                    .and_where(Expr::col(IamAuthPolicy::ValidEndTime).gte(auth_policy_modify_req.valid_end_time.unwrap()))
                    .done(),
                None,
            )
            .await?
        {
            return BIOSRespHelper::bus_error(BIOSError::Conflict("AuthPolicy already exists".to_string()));
        }
    }

    let mut conn = BIOSFuns::reldb().conn().await;
    let mut tx = conn.begin().await?;

    let mut values = Vec::new();
    if let Some(name) = &auth_policy_modify_req.name {
        values.push((IamAuthPolicy::Name, name.to_string().into()));
    }
    if let Some(valid_start_time) = auth_policy_modify_req.valid_start_time {
        values.push((IamAuthPolicy::ValidStartTime, valid_start_time.into()));
    }
    if let Some(valid_end_time) = auth_policy_modify_req.valid_end_time {
        values.push((IamAuthPolicy::ValidEndTime, valid_end_time.into()));
    }
    values.push((IamAuthPolicy::UpdateUser, ident_info.account_id.clone().into()));
    BIOSFuns::reldb()
        .exec(
            &Query::update()
                .table(IamAuthPolicy::Table)
                .values(values)
                .and_where(Expr::col(IamAuthPolicy::Id).eq(id.clone()))
                .and_where(Expr::col(IamAuthPolicy::RelAppId).eq(ident_info.app_id.clone()))
                .done(),
            Some(&mut tx),
        )
        .await?;
    rebuild_cache(&id, &mut tx).await?;
    tx.commit().await?;
    BIOSRespHelper::ok("")
}

#[get("/console/app/auth-policy")]
pub async fn list_auth_policy(query: VQuery<AuthPolicyQueryReq>, req: HttpRequest) -> BIOSResp {
    let ident_info = get_ident_account_info(&req)?;

    let create_user_table = Alias::new("create");
    let update_user_table = Alias::new("update");
    let sql_builder = Query::select()
        .columns(vec![
            (IamAuthPolicy::Table, IamAuthPolicy::Id),
            (IamAuthPolicy::Table, IamAuthPolicy::CreateTime),
            (IamAuthPolicy::Table, IamAuthPolicy::UpdateTime),
            (IamAuthPolicy::Table, IamAuthPolicy::Name),
            (IamAuthPolicy::Table, IamAuthPolicy::ActionKind),
            (IamAuthPolicy::Table, IamAuthPolicy::RelResourceId),
            (IamAuthPolicy::Table, IamAuthPolicy::ValidStartTime),
            (IamAuthPolicy::Table, IamAuthPolicy::ValidEndTime),
            (IamAuthPolicy::Table, IamAuthPolicy::ResultKind),
            (IamAuthPolicy::Table, IamAuthPolicy::RelAppId),
            (IamAuthPolicy::Table, IamAuthPolicy::RelTenantId),
        ])
        .expr_as(Expr::tbl(create_user_table.clone(), IamAccount::Name), Alias::new("create_user"))
        .expr_as(Expr::tbl(update_user_table.clone(), IamAccount::Name), Alias::new("update_user"))
        .from(IamAuthPolicy::Table)
        .join_as(
            JoinType::InnerJoin,
            IamAccount::Table,
            create_user_table.clone(),
            Expr::tbl(create_user_table, IamAccount::Id).equals(IamAuthPolicy::Table, IamAuthPolicy::CreateUser),
        )
        .join_as(
            JoinType::InnerJoin,
            IamAccount::Table,
            update_user_table.clone(),
            Expr::tbl(update_user_table, IamAccount::Id).equals(IamAuthPolicy::Table, IamAuthPolicy::UpdateUser),
        )
        .and_where_option(if let Some(name) = &query.name {
            Some(Expr::tbl(IamAuthPolicy::Table, IamAuthPolicy::Name).like(format!("%{}%", name).as_str()))
        } else {
            None
        })
        .and_where_option(if let Some(rel_resource_id) = &query.rel_resource_id {
            Some(Expr::tbl(IamAuthPolicy::Table, IamAuthPolicy::RelResourceId).eq(rel_resource_id.to_string()))
        } else {
            None
        })
        .and_where_option(if let Some(valid_start_time) = query.valid_start_time {
            Some(Expr::tbl(IamAuthPolicy::Table, IamAuthPolicy::ValidStartTime).gte(valid_start_time))
        } else {
            None
        })
        .and_where_option(if let Some(valid_end_time) = query.valid_end_time {
            Some(Expr::tbl(IamAuthPolicy::Table, IamAuthPolicy::ValidEndTime).lte(valid_end_time))
        } else {
            None
        })
        .and_where(Expr::tbl(IamAuthPolicy::Table, IamAuthPolicy::RelAppId).eq(ident_info.app_id.clone()))
        .order_by(IamAuthPolicy::UpdateTime, Order::Desc)
        .done();
    let items = BIOSFuns::reldb()
        .pagination::<AuthPolicyDetailResp>(&sql_builder, query.page_number, query.page_size, None)
        .await?;
    BIOSRespHelper::ok(items)
}

#[delete("/console/app/auth-policy/{id}")]
pub async fn delete_auth_policy(req: HttpRequest) -> BIOSResp {
    let ident_info = get_ident_account_info(&req)?;
    let id: String = req.match_info().get("id").unwrap().parse()?;

    if !BIOSFuns::reldb()
        .exists(
            &Query::select()
                .columns(vec![IamAuthPolicy::Id])
                .from(IamAuthPolicy::Table)
                .and_where(Expr::col(IamAuthPolicy::Id).eq(id.clone()))
                .and_where(Expr::col(IamAuthPolicy::RelAppId).eq(ident_info.app_id.clone().to_lowercase()))
                .done(),
            None,
        )
        .await?
    {
        return BIOSRespHelper::bus_error(BIOSError::NotFound("AuthPolicy not exists".to_string()));
    }
    if BIOSFuns::reldb()
        .exists(
            &Query::select()
                .columns(vec![IamAuthPolicySubject::Id])
                .from(IamAuthPolicySubject::Table)
                .and_where(Expr::col(IamAuthPolicySubject::RelAuthPolicyId).eq(id.clone()))
                .done(),
            None,
        )
        .await?
    {
        return BIOSRespHelper::bus_error(BIOSError::Conflict("Please delete the associated [auth_policy_subject] data first".to_owned()));
    }

    let mut conn = BIOSFuns::reldb().conn().await;
    let mut tx = conn.begin().await?;

    let sql_builder = Query::select()
        .columns(IamAuthPolicy::iter().filter(|i| *i != IamAuthPolicy::Table))
        .from(IamAuthPolicy::Table)
        .and_where(Expr::col(IamAuthPolicy::Id).eq(id.clone()))
        .and_where(Expr::col(IamAuthPolicy::RelAppId).eq(ident_info.app_id.clone()))
        .done();
    BIOSFuns::reldb()
        .soft_del(IamAuthPolicy::Table, IamAuthPolicy::Id, &ident_info.account_id, &sql_builder, &mut tx)
        .await?;
    delete_cache(&id, &mut tx).await?;
    tx.commit().await?;
    BIOSRespHelper::ok("")
}

// ------------------------------------

#[post("/console/app/auth-policy/{auth_policy_id}/subject")]
pub async fn add_auth_policy_subject(auth_policy_subject_add_req: Json<AuthPolicySubjectAddReq>, req: HttpRequest) -> BIOSResp {
    let ident_info = get_ident_account_info(&req)?;
    let auth_policy_id: String = req.match_info().get("auth_policy_id").unwrap().parse()?;
    let id = bios::basic::field::uuid();

    if !BIOSFuns::reldb()
        .exists(
            &Query::select()
                .columns(vec![IamAuthPolicy::Id])
                .from(IamAuthPolicy::Table)
                .and_where(Expr::col(IamAuthPolicy::Id).eq(auth_policy_id.clone()))
                .and_where(Expr::col(IamAuthPolicy::RelAppId).eq(ident_info.app_id.clone()))
                .done(),
            None,
        )
        .await?
    {
        return BIOSRespHelper::bus_error(BIOSError::NotFound("AuthPolicySubject [rel_auth_policy_id] not exists".to_string()));
    }

    let allow = match auth_policy_subject_add_req.subject_kind {
        AuthSubjectKind::Tenant => auth_policy_subject_add_req.subject_id == ident_info.tenant_id,
        AuthSubjectKind::App => auth_policy_subject_add_req.subject_id == ident_info.app_id,
        AuthSubjectKind::Role => {
            BIOSFuns::reldb()
                .exists(
                    &Query::select()
                        .columns(vec![IamRole::Id])
                        .from(IamRole::Table)
                        .and_where(Expr::col(IamRole::Id).eq(auth_policy_subject_add_req.subject_id.clone()))
                        .and_where(Expr::col(IamRole::RelAppId).eq(ident_info.app_id.clone()))
                        .done(),
                    None,
                )
                .await?
        }
        AuthSubjectKind::GroupNode => {
            let split_idx = auth_policy_subject_add_req.subject_id.clone().find(".").unwrap();
            let group_id = &auth_policy_subject_add_req.subject_id.as_str()[..split_idx];
            let group_node_code = &auth_policy_subject_add_req.subject_id.as_str()[split_idx + 1..];
            BIOSFuns::reldb()
                .exists(
                    &Query::select()
                        .columns(vec![(IamGroupNode::Table, IamGroupNode::Id)])
                        .from(IamGroupNode::Table)
                        .inner_join(
                            IamGroup::Table,
                            Expr::tbl(IamGroup::Table, IamGroup::Id).equals(IamGroupNode::Table, IamGroupNode::RelGroupId),
                        )
                        .and_where(Expr::tbl(IamGroupNode::Table, IamGroupNode::Code).eq(group_node_code.clone()))
                        .and_where(Expr::tbl(IamGroup::Table, IamGroup::Id).eq(group_id.clone()))
                        .and_where(Expr::tbl(IamGroup::Table, IamGroup::RelAppId).eq(ident_info.app_id.clone()))
                        .done(),
                    None,
                )
                .await?
        }
        AuthSubjectKind::Account => {
            BIOSFuns::reldb()
                .exists(
                    &Query::select()
                        .columns(vec![IamAccountApp::Id])
                        .from(IamAccountApp::Table)
                        .and_where(Expr::col(IamAccountApp::RelAccountId).eq(auth_policy_subject_add_req.subject_id.clone()))
                        .and_where(Expr::col(IamAccountApp::RelAppId).eq(ident_info.app_id.clone()))
                        .done(),
                    None,
                )
                .await?
        }
    };
    if !allow {
        {
            return BIOSRespHelper::bus_error(BIOSError::NotFound("AuthPolicySubject [subject_id] not exists".to_string()));
        }
    }
    if BIOSFuns::reldb()
        .exists(
            &Query::select()
                .columns(vec![IamAuthPolicySubject::Id])
                .from(IamAuthPolicySubject::Table)
                .and_where(Expr::col(IamAuthPolicySubject::SubjectKind).eq(auth_policy_subject_add_req.subject_kind.to_string().to_lowercase()))
                .and_where(Expr::col(IamAuthPolicySubject::SubjectId).eq(auth_policy_subject_add_req.subject_id.clone()))
                .and_where(Expr::col(IamAuthPolicySubject::RelAuthPolicyId).eq(auth_policy_id.clone()))
                .done(),
            None,
        )
        .await?
    {
        return BIOSRespHelper::bus_error(BIOSError::Conflict("AuthPolicySubject already exists".to_string()));
    }

    let mut conn = BIOSFuns::reldb().conn().await;
    let mut tx = conn.begin().await?;

    BIOSFuns::reldb()
        .exec(
            &Query::insert()
                .into_table(IamAuthPolicySubject::Table)
                .columns(vec![
                    IamAuthPolicySubject::Id,
                    IamAuthPolicySubject::CreateUser,
                    IamAuthPolicySubject::UpdateUser,
                    IamAuthPolicySubject::SubjectKind,
                    IamAuthPolicySubject::SubjectId,
                    IamAuthPolicySubject::SubjectOperator,
                    IamAuthPolicySubject::RelAuthPolicyId,
                ])
                .values_panic(vec![
                    id.clone().into(),
                    ident_info.account_id.clone().into(),
                    ident_info.account_id.clone().into(),
                    auth_policy_subject_add_req.subject_kind.to_string().to_lowercase().into(),
                    auth_policy_subject_add_req.subject_id.clone().into(),
                    auth_policy_subject_add_req.subject_operator.to_string().to_lowercase().into(),
                    auth_policy_id.clone().into(),
                ])
                .done(),
            Some(&mut tx),
        )
        .await?;
    rebuild_cache(&auth_policy_id, &mut tx).await?;
    tx.commit().await?;
    BIOSRespHelper::ok(id)
}

#[get("/console/app/auth-policy/{auth_policy_id}/subject")]
pub async fn list_auth_policy_subject(req: HttpRequest) -> BIOSResp {
    let ident_info = get_ident_account_info(&req)?;
    let auth_policy_id: String = req.match_info().get("auth_policy_id").unwrap().parse()?;

    if !BIOSFuns::reldb()
        .exists(
            &Query::select()
                .columns(vec![IamAuthPolicy::Id])
                .from(IamAuthPolicy::Table)
                .and_where(Expr::col(IamAuthPolicy::Id).eq(auth_policy_id.clone()))
                .and_where(Expr::col(IamAuthPolicy::RelAppId).eq(ident_info.app_id.clone()))
                .done(),
            None,
        )
        .await?
    {
        return BIOSRespHelper::bus_error(BIOSError::NotFound("AuthPolicySubject [rel_auth_policy_id] not exists".to_string()));
    }

    let create_user_table = Alias::new("create");
    let update_user_table = Alias::new("update");
    let sql_builder = Query::select()
        .columns(vec![
            (IamAuthPolicySubject::Table, IamAuthPolicySubject::Id),
            (IamAuthPolicySubject::Table, IamAuthPolicySubject::CreateTime),
            (IamAuthPolicySubject::Table, IamAuthPolicySubject::UpdateTime),
            (IamAuthPolicySubject::Table, IamAuthPolicySubject::SubjectKind),
            (IamAuthPolicySubject::Table, IamAuthPolicySubject::SubjectId),
            (IamAuthPolicySubject::Table, IamAuthPolicySubject::SubjectOperator),
            (IamAuthPolicySubject::Table, IamAuthPolicySubject::RelAuthPolicyId),
        ])
        .expr_as(Expr::tbl(create_user_table.clone(), IamAccount::Name), Alias::new("create_user"))
        .expr_as(Expr::tbl(update_user_table.clone(), IamAccount::Name), Alias::new("update_user"))
        .from(IamAuthPolicySubject::Table)
        .join_as(
            JoinType::InnerJoin,
            IamAccount::Table,
            create_user_table.clone(),
            Expr::tbl(create_user_table, IamAccount::Id).equals(IamAuthPolicySubject::Table, IamAuthPolicySubject::CreateUser),
        )
        .join_as(
            JoinType::InnerJoin,
            IamAccount::Table,
            update_user_table.clone(),
            Expr::tbl(update_user_table, IamAccount::Id).equals(IamAuthPolicySubject::Table, IamAuthPolicySubject::UpdateUser),
        )
        .and_where(Expr::tbl(IamAuthPolicySubject::Table, IamAuthPolicySubject::RelAuthPolicyId).eq(auth_policy_id))
        .order_by(IamAuthPolicySubject::UpdateTime, Order::Desc)
        .done();
    let items = BIOSFuns::reldb().fetch_all::<AuthPolicySubjectDetailResp>(&sql_builder, None).await?;
    BIOSRespHelper::ok(items)
}

#[delete("/console/app/auth-policy/{auth_policy_id}/subject/{id}")]
pub async fn delete_auth_policy_subject(req: HttpRequest) -> BIOSResp {
    let ident_info = get_ident_account_info(&req)?;
    let auth_policy_id: String = req.match_info().get("auth_policy_id").unwrap().parse()?;
    let id: String = req.match_info().get("id").unwrap().parse()?;

    if !BIOSFuns::reldb()
        .exists(
            &Query::select()
                .columns(vec![IamAuthPolicy::Id])
                .from(IamAuthPolicy::Table)
                .and_where(Expr::col(IamAuthPolicy::Id).eq(auth_policy_id.clone()))
                .and_where(Expr::col(IamAuthPolicy::RelAppId).eq(ident_info.app_id.clone().to_lowercase()))
                .done(),
            None,
        )
        .await?
    {
        return BIOSRespHelper::bus_error(BIOSError::NotFound("AuthPolicy not exists".to_string()));
    }
    if !BIOSFuns::reldb()
        .exists(
            &Query::select()
                .columns(vec![IamAuthPolicySubject::Id])
                .from(IamAuthPolicySubject::Table)
                .and_where(Expr::col(IamAuthPolicySubject::Id).eq(id.clone()))
                .and_where(Expr::col(IamAuthPolicySubject::RelAuthPolicyId).eq(auth_policy_id.clone()))
                .done(),
            None,
        )
        .await?
    {
        return BIOSRespHelper::bus_error(BIOSError::NotFound("AuthPolicySubject not exists".to_string()));
    }
    if !BIOSFuns::reldb()
        .exists(
            &Query::select()
                .columns(vec![(IamAuthPolicySubject::Table, IamAuthPolicySubject::Id)])
                .from(IamAuthPolicySubject::Table)
                .inner_join(
                    IamAuthPolicy::Table,
                    Expr::tbl(IamAuthPolicy::Table, IamAuthPolicy::Id).equals(IamAuthPolicySubject::Table, IamAuthPolicySubject::RelAuthPolicyId),
                )
                .and_where(Expr::tbl(IamAuthPolicy::Table, IamAuthPolicy::Id).eq(auth_policy_id.clone()))
                .and_where(Expr::tbl(IamAuthPolicySubject::Table, IamAuthPolicySubject::Id).eq(id.clone()))
                .and_where(Expr::tbl(IamAuthPolicy::Table, IamAuthPolicy::RelAppId).eq(ident_info.app_id.clone()))
                .done(),
            None,
        )
        .await?
    {
        return BIOSRespHelper::bus_error(BIOSError::NotFound("AuthPolicySubject not exists".to_string()));
    }

    let mut conn = BIOSFuns::reldb().conn().await;
    let mut tx = conn.begin().await?;

    let sql_builder = Query::select()
        .columns(IamAuthPolicySubject::iter().filter(|i| *i != IamAuthPolicySubject::Table))
        .from(IamAuthPolicySubject::Table)
        .and_where(Expr::col(IamAuthPolicySubject::Id).eq(id.clone()))
        .done();
    BIOSFuns::reldb()
        .soft_del(IamAuthPolicySubject::Table, IamAuthPolicySubject::Id, &ident_info.account_id, &sql_builder, &mut tx)
        .await?;
    rebuild_cache(&auth_policy_id, &mut tx).await?;
    tx.commit().await?;
    BIOSRespHelper::ok(id)
}

// ------------------------------------

async fn delete_cache<'c>(auth_policy_id: &str, tx: &mut Transaction<'c, MySql>) -> BIOSResult<()> {
    let key_info = BIOSFuns::reldb()
        .fetch_one::<RebuildKeyInfoResp>(
            &Query::select()
                .column((IamAuthPolicy::Table, IamAuthPolicy::ActionKind))
                .column((IamResourceSubject::Table, IamResourceSubject::Uri))
                .column((IamResource::Table, IamResource::PathAndQuery))
                .column((IamAuthPolicy::Table, IamAuthPolicy::ValidStartTime))
                .column((IamAuthPolicy::Table, IamAuthPolicy::ValidEndTime))
                .from(IamAuthPolicy::Table)
                .inner_join(
                    IamResource::Table,
                    Expr::tbl(IamResource::Table, IamResource::Id).equals(IamAuthPolicy::Table, IamAuthPolicy::RelResourceId),
                )
                .inner_join(
                    IamResourceSubject::Table,
                    Expr::tbl(IamResourceSubject::Table, IamResourceSubject::Id).equals(IamResource::Table, IamResource::RelResourceSubjectId),
                )
                .and_where(Expr::tbl(IamAuthPolicy::Table, IamAuthPolicy::Id).eq(auth_policy_id))
                .done(),
            Some(tx),
        )
        .await?;

    let field = format!(
        "{}##{}",
        &key_info.action_kind,
        bios::basic::uri::format_with_item(&key_info.uri, &key_info.path_and_query).unwrap()
    );
    BIOSFuns::cache().hdel(&BIOSFuns::ws_config::<WorkSpaceConfig>().iam.cache_resources, &field).await?;
    BIOSFuns::cache()
        .set_ex(
            format!("{}{}", &BIOSFuns::ws_config::<WorkSpaceConfig>().iam.cache_change_resources, Utc::now().timestamp_nanos()).as_str(),
            &field,
            BIOSFuns::ws_config::<WorkSpaceConfig>().iam.cache_change_resources_exp,
        )
        .await?;
    Ok(())
}

// api://app1.tenant1/p1?a=1##get","{\"account_ids\":\"#acc1#\"}
async fn rebuild_cache<'c>(auth_policy_id: &str, tx: &mut Transaction<'c, MySql>) -> BIOSResult<()> {
    let key_info = BIOSFuns::reldb()
        .fetch_one::<RebuildKeyInfoResp>(
            &Query::select()
                .column((IamAuthPolicy::Table, IamAuthPolicy::ActionKind))
                .column((IamResourceSubject::Table, IamResourceSubject::Uri))
                .column((IamResource::Table, IamResource::PathAndQuery))
                .column((IamAuthPolicy::Table, IamAuthPolicy::ValidStartTime))
                .column((IamAuthPolicy::Table, IamAuthPolicy::ValidEndTime))
                .from(IamAuthPolicy::Table)
                .inner_join(
                    IamResource::Table,
                    Expr::tbl(IamResource::Table, IamResource::Id).equals(IamAuthPolicy::Table, IamAuthPolicy::RelResourceId),
                )
                .inner_join(
                    IamResourceSubject::Table,
                    Expr::tbl(IamResourceSubject::Table, IamResourceSubject::Id).equals(IamResource::Table, IamResource::RelResourceSubjectId),
                )
                .and_where(Expr::tbl(IamAuthPolicy::Table, IamAuthPolicy::Id).eq(auth_policy_id))
                .done(),
            Some(tx),
        )
        .await?;
    let mut value_info = BIOSFuns::reldb()
        .fetch_all::<RebuildValueInfoResp>(
            &Query::select()
                .column(IamAuthPolicySubject::SubjectKind)
                .column(IamAuthPolicySubject::SubjectId)
                .from(IamAuthPolicySubject::Table)
                .and_where(Expr::col(IamAuthPolicySubject::RelAuthPolicyId).eq(auth_policy_id))
                .and_where(Expr::col(IamAuthPolicySubject::SubjectKind).ne(AuthSubjectKind::GroupNode.to_string().to_lowercase()))
                .done(),
            Some(tx),
        )
        .await?;
    let value_info_by_group_node = BIOSFuns::reldb()
        .fetch_all::<RebuildValueByGroupNodeInfoResp>(
            &Query::select()
                .column((IamGroupNode::Table, IamGroupNode::Id))
                .column((IamGroupNode::Table, IamGroupNode::Code))
                .from(IamAuthPolicySubject::Table)
                .inner_join(
                    IamGroupNode::Table,
                    Expr::tbl(IamGroupNode::Table, IamGroupNode::Id).equals(IamAuthPolicySubject::Table, IamAuthPolicySubject::SubjectId),
                )
                .and_where(Expr::tbl(IamAuthPolicySubject::Table, IamAuthPolicySubject::RelAuthPolicyId).eq(auth_policy_id))
                .and_where(Expr::tbl(IamAuthPolicySubject::Table, IamAuthPolicySubject::SubjectKind).eq(AuthSubjectKind::GroupNode.to_string().to_lowercase()))
                .done(),
            Some(tx),
        )
        .await?;
    for group_node in value_info_by_group_node {
        value_info.push(RebuildValueInfoResp {
            subject_kind: AuthSubjectKind::GroupNode.to_string().to_lowercase(),
            subject_id: format!("{}.{}", group_node.id, group_node.code),
        })
    }
    let value_info_json = serde_json::json!({
        "_start":key_info.valid_start_time,
        "_end":key_info.valid_end_time,
        AuthSubjectKind::Tenant.to_string().to_lowercase():value_info.iter().filter(|x|x.subject_kind==AuthSubjectKind::Tenant.to_string().to_lowercase()).map(|x|format!("#{}#",x.subject_id.clone())).join(""),
        AuthSubjectKind::App.to_string().to_lowercase():value_info.iter().filter(|x|x.subject_kind==AuthSubjectKind::App.to_string().to_lowercase()).map(|x|format!("#{}#",x.subject_id.clone())).join(""),
        AuthSubjectKind::Account.to_string().to_lowercase():value_info.iter().filter(|x|x.subject_kind==AuthSubjectKind::Account.to_string().to_lowercase()).map(|x|format!("#{}#",x.subject_id.clone())).join(""),
        AuthSubjectKind::Role.to_string().to_lowercase():value_info.iter().filter(|x|x.subject_kind==AuthSubjectKind::Role.to_string().to_lowercase()).map(|x|format!("#{}#",x.subject_id.clone())).join(""),
        AuthSubjectKind::GroupNode.to_string().to_lowercase():value_info.iter().filter(|x|x.subject_kind==AuthSubjectKind::GroupNode.to_string().to_lowercase()).map(|x|format!("#{}#",x.subject_id.clone())).join(""),
    });

    // TODO
    /* let value_info_json = value_info
    .into_iter()
    .group_by(|x| x.subject_kind)
    .into_iter()
    .map(|(group, records)| (group, records.into_iter().map(|record| format!("#{}#", record.subject_id)).join("")))
    .collect();*/

    let field = format!(
        "{}##{}",
        &key_info.action_kind,
        bios::basic::uri::format_with_item(&key_info.uri, &key_info.path_and_query).unwrap()
    );
    BIOSFuns::cache()
        .hset(
            &BIOSFuns::ws_config::<WorkSpaceConfig>().iam.cache_resources,
            &field,
            bios::basic::json::obj_to_string(&value_info_json)?.as_str(),
        )
        .await?;
    BIOSFuns::cache()
        .set_ex(
            format!("{}{}", &BIOSFuns::ws_config::<WorkSpaceConfig>().iam.cache_change_resources, Utc::now().timestamp_nanos()).as_str(),
            &field,
            BIOSFuns::ws_config::<WorkSpaceConfig>().iam.cache_change_resources_exp,
        )
        .await?;
    Ok(())
}

#[derive(sqlx::FromRow, serde::Deserialize)]
pub struct RebuildKeyInfoResp {
    pub action_kind: String,
    pub uri: String,
    pub path_and_query: String,
    pub valid_start_time: i64,
    pub valid_end_time: i64,
}

#[derive(sqlx::FromRow, serde::Deserialize)]
pub struct RebuildValueInfoResp {
    pub subject_kind: String,
    pub subject_id: String,
}

#[derive(sqlx::FromRow, serde::Deserialize)]
pub struct RebuildValueByGroupNodeInfoResp {
    pub id: String,
    pub code: String,
}
