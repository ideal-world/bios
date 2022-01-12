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
use sea_query::{Alias, Cond, Expr, JoinType, Order, Query};
use sqlx::Connection;
use strum::IntoEnumIterator;

use bios::basic::dto::BIOSResp;
use bios::db::reldb_client::SqlBuilderProcess;
use bios::web::basic_processor::extract_context_with_account;
use bios::web::resp_handler::BIOSResponse;
use bios::web::validate::json::Json;
use bios::web::validate::query::Query as VQuery;
use bios::BIOSFuns;

use crate::domain::auth_domain::{IamAuthPolicy, IamAuthPolicyObject, IamGroup, IamGroupNode, IamResource, IamRole};
use crate::domain::ident_domain::{IamAccount, IamAccountApp};
use crate::iam_constant::{IamOutput, ObjectKind};
use crate::process::app_console::ac_auth_policy_dto::{
    AuthPolicyAddReq, AuthPolicyDetailResp, AuthPolicyModifyReq, AuthPolicyObjectAddReq, AuthPolicyObjectDetailResp, AuthPolicyQueryReq,
};
use crate::process::basic_dto::AuthObjectKind;
use crate::process::common::cache_processor;

#[post("/console/app/auth-policy")]
pub async fn add_auth_policy(auth_policy_add_req: Json<AuthPolicyAddReq>, req: HttpRequest) -> BIOSResponse {
    let context = extract_context_with_account(&req)?;
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
                                .add(Expr::tbl(IamResource::Table, IamResource::RelTenantId).eq(context.ident.tenant_id.as_str()))
                                .add(Expr::tbl(IamResource::Table, IamResource::ExposeKind).eq(crate::process::basic_dto::ExposeKind::Tenant.to_string().to_lowercase())),
                        )
                        .add(
                            Cond::all()
                                .add(Expr::tbl(IamResource::Table, IamResource::RelAppId).eq(context.ident.app_id.as_str()))
                                .add(Expr::tbl(IamResource::Table, IamResource::ExposeKind).eq(crate::process::basic_dto::ExposeKind::App.to_string().to_lowercase())),
                        ),
                )
                .and_where(Expr::col(IamResource::Id).eq(auth_policy_add_req.rel_resource_id.as_str()))
                .done(),
            None,
        )
        .await?
    {
        return BIOSResp::err(IamOutput::AppConsoleEntityCreateCheckNotFound(ObjectKind::AuthPolicy, "Resource"), Some(&context));
    }

    if BIOSFuns::reldb()
        .exists(
            &Query::select()
                .columns(vec![IamAuthPolicy::Id])
                .from(IamAuthPolicy::Table)
                .and_where(Expr::col(IamAuthPolicy::ActionKind).eq(auth_policy_add_req.action_kind.to_string().to_lowercase()))
                .and_where(Expr::col(IamAuthPolicy::RelResourceId).eq(auth_policy_add_req.rel_resource_id.as_str()))
                .and_where(Expr::col(IamAuthPolicy::ValidStartTime).lte(auth_policy_add_req.valid_start_time))
                .and_where(Expr::col(IamAuthPolicy::ValidEndTime).gte(auth_policy_add_req.valid_end_time))
                .and_where(Expr::col(IamAuthPolicy::ResultKind).eq(auth_policy_add_req.result_kind.to_string().to_lowercase()))
                .done(),
            None,
        )
        .await?
    {
        return BIOSResp::err(IamOutput::AppConsoleEntityCreateCheckExists(ObjectKind::AuthPolicy, "AuthPolicy"), Some(&context));
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
                    id.as_str().into(),
                    context.ident.account_id.as_str().into(),
                    context.ident.account_id.as_str().into(),
                    auth_policy_add_req.name.as_str().into(),
                    auth_policy_add_req.valid_start_time.into(),
                    auth_policy_add_req.valid_end_time.into(),
                    auth_policy_add_req.action_kind.to_string().to_lowercase().into(),
                    auth_policy_add_req.rel_resource_id.as_str().into(),
                    auth_policy_add_req.result_kind.to_string().to_lowercase().into(),
                    context.ident.app_id.as_str().into(),
                    context.ident.tenant_id.as_str().into(),
                ])
                .done(),
            Some(&mut tx),
        )
        .await?;
    cache_processor::rebuild_auth_policy(&id, &mut tx, &context).await?;
    tx.commit().await?;
    BIOSResp::ok(id, Some(&context))
}

#[put("/console/app/auth-policy/{id}")]
pub async fn modify_auth_policy(auth_policy_modify_req: Json<AuthPolicyModifyReq>, req: HttpRequest) -> BIOSResponse {
    let context = extract_context_with_account(&req)?;
    let id: String = req.match_info().get("id").unwrap().parse()?;

    if auth_policy_modify_req.valid_start_time.is_some() && auth_policy_modify_req.valid_end_time.is_none()
        || auth_policy_modify_req.valid_start_time.is_none() && auth_policy_modify_req.valid_end_time.is_some()
    {
        return BIOSResp::err(
            IamOutput::AppConsoleEntityModifyCheckExistFieldsAtSomeTime(ObjectKind::AuthPolicy, "[valid_start_time] and [valid_end_time]"),
            Some(&context),
        );
    }
    if !BIOSFuns::reldb()
        .exists(
            &Query::select()
                .columns(vec![IamAuthPolicy::Id])
                .from(IamAuthPolicy::Table)
                .and_where(Expr::col(IamAuthPolicy::Id).eq(id.as_str()))
                .and_where(Expr::col(IamAuthPolicy::RelAppId).eq(context.ident.app_id.as_str()))
                .done(),
            None,
        )
        .await?
    {
        return BIOSResp::err(IamOutput::AppConsoleEntityModifyCheckNotFound(ObjectKind::AuthPolicy, "AuthPolicy"), Some(&context));
    }
    if auth_policy_modify_req.valid_start_time.is_some() && auth_policy_modify_req.valid_end_time.is_some() {
        if BIOSFuns::reldb()
            .exists(
                &Query::select()
                    .columns(vec![IamAuthPolicy::Id])
                    .from(IamAuthPolicy::Table)
                    .and_where(Expr::col(IamAuthPolicy::Id).ne(id.as_str()))
                    .and_where(Expr::col(IamAuthPolicy::ValidStartTime).lte(auth_policy_modify_req.valid_start_time.unwrap()))
                    .and_where(Expr::col(IamAuthPolicy::ValidEndTime).gte(auth_policy_modify_req.valid_end_time.unwrap()))
                    .done(),
                None,
            )
            .await?
        {
            return BIOSResp::err(IamOutput::AppConsoleEntityModifyCheckExists(ObjectKind::AuthPolicy, "AuthPolicy"), Some(&context));
        }
    }

    let mut conn = BIOSFuns::reldb().conn().await;
    let mut tx = conn.begin().await?;

    let mut values = Vec::new();
    if let Some(name) = &auth_policy_modify_req.name {
        values.push((IamAuthPolicy::Name, name.as_str().into()));
    }
    if let Some(valid_start_time) = auth_policy_modify_req.valid_start_time {
        values.push((IamAuthPolicy::ValidStartTime, valid_start_time.into()));
    }
    if let Some(valid_end_time) = auth_policy_modify_req.valid_end_time {
        values.push((IamAuthPolicy::ValidEndTime, valid_end_time.into()));
    }
    values.push((IamAuthPolicy::UpdateUser, context.ident.account_id.as_str().into()));
    BIOSFuns::reldb()
        .exec(
            &Query::update()
                .table(IamAuthPolicy::Table)
                .values(values)
                .and_where(Expr::col(IamAuthPolicy::Id).eq(id.as_str()))
                .and_where(Expr::col(IamAuthPolicy::RelAppId).eq(context.ident.app_id.as_str()))
                .done(),
            Some(&mut tx),
        )
        .await?;
    cache_processor::rebuild_auth_policy(&id, &mut tx, &context).await?;
    tx.commit().await?;
    BIOSResp::ok("", Some(&context))
}

#[get("/console/app/auth-policy")]
pub async fn list_auth_policy(query: VQuery<AuthPolicyQueryReq>, req: HttpRequest) -> BIOSResponse {
    let context = extract_context_with_account(&req)?;

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
        .and_where(Expr::tbl(IamAuthPolicy::Table, IamAuthPolicy::RelAppId).eq(context.ident.app_id.as_str()))
        .order_by(IamAuthPolicy::UpdateTime, Order::Desc)
        .done();
    let items = BIOSFuns::reldb().pagination::<AuthPolicyDetailResp>(&sql_builder, query.page_number, query.page_size, None).await?;
    BIOSResp::ok(items, Some(&context))
}

#[delete("/console/app/auth-policy/{id}")]
pub async fn delete_auth_policy(req: HttpRequest) -> BIOSResponse {
    let context = extract_context_with_account(&req)?;
    let id: String = req.match_info().get("id").unwrap().parse()?;

    if !BIOSFuns::reldb()
        .exists(
            &Query::select()
                .columns(vec![IamAuthPolicy::Id])
                .from(IamAuthPolicy::Table)
                .and_where(Expr::col(IamAuthPolicy::Id).eq(id.as_str()))
                .and_where(Expr::col(IamAuthPolicy::RelAppId).eq(context.ident.app_id.as_str()))
                .done(),
            None,
        )
        .await?
    {
        return BIOSResp::err(IamOutput::AppConsoleEntityDeleteCheckNotFound(ObjectKind::AuthPolicy, "AuthPolicy"), Some(&context));
    }
    if BIOSFuns::reldb()
        .exists(
            &Query::select()
                .columns(vec![IamAuthPolicyObject::Id])
                .from(IamAuthPolicyObject::Table)
                .and_where(Expr::col(IamAuthPolicyObject::RelAuthPolicyId).eq(id.as_str()))
                .done(),
            None,
        )
        .await?
    {
        return BIOSResp::err(
            IamOutput::AppConsoleEntityDeleteCheckExistAssociatedData(ObjectKind::AuthPolicy, "AuthPolicyObject"),
            Some(&context),
        );
    }

    let mut conn = BIOSFuns::reldb().conn().await;
    let mut tx = conn.begin().await?;

    cache_processor::remove_auth_policy(&id, &mut tx, &context).await?;

    let sql_builder = Query::select()
        .columns(IamAuthPolicy::iter().filter(|i| *i != IamAuthPolicy::Table))
        .from(IamAuthPolicy::Table)
        .and_where(Expr::col(IamAuthPolicy::Id).eq(id.as_str()))
        .and_where(Expr::col(IamAuthPolicy::RelAppId).eq(context.ident.app_id.as_str()))
        .done();
    BIOSFuns::reldb().soft_del(IamAuthPolicy::Table, IamAuthPolicy::Id, &context.ident.account_id, &sql_builder, &mut tx).await?;

    tx.commit().await?;
    BIOSResp::ok("", Some(&context))
}

// ------------------------------------

#[post("/console/app/auth-policy/{auth_policy_id}/object")]
pub async fn add_auth_policy_object(auth_policy_object_add_req: Json<AuthPolicyObjectAddReq>, req: HttpRequest) -> BIOSResponse {
    let context = extract_context_with_account(&req)?;
    let auth_policy_id: String = req.match_info().get("auth_policy_id").unwrap().parse()?;
    let id = bios::basic::field::uuid();

    if !BIOSFuns::reldb()
        .exists(
            &Query::select()
                .columns(vec![IamAuthPolicy::Id])
                .from(IamAuthPolicy::Table)
                .and_where(Expr::col(IamAuthPolicy::Id).eq(auth_policy_id.as_str()))
                .and_where(Expr::col(IamAuthPolicy::RelAppId).eq(context.ident.app_id.as_str()))
                .done(),
            None,
        )
        .await?
    {
        return BIOSResp::err(IamOutput::AppConsoleEntityCreateCheckNotFound(ObjectKind::AuthPolicyObject, "AuthPolicy"), Some(&context));
    }

    let allow = match auth_policy_object_add_req.object_kind {
        AuthObjectKind::Tenant => auth_policy_object_add_req.object_id == context.ident.tenant_id,
        AuthObjectKind::App => auth_policy_object_add_req.object_id == context.ident.app_id,
        AuthObjectKind::Role => {
            BIOSFuns::reldb()
                .exists(
                    &Query::select()
                        .columns(vec![IamRole::Id])
                        .from(IamRole::Table)
                        .and_where(Expr::col(IamRole::Id).eq(auth_policy_object_add_req.object_id.as_str()))
                        .and_where(Expr::col(IamRole::RelAppId).eq(context.ident.app_id.as_str()))
                        .done(),
                    None,
                )
                .await?
        }
        AuthObjectKind::GroupNode => {
            let split_idx = auth_policy_object_add_req.object_id.as_str().find(".").unwrap();
            let group_id = &auth_policy_object_add_req.object_id.as_str()[..split_idx];
            let group_node_code = &auth_policy_object_add_req.object_id.as_str()[split_idx + 1..];
            BIOSFuns::reldb()
                .exists(
                    &Query::select()
                        .columns(vec![(IamGroupNode::Table, IamGroupNode::Id)])
                        .from(IamGroupNode::Table)
                        .inner_join(
                            IamGroup::Table,
                            Expr::tbl(IamGroup::Table, IamGroup::Id).equals(IamGroupNode::Table, IamGroupNode::RelGroupId),
                        )
                        .and_where(Expr::tbl(IamGroupNode::Table, IamGroupNode::Code).eq(group_node_code))
                        .and_where(Expr::tbl(IamGroup::Table, IamGroup::Id).eq(group_id))
                        .and_where(Expr::tbl(IamGroup::Table, IamGroup::RelAppId).eq(context.ident.app_id.as_str()))
                        .done(),
                    None,
                )
                .await?
        }
        AuthObjectKind::Account => {
            BIOSFuns::reldb()
                .exists(
                    &Query::select()
                        .columns(vec![IamAccountApp::Id])
                        .from(IamAccountApp::Table)
                        .and_where(Expr::col(IamAccountApp::RelAccountId).eq(auth_policy_object_add_req.object_id.as_str()))
                        .and_where(Expr::col(IamAccountApp::RelAppId).eq(context.ident.app_id.as_str()))
                        .done(),
                    None,
                )
                .await?
        }
    };
    if !allow {
        {
            return BIOSResp::err(
                IamOutput::AppConsoleEntityCreateCheckNotFoundField(ObjectKind::AuthPolicyObject, "object_id"),
                Some(&context),
            );
        }
    }
    if BIOSFuns::reldb()
        .exists(
            &Query::select()
                .columns(vec![IamAuthPolicyObject::Id])
                .from(IamAuthPolicyObject::Table)
                .and_where(Expr::col(IamAuthPolicyObject::ObjectKind).eq(auth_policy_object_add_req.object_kind.to_string().to_lowercase()))
                .and_where(Expr::col(IamAuthPolicyObject::ObjectId).eq(auth_policy_object_add_req.object_id.as_str()))
                .and_where(Expr::col(IamAuthPolicyObject::RelAuthPolicyId).eq(auth_policy_id.as_str()))
                .done(),
            None,
        )
        .await?
    {
        return BIOSResp::err(
            IamOutput::AppConsoleEntityCreateCheckExists(ObjectKind::AuthPolicyObject, "AuthPolicyObject"),
            Some(&context),
        );
    }

    let mut conn = BIOSFuns::reldb().conn().await;
    let mut tx = conn.begin().await?;

    BIOSFuns::reldb()
        .exec(
            &Query::insert()
                .into_table(IamAuthPolicyObject::Table)
                .columns(vec![
                    IamAuthPolicyObject::Id,
                    IamAuthPolicyObject::CreateUser,
                    IamAuthPolicyObject::UpdateUser,
                    IamAuthPolicyObject::ObjectKind,
                    IamAuthPolicyObject::ObjectId,
                    IamAuthPolicyObject::ObjectOperator,
                    IamAuthPolicyObject::RelAuthPolicyId,
                ])
                .values_panic(vec![
                    id.as_str().into(),
                    context.ident.account_id.as_str().into(),
                    context.ident.account_id.as_str().into(),
                    auth_policy_object_add_req.object_kind.to_string().to_lowercase().into(),
                    auth_policy_object_add_req.object_id.as_str().into(),
                    auth_policy_object_add_req.object_operator.to_string().to_lowercase().into(),
                    auth_policy_id.as_str().into(),
                ])
                .done(),
            Some(&mut tx),
        )
        .await?;
    cache_processor::rebuild_auth_policy(&auth_policy_id, &mut tx, &context).await?;
    tx.commit().await?;
    BIOSResp::ok(id, Some(&context))
}

#[get("/console/app/auth-policy/{auth_policy_id}/object")]
pub async fn list_auth_policy_object(req: HttpRequest) -> BIOSResponse {
    let context = extract_context_with_account(&req)?;
    let auth_policy_id: String = req.match_info().get("auth_policy_id").unwrap().parse()?;

    if !BIOSFuns::reldb()
        .exists(
            &Query::select()
                .columns(vec![IamAuthPolicy::Id])
                .from(IamAuthPolicy::Table)
                .and_where(Expr::col(IamAuthPolicy::Id).eq(auth_policy_id.as_str()))
                .and_where(Expr::col(IamAuthPolicy::RelAppId).eq(context.ident.app_id.as_str()))
                .done(),
            None,
        )
        .await?
    {
        return BIOSResp::err(
            IamOutput::AppConsoleEntityFetchListCheckNotFound(ObjectKind::AuthPolicyObject, "AuthPolicy"),
            Some(&context),
        );
    }

    let create_user_table = Alias::new("create");
    let update_user_table = Alias::new("update");
    let sql_builder = Query::select()
        .columns(vec![
            (IamAuthPolicyObject::Table, IamAuthPolicyObject::Id),
            (IamAuthPolicyObject::Table, IamAuthPolicyObject::CreateTime),
            (IamAuthPolicyObject::Table, IamAuthPolicyObject::UpdateTime),
            (IamAuthPolicyObject::Table, IamAuthPolicyObject::ObjectKind),
            (IamAuthPolicyObject::Table, IamAuthPolicyObject::ObjectId),
            (IamAuthPolicyObject::Table, IamAuthPolicyObject::ObjectOperator),
            (IamAuthPolicyObject::Table, IamAuthPolicyObject::RelAuthPolicyId),
        ])
        .expr_as(Expr::tbl(create_user_table.clone(), IamAccount::Name), Alias::new("create_user"))
        .expr_as(Expr::tbl(update_user_table.clone(), IamAccount::Name), Alias::new("update_user"))
        .from(IamAuthPolicyObject::Table)
        .join_as(
            JoinType::InnerJoin,
            IamAccount::Table,
            create_user_table.clone(),
            Expr::tbl(create_user_table, IamAccount::Id).equals(IamAuthPolicyObject::Table, IamAuthPolicyObject::CreateUser),
        )
        .join_as(
            JoinType::InnerJoin,
            IamAccount::Table,
            update_user_table.clone(),
            Expr::tbl(update_user_table, IamAccount::Id).equals(IamAuthPolicyObject::Table, IamAuthPolicyObject::UpdateUser),
        )
        .and_where(Expr::tbl(IamAuthPolicyObject::Table, IamAuthPolicyObject::RelAuthPolicyId).eq(auth_policy_id))
        .order_by(IamAuthPolicyObject::UpdateTime, Order::Desc)
        .done();
    let items = BIOSFuns::reldb().fetch_all::<AuthPolicyObjectDetailResp>(&sql_builder, None).await?;
    BIOSResp::ok(items, Some(&context))
}

#[delete("/console/app/auth-policy/{auth_policy_id}/object/{id}")]
pub async fn delete_auth_policy_object(req: HttpRequest) -> BIOSResponse {
    let context = extract_context_with_account(&req)?;
    let auth_policy_id: String = req.match_info().get("auth_policy_id").unwrap().parse()?;
    let id: String = req.match_info().get("id").unwrap().parse()?;

    if !BIOSFuns::reldb()
        .exists(
            &Query::select()
                .columns(vec![IamAuthPolicy::Id])
                .from(IamAuthPolicy::Table)
                .and_where(Expr::col(IamAuthPolicy::Id).eq(auth_policy_id.as_str()))
                .and_where(Expr::col(IamAuthPolicy::RelAppId).eq(context.ident.app_id.as_str()))
                .done(),
            None,
        )
        .await?
    {
        return BIOSResp::err(IamOutput::AppConsoleEntityDeleteCheckNotFound(ObjectKind::AuthPolicyObject, "AuthPolicy"), Some(&context));
    }
    if !BIOSFuns::reldb()
        .exists(
            &Query::select()
                .columns(vec![(IamAuthPolicyObject::Table, IamAuthPolicyObject::Id)])
                .from(IamAuthPolicyObject::Table)
                .inner_join(
                    IamAuthPolicy::Table,
                    Expr::tbl(IamAuthPolicy::Table, IamAuthPolicy::Id).equals(IamAuthPolicyObject::Table, IamAuthPolicyObject::RelAuthPolicyId),
                )
                .and_where(Expr::tbl(IamAuthPolicy::Table, IamAuthPolicy::Id).eq(auth_policy_id.as_str()))
                .and_where(Expr::tbl(IamAuthPolicyObject::Table, IamAuthPolicyObject::Id).eq(id.as_str()))
                .and_where(Expr::tbl(IamAuthPolicy::Table, IamAuthPolicy::RelAppId).eq(context.ident.app_id.as_str()))
                .done(),
            None,
        )
        .await?
    {
        return BIOSResp::err(
            IamOutput::AppConsoleEntityDeleteCheckNotFound(ObjectKind::AuthPolicyObject, "AuthPolicyObject"),
            Some(&context),
        );
    }

    let mut conn = BIOSFuns::reldb().conn().await;
    let mut tx = conn.begin().await?;

    let sql_builder = Query::select()
        .columns(IamAuthPolicyObject::iter().filter(|i| *i != IamAuthPolicyObject::Table))
        .from(IamAuthPolicyObject::Table)
        .and_where(Expr::col(IamAuthPolicyObject::Id).eq(id.as_str()))
        .done();
    BIOSFuns::reldb().soft_del(IamAuthPolicyObject::Table, IamAuthPolicyObject::Id, &context.ident.account_id, &sql_builder, &mut tx).await?;
    cache_processor::rebuild_auth_policy(&auth_policy_id, &mut tx, &context).await?;
    tx.commit().await?;
    BIOSResp::ok(id, Some(&context))
}
