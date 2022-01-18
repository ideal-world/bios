/*
 * Copyright 2022. the original author or authors.
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

use sea_query::Query;
use strum::IntoEnumIterator;
use testcontainers::clients;

use bios::basic::result::BIOSResult;
use bios::db::reldb_client::SqlBuilderProcess;
use bios::BIOSFuns;
use bios_baas_iam::domain::auth_domain::{IamAccountRole, IamAuthPolicy, IamAuthPolicyObject, IamResource, IamResourceSubject, IamRole};
use bios_baas_iam::domain::ident_domain::{IamAccount, IamAccountApp, IamAccountIdent, IamApp, IamAppIdent, IamTenant, IamTenantCert, IamTenantIdent};
use bios_baas_iam::iam_config::WorkSpaceConfig;
use bios_baas_iam::iam_initializer;
use bios_baas_iam::process::app_console::ac_account_dto::AccountRoleDetailResp;
use bios_baas_iam::process::app_console::ac_app_dto::AppIdentDetailResp;
use bios_baas_iam::process::app_console::ac_auth_policy_dto::{AuthPolicyDetailResp, AuthPolicyObjectDetailResp};
use bios_baas_iam::process::app_console::ac_resource_dto::{ResourceDetailResp, ResourceSubjectDetailResp};
use bios_baas_iam::process::app_console::ac_role_dto::RoleDetailResp;
use bios_baas_iam::process::tenant_console::tc_account_dto::{AccountAppDetailResp, AccountDetailResp, AccountIdentDetailResp};
use bios_baas_iam::process::tenant_console::tc_app_dto::AppDetailResp;
use bios_baas_iam::process::tenant_console::tc_tenant_dto::{TenantCertDetailResp, TenantDetailResp, TenantIdentDetailResp};

use crate::test_basic;

#[actix_rt::test]
async fn test_init() -> BIOSResult<()> {
    let docker = clients::Cli::default();
    let _c = test_basic::init_without_data(&docker).await;
    let iam_config = &BIOSFuns::ws_config::<WorkSpaceConfig>().iam;

    iam_initializer::init().await?;

    let records = BIOSFuns::reldb()
        .fetch_all::<TenantDetailResp>(
            &Query::select().columns(IamTenant::iter().filter(|i| *i != IamTenant::Table)).from(IamTenant::Table).done(),
            None,
        )
        .await?;
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].name, iam_config.app.tenant_name);
    let tenant_id = records[0].id.clone();

    let records = BIOSFuns::reldb()
        .fetch_all::<TenantIdentDetailResp>(
            &Query::select().columns(IamTenantIdent::iter().filter(|i| *i != IamTenantIdent::Table)).from(IamTenantIdent::Table).done(),
            None,
        )
        .await?;
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].valid_ak_rule, iam_config.security.default_valid_ak_rule);
    assert_eq!(records[0].valid_sk_rule, iam_config.security.default_valid_sk_rule);

    let records = BIOSFuns::reldb()
        .fetch_all::<TenantCertDetailResp>(
            &Query::select().columns(IamTenantCert::iter().filter(|i| *i != IamTenantCert::Table)).from(IamTenantCert::Table).done(),
            None,
        )
        .await?;
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].version, 1);
    assert_eq!(records[0].category, "");

    let records = BIOSFuns::reldb().fetch_all::<AppDetailResp>(&Query::select().columns(IamApp::iter().filter(|i| *i != IamApp::Table)).from(IamApp::Table).done(), None).await?;
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].name, iam_config.app.app_name);
    let app_id = records[0].id.clone();

    let records = BIOSFuns::reldb()
        .fetch_all::<AppIdentDetailResp>(
            &Query::select().columns(IamAppIdent::iter().filter(|i| *i != IamAppIdent::Table)).from(IamAppIdent::Table).done(),
            None,
        )
        .await?;
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].note, "");
    let ak = records[0].ak.clone();

    let records = BIOSFuns::reldb()
        .fetch_all::<AccountDetailResp>(
            &Query::select().columns(IamAccount::iter().filter(|i| *i != IamAccount::Table)).from(IamAccount::Table).done(),
            None,
        )
        .await?;
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].name, iam_config.app.admin_name);

    let records = BIOSFuns::reldb()
        .fetch_all::<AccountIdentDetailResp>(
            &Query::select().columns(IamAccountIdent::iter().filter(|i| *i != IamAccountIdent::Table)).from(IamAccountIdent::Table).done(),
            None,
        )
        .await?;
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].ak, iam_config.app.admin_name);

    let records = BIOSFuns::reldb()
        .fetch_all::<AccountAppDetailResp>(
            &Query::select().columns(IamAccountApp::iter().filter(|i| *i != IamAccountApp::Table)).from(IamAccountApp::Table).done(),
            None,
        )
        .await?;
    assert_eq!(records.len(), 1);

    let records =
        BIOSFuns::reldb().fetch_all::<RoleDetailResp>(&Query::select().columns(IamRole::iter().filter(|i| *i != IamRole::Table)).from(IamRole::Table).done(), None).await?;
    assert_eq!(records.len(), 3);
    assert!(records.iter().find(|x| x.code == iam_config.security.system_admin_role_code).is_some());

    let records = BIOSFuns::reldb()
        .fetch_all::<AccountRoleDetailResp>(
            &Query::select().columns(IamAccountRole::iter().filter(|i| *i != IamAccountRole::Table)).from(IamAccountRole::Table).done(),
            None,
        )
        .await?;
    assert_eq!(records.len(), 3);

    let records = BIOSFuns::reldb()
        .fetch_all::<ResourceSubjectDetailResp>(
            &Query::select().columns(IamResourceSubject::iter().filter(|i| *i != IamResourceSubject::Table)).from(IamResourceSubject::Table).done(),
            None,
        )
        .await?;
    assert_eq!(records.len(), 3);
    assert!(records.iter().find(|x| x.ident_uri == format!("https://{}", iam_config.service_name)).is_some());
    assert!(records.iter().find(|x| x.name == format!("{}接口", iam_config.app.app_name)).is_some());

    let records = BIOSFuns::reldb()
        .fetch_all::<ResourceDetailResp>(
            &Query::select().columns(IamResource::iter().filter(|i| *i != IamResource::Table)).from(IamResource::Table).done(),
            None,
        )
        .await?;
    assert_eq!(records.len(), 3);
    assert!(records.iter().find(|x| x.path_and_query == "/console/system/**").is_some());

    let records = BIOSFuns::reldb()
        .fetch_all::<AuthPolicyDetailResp>(
            &Query::select().columns(IamAuthPolicy::iter().filter(|i| *i != IamAuthPolicy::Table)).from(IamAuthPolicy::Table).done(),
            None,
        )
        .await?;
    assert_eq!(records.len(), 15);

    let records = BIOSFuns::reldb()
        .fetch_all::<AuthPolicyObjectDetailResp>(
            &Query::select().columns(IamAuthPolicyObject::iter().filter(|i| *i != IamAuthPolicyObject::Table)).from(IamAuthPolicyObject::Table).done(),
            None,
        )
        .await?;
    assert_eq!(records.len(), 15);

    let aksk = BIOSFuns::cache().get(format!("{}{}", iam_config.cache.aksk, ak).as_str()).await?.unwrap();
    assert!(aksk.contains(&tenant_id));
    assert!(aksk.contains(&app_id));

    let auth_policies = BIOSFuns::cache().hgetall(&iam_config.cache.resources).await?;
    assert_eq!(auth_policies.len(), 15);

    Ok(())
}
