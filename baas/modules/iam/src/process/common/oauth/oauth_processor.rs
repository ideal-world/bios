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

use actix_web::{post, HttpRequest};
use sea_query::{Cond, Expr, Query};

use bios::basic::dto::BIOSResp;
use bios::basic::dto::{BIOSContext, IdentInfo, Trace};
use bios::basic::error::BIOSError;
use bios::basic::result::BIOSResult;
use bios::db::reldb_client::SqlBuilderProcess;
use bios::web::basic_processor::extract_context;
use bios::web::resp_handler::BIOSResponse;
use bios::web::validate::json::Json;
use bios::BIOSFuns;

use crate::domain::auth_domain::{IamResource, IamResourceSubject};
use crate::domain::ident_domain::{IamAccount, IamAccountIdent, IamApp, IamTenant};
use crate::iam_constant::IamOutput;
use crate::process::basic_dto::{AccountIdentKind, CommonStatus, ResourceKind};
use crate::process::common::com_account_dto::{AccountLoginReq, AccountOAuthLoginReq, AccountRegisterReq};
use crate::process::common::common_processor;
use crate::process::common::oauth::platform_api::PlatformAPI;
use crate::process::common::oauth::wechat_xcx_api::WechatXcx;

static WECHAT_XCX_API: WechatXcx = WechatXcx {};

#[post("/common/oauth/login")]
pub async fn oauth_login(account_oauth_login_req: Json<AccountOAuthLoginReq>, req: HttpRequest) -> BIOSResponse {
    let context = extract_context(&req)?;
    let tenant_id = BIOSFuns::reldb()
        .fetch_one_json(
            &Query::select()
                .column((IamTenant::Table, IamTenant::Id))
                .from(IamApp::Table)
                .inner_join(IamTenant::Table, Expr::tbl(IamTenant::Table, IamTenant::Id).equals(IamApp::Table, IamApp::RelTenantId))
                .and_where(Expr::tbl(IamApp::Table, IamApp::Status).eq(CommonStatus::Enabled.to_string().to_lowercase()))
                .and_where(Expr::tbl(IamTenant::Table, IamTenant::Status).eq(CommonStatus::Enabled.to_string().to_lowercase()))
                .and_where(Expr::tbl(IamApp::Table, IamApp::Id).eq(account_oauth_login_req.rel_app_id.as_str()))
                .done(),
            None,
        )
        .await?;
    let context = BIOSContext {
        trace: Trace { ..context.trace },
        ident: IdentInfo {
            app_id: account_oauth_login_req.rel_app_id.to_string(),
            tenant_id: tenant_id["id"].as_str().unwrap().to_string(),
            ..context.ident
        },
        lang: context.lang,
    };
    log::info!(
        "Login : [{}] kind = {}, code = {}",
        account_oauth_login_req.rel_app_id,
        account_oauth_login_req.kind.to_string().to_lowercase(),
        account_oauth_login_req.code
    );
    let aksk_info = check_and_get_aksk(&account_oauth_login_req.kind, &context).await?;
    let platform_api = aksk_info.0;
    let ak = aksk_info.1;
    let sk = aksk_info.2;
    let user_info = platform_api.get_user_info(&account_oauth_login_req.code, &ak, &sk, &context).await?;
    let access_token = user_info.0;
    let user_info = user_info.1;
    let account_info = BIOSFuns::reldb()
        .fetch_optional_json(
            &Query::select()
                .column((IamAccount::Table, IamAccount::Id))
                .column((IamAccount::Table, IamAccount::Status))
                .from(IamAccountIdent::Table)
                .inner_join(
                    IamAccount::Table,
                    Expr::tbl(IamAccount::Table, IamAccount::Id).equals(IamAccountIdent::Table, IamAccountIdent::RelAccountId),
                )
                .and_where(Expr::tbl(IamAccountIdent::Table, IamAccountIdent::Kind).eq(account_oauth_login_req.kind.to_string().to_lowercase()))
                .and_where(Expr::tbl(IamAccountIdent::Table, IamAccountIdent::Ak).eq(user_info.account_open_id.as_str()))
                .and_where(Expr::tbl(IamAccountIdent::Table, IamAccountIdent::RelTenantId).lte(context.ident.tenant_id.as_str()))
                .done(),
            None,
        )
        .await?;
    if account_info.is_some() {
        let account_info = account_info.unwrap();
        let account_status = account_info["status"].as_str().unwrap();
        if account_status == CommonStatus::Disabled.to_string().to_lowercase() {
            return BIOSResp::err(IamOutput::CommonOAuthAccountDisabled(), Some(&context));
        }
        let ident_info = common_processor::login(
            &AccountLoginReq {
                kind: account_oauth_login_req.kind.clone(),
                ak: user_info.account_open_id,
                sk: access_token,
                cert_category: None,
                rel_app_id: context.ident.app_id.to_string(),
            },
            &context,
        )
        .await?;
        BIOSResp::ok(ident_info, Some(&context))
    } else {
        let ident_info = common_processor::register_account(
            &AccountRegisterReq {
                name: "".to_string(),
                avatar: None,
                parameters: None,
                kind: account_oauth_login_req.kind.clone(),
                ak: user_info.account_open_id,
                sk: access_token,
                rel_app_id: context.ident.app_id.to_string(),
            },
            &context,
        )
        .await?;
        BIOSResp::ok(ident_info, Some(&context))
    }
}

async fn check_and_get_aksk<'c>(ident_kind: &AccountIdentKind, context: &BIOSContext) -> BIOSResult<(&'c dyn PlatformAPI, String, String)> {
    let platform_api = match ident_kind {
        AccountIdentKind::WechatXcx => Some(&WECHAT_XCX_API),
        _ => None,
    };
    if platform_api.is_none() {
        return IamOutput::CommonOAuthFetchAccountIdentKindNotFound(ident_kind.to_string().to_lowercase())?;
    }
    let account_info = BIOSFuns::reldb()
        .fetch_optional_json(
            &Query::select()
                .column((IamResourceSubject::Table, IamResourceSubject::Ak))
                .column((IamResourceSubject::Table, IamResourceSubject::Sk))
                .from(IamResource::Table)
                .inner_join(
                    IamResourceSubject::Table,
                    Expr::tbl(IamResourceSubject::Table, IamResourceSubject::Id).equals(IamResource::Table, IamResource::RelResourceSubjectId),
                )
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
                .and_where(Expr::tbl(IamResourceSubject::Table, IamResourceSubject::Kind).eq(ResourceKind::OAuth.to_string().to_lowercase()))
                .and_where(Expr::tbl(IamResourceSubject::Table, IamResourceSubject::IdentUri).eq(format!(
                    "https://{}/common/oauth/{}/{}",
                    BIOSFuns::fw_config().app.id,
                    ident_kind.to_string().to_lowercase(),
                    context.ident.app_id.as_str()
                )))
                .done(),
            None,
        )
        .await?;
    if account_info.is_none() {
        return IamOutput::CommonOAuthFetchResourceSubjectNotFound()?;
    }
    let account_info = account_info.unwrap();
    let ak = account_info["ak"].as_str().unwrap();
    let sk = account_info["sk"].as_str().unwrap();
    Ok((platform_api.unwrap(), ak.to_string(), sk.to_string()))
}
