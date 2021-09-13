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

use chrono::Utc;
use itertools::Itertools;
use sea_query::{Expr, Query};
use sqlx::{MySql, Transaction};

use bios::basic::error::BIOSResult;
use bios::db::reldb_client::SqlBuilderProcess;
use bios::BIOSFuns;

use crate::domain::auth_domain::{IamAuthPolicy, IamAuthPolicySubject, IamGroupNode, IamResource, IamResourceSubject};
use crate::iam_config::WorkSpaceConfig;
use crate::process::basic_dto::AuthSubjectKind;

pub async fn remove_token_by_account(account_id: &str) -> BIOSResult<()> {
    let tokens = BIOSFuns::cache().hgetall(format!("{}{}", BIOSFuns::ws_config::<WorkSpaceConfig>().iam.cache_token_rel, account_id).as_str()).await?;
    BIOSFuns::cache().del(format!("{}{}", BIOSFuns::ws_config::<WorkSpaceConfig>().iam.cache_token_rel, account_id).as_str()).await?;
    for (k, _) in tokens {
        BIOSFuns::cache().del(format!("{}{}", BIOSFuns::ws_config::<WorkSpaceConfig>().iam.cache_access_token, k).as_str()).await?;
    }
    Ok(())
}

// ------------------------------------

pub async fn set_aksk(tenant_id: &str, app_id: &str, ak: &str, sk: &str, valid_time: i64) -> BIOSResult<()> {
    match BIOSFuns::cache()
        .set_ex(
            format!("{}{}", &BIOSFuns::ws_config::<WorkSpaceConfig>().iam.cache_aksk, ak).as_str(),
            format!("{}:{}:{}", sk, tenant_id, app_id).as_str(),
            (valid_time - Utc::now().timestamp()) as usize,
        )
        .await
    {
        Ok(result) => Ok(result),
        Err(e) => Err(e.into()),
    }
}

pub async fn remove_aksk(ak: &str) -> BIOSResult<()> {
    match BIOSFuns::cache().del(format!("{}{}", &BIOSFuns::ws_config::<WorkSpaceConfig>().iam.cache_aksk, ak).as_str()).await {
        Ok(result) => Ok(result),
        Err(e) => Err(e.into()),
    }
}

// ------------------------------------

pub async fn get_vcode(tenant_id: &str, ak: &str) -> BIOSResult<Option<String>> {
    match BIOSFuns::cache().get(format!("{}{}:{}", BIOSFuns::ws_config::<WorkSpaceConfig>().iam.cache_account_vcode_tmp_rel, tenant_id, ak).as_str()).await {
        Ok(result) => Ok(result),
        Err(e) => Err(e.into()),
    }
}

pub async fn get_account_token(rel_app_id: &str, kind: &str) -> BIOSResult<Option<String>> {
    match BIOSFuns::cache().get(format!("{}{}:{}", BIOSFuns::ws_config::<WorkSpaceConfig>().iam.cache_access_token, rel_app_id, kind).as_str()).await {
        Ok(result) => Ok(result),
        Err(e) => Err(e.into()),
    }
}

// ------------------------------------

pub async fn remove_auth_policy<'c>(auth_policy_id: &str, tx: &mut Transaction<'c, MySql>) -> BIOSResult<()> {
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
pub async fn rebuild_auth_policy<'c>(auth_policy_id: &str, tx: &mut Transaction<'c, MySql>) -> BIOSResult<()> {
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
