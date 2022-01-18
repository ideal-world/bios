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

use bios::basic::error::BIOSError;
use derive_more::Display;

use bios::basic::result::StatusCodeKind;
use bios::basic::result::{ActionKind, BIOSResult};

static APP_FLAG: &str = "01";

#[derive(Display, Debug)]
enum ModuleKind {
    #[display(fmt = "01")]
    Common,
    #[display(fmt = "02")]
    AppConsole,
    #[display(fmt = "03")]
    SystemConsole,
    #[display(fmt = "04")]
    TenantConsole,
    #[display(fmt = "05")]
    OAuth,
}

#[derive(Display, Debug)]
pub enum ObjectKind {
    #[display(fmt = "001")]
    Tenant,
    #[display(fmt = "002")]
    TenantCert,
    #[display(fmt = "003")]
    TenantIdent,
    #[display(fmt = "004")]
    App,
    #[display(fmt = "005")]
    AppIdent,
    #[display(fmt = "006")]
    Role,
    #[display(fmt = "007")]
    Group,
    #[display(fmt = "008")]
    GroupNode,
    #[display(fmt = "009")]
    Account,
    #[display(fmt = "010")]
    AccountIdent,
    #[display(fmt = "011")]
    AccountApp,
    #[display(fmt = "012")]
    AccountBind,
    #[display(fmt = "013")]
    AccountRole,
    #[display(fmt = "014")]
    AccountGroup,
    #[display(fmt = "015")]
    ResourceSubject,
    #[display(fmt = "016")]
    Resource,
    #[display(fmt = "017")]
    AuthPolicy,
    #[display(fmt = "018")]
    AuthPolicyObject,
    #[display(fmt = "101")]
    Token,
    #[display(fmt = "102")]
    AccessToken,
    #[display(fmt = "103")]
    OAuthInfo,
}

/// 统一错误输出
///
/// 错误码格式：错误状态码 APP标识 模块标识 操作主体 操作标识
#[derive(Display, Debug)]
pub enum IamOutput {
    // -------------------- App Console --------------------
    #[display(fmt = "{}{}{}{}{}##[{}] not found", StatusCodeKind::NotFound, APP_FLAG, ModuleKind::AppConsole, _0, ActionKind::Create, _1)]
    AppConsoleEntityCreateCheckNotFound(ObjectKind, &'static str),
    #[display(fmt = "{}{}{}{}{}##[{}] not found", StatusCodeKind::NotFound, APP_FLAG, ModuleKind::AppConsole, _0, ActionKind::Create, _1)]
    AppConsoleEntityCreateCheckNotFoundField(ObjectKind, &'static str),
    #[display(
        fmt = "{}{}{}{}{}##[{}] already exists",
        StatusCodeKind::ConflictExists,
        APP_FLAG,
        ModuleKind::AppConsole,
        _0,
        ActionKind::Create,
        _1
    )]
    AppConsoleEntityCreateCheckExists(ObjectKind, &'static str),
    #[display(
        fmt = "{}{}{}{}{}##{} must exist at the same time",
        StatusCodeKind::ConflictExistFieldsAtSomeTime,
        APP_FLAG,
        ModuleKind::AppConsole,
        _0,
        ActionKind::Create,
        _1
    )]
    AppConsoleEntityCreateCheckMustExistAtSomeTime(ObjectKind, &'static str),

    #[display(fmt = "{}{}{}{}{}##[{}] not found", StatusCodeKind::NotFound, APP_FLAG, ModuleKind::AppConsole, ActionKind::Modify, _0, _1)]
    AppConsoleEntityModifyCheckNotFound(ObjectKind, &'static str),
    #[display(
        fmt = "{}{}{}{}{}##[{}] already exists",
        StatusCodeKind::ConflictExists,
        APP_FLAG,
        ModuleKind::AppConsole,
        _0,
        ActionKind::Modify,
        _1
    )]
    AppConsoleEntityModifyCheckExists(ObjectKind, &'static str),
    #[display(
        fmt = "{}{}{}{}{}##{} must exist at the same time",
        StatusCodeKind::ConflictExistFieldsAtSomeTime,
        APP_FLAG,
        ModuleKind::AppConsole,
        _0,
        ActionKind::Modify,
        _1
    )]
    AppConsoleEntityModifyCheckExistFieldsAtSomeTime(ObjectKind, &'static str),

    #[display(fmt = "{}{}{}{}{}##[{}] not found", StatusCodeKind::NotFound, APP_FLAG, ModuleKind::AppConsole, ActionKind::FetchList, _0, _1)]
    AppConsoleEntityFetchListCheckNotFound(ObjectKind, &'static str),
    #[display(fmt = "{}{}{}{}{}##[{}] not found", StatusCodeKind::NotFound, APP_FLAG, ModuleKind::AppConsole, ActionKind::Delete, _0, _1)]
    AppConsoleEntityDeleteCheckNotFound(ObjectKind, &'static str),
    #[display(
        fmt = "{}{}{}{}{}##Please delete the associated [{}] data first",
        StatusCodeKind::ConflictExistAssociatedData,
        APP_FLAG,
        ModuleKind::AppConsole,
        _0,
        ActionKind::Delete,
        _1
    )]
    AppConsoleEntityDeleteCheckExistAssociatedData(ObjectKind, &'static str),

    // -------------------- System Console --------------------
    #[display(fmt = "{}{}{}{}{}##[{}] not found", StatusCodeKind::NotFound, APP_FLAG, ModuleKind::SystemConsole, ActionKind::Create, _0, _1)]
    SystemConsoleEntityCreateCheckNotFound(ObjectKind, &'static str),
    #[display(fmt = "{}{}{}{}{}##[{}] not found", StatusCodeKind::NotFound, APP_FLAG, ModuleKind::SystemConsole, ActionKind::Create, _0, _1)]
    SystemConsoleEntityCreateCheckNotFoundField(ObjectKind, &'static str),
    #[display(
        fmt = "{}{}{}{}{}##[{}] already exists",
        StatusCodeKind::ConflictExists,
        APP_FLAG,
        ModuleKind::SystemConsole,
        _0,
        ActionKind::Create,
        _1
    )]
    SystemConsoleEntityCreateCheckExists(ObjectKind, &'static str),
    #[display(
        fmt = "{}{}{}{}{}##{} must exist at the same time",
        StatusCodeKind::ConflictExistFieldsAtSomeTime,
        APP_FLAG,
        ModuleKind::SystemConsole,
        _0,
        ActionKind::Create,
        _1
    )]
    SystemConsoleEntityCreateCheckMustExistAtSomeTime(ObjectKind, &'static str),

    #[display(fmt = "{}{}{}{}{}##[{}] not found", StatusCodeKind::NotFound, APP_FLAG, ModuleKind::SystemConsole, ActionKind::Modify, _0, _1)]
    SystemConsoleEntityModifyCheckNotFound(ObjectKind, &'static str),
    #[display(
        fmt = "{}{}{}{}{}##[{}] already exists",
        StatusCodeKind::ConflictExists,
        APP_FLAG,
        ModuleKind::SystemConsole,
        _0,
        ActionKind::Modify,
        _1
    )]
    SystemConsoleEntityModifyCheckExists(ObjectKind, &'static str),
    #[display(
        fmt = "{}{}{}{}{}##{} must exist at the same time",
        StatusCodeKind::ConflictExistFieldsAtSomeTime,
        APP_FLAG,
        ModuleKind::SystemConsole,
        _0,
        ActionKind::Modify,
        _1
    )]
    SystemConsoleEntityModifyCheckExistFieldsAtSomeTime(ObjectKind, &'static str),

    #[display(
        fmt = "{}{}{}{}{}##[{}] not found",
        StatusCodeKind::NotFound,
        APP_FLAG,
        ModuleKind::SystemConsole,
        _0,
        ActionKind::FetchList,
        _1
    )]
    SystemConsoleEntityFetchListCheckNotFound(ObjectKind, &'static str),
    #[display(fmt = "{}{}{}{}{}##[{}] not found", StatusCodeKind::NotFound, APP_FLAG, ModuleKind::SystemConsole, ActionKind::Delete, _0, _1)]
    SystemConsoleEntityDeleteCheckNotFound(ObjectKind, &'static str),
    #[display(
        fmt = "{}{}{}{}{}##Please delete the associated [{}] data first",
        StatusCodeKind::ConflictExistAssociatedData,
        APP_FLAG,
        ModuleKind::SystemConsole,
        _0,
        ActionKind::Delete,
        _1
    )]
    SystemConsoleEntityDeleteCheckExistAssociatedData(ObjectKind, &'static str),

    // -------------------- Tenant Console --------------------
    #[display(fmt = "{}{}{}{}{}##[{}] not found", StatusCodeKind::NotFound, APP_FLAG, ModuleKind::TenantConsole, ActionKind::Create, _0, _1)]
    TenantConsoleEntityCreateCheckNotFound(ObjectKind, &'static str),
    #[display(fmt = "{}{}{}{}{}##[{}] not found", StatusCodeKind::NotFound, APP_FLAG, ModuleKind::TenantConsole, ActionKind::Create, _0, _1)]
    TenantConsoleEntityCreateCheckNotFoundField(ObjectKind, &'static str),
    #[display(
        fmt = "{}{}{}{}{}##[{}] already exists",
        StatusCodeKind::ConflictExists,
        APP_FLAG,
        ModuleKind::TenantConsole,
        _0,
        ActionKind::Create,
        _1
    )]
    TenantConsoleEntityCreateCheckExists(ObjectKind, &'static str),
    #[display(
        fmt = "{}{}{}{}{}##{} must exist at the same time",
        StatusCodeKind::ConflictExistFieldsAtSomeTime,
        APP_FLAG,
        ModuleKind::TenantConsole,
        _0,
        ActionKind::Create,
        _1
    )]
    TenantConsoleEntityCreateCheckMustExistAtSomeTime(ObjectKind, &'static str),

    #[display(fmt = "{}{}{}{}{}##[{}] not found", StatusCodeKind::NotFound, APP_FLAG, ModuleKind::TenantConsole, ActionKind::Modify, _0, _1)]
    TenantConsoleEntityModifyCheckNotFound(ObjectKind, &'static str),
    #[display(
        fmt = "{}{}{}{}{}##[{}] already exists",
        StatusCodeKind::ConflictExists,
        APP_FLAG,
        ModuleKind::TenantConsole,
        _0,
        ActionKind::Modify,
        _1
    )]
    TenantConsoleEntityModifyCheckExists(ObjectKind, &'static str),
    #[display(
        fmt = "{}{}{}{}{}##{} must exist at the same time",
        StatusCodeKind::ConflictExistFieldsAtSomeTime,
        APP_FLAG,
        ModuleKind::TenantConsole,
        _0,
        ActionKind::Modify,
        _1
    )]
    TenantConsoleEntityModifyCheckExistFieldsAtSomeTime(ObjectKind, &'static str),

    #[display(
        fmt = "{}{}{}{}{}##[{}] not found",
        StatusCodeKind::NotFound,
        APP_FLAG,
        ModuleKind::TenantConsole,
        _0,
        ActionKind::FetchList,
        _1
    )]
    TenantConsoleEntityFetchListCheckNotFound(ObjectKind, &'static str),
    #[display(fmt = "{}{}{}{}{}##[{}] not found", StatusCodeKind::NotFound, APP_FLAG, ModuleKind::TenantConsole, ActionKind::Delete, _0, _1)]
    TenantConsoleEntityDeleteCheckNotFound(ObjectKind, &'static str),
    #[display(
        fmt = "{}{}{}{}{}##Please delete the associated [{}] data first",
        StatusCodeKind::ConflictExistAssociatedData,
        APP_FLAG,
        ModuleKind::TenantConsole,
        _0,
        ActionKind::Delete,
        _1
    )]
    TenantConsoleEntityDeleteCheckExistAssociatedData(ObjectKind, &'static str),

    // -------------------- Common --------------------
    #[display(fmt = "{}{}{}{}{}##[{}] not found", StatusCodeKind::NotFound, APP_FLAG, ModuleKind::Common, ActionKind::Create, _0, _1)]
    CommonEntityCreateCheckNotFound(ObjectKind, &'static str),
    #[display(fmt = "{}{}{}{}{}##[{}] not found", StatusCodeKind::NotFound, APP_FLAG, ModuleKind::Common, ActionKind::Create, _0, _1)]
    CommonEntityCreateCheckNotFoundField(ObjectKind, &'static str),
    #[display(
        fmt = "{}{}{}{}{}##[{}] already exists",
        StatusCodeKind::ConflictExists,
        APP_FLAG,
        ModuleKind::Common,
        _0,
        ActionKind::Create,
        _1
    )]
    CommonEntityCreateCheckExists(ObjectKind, &'static str),
    #[display(
        fmt = "{}{}{}{}{}##{} must exist at the same time",
        StatusCodeKind::ConflictExistFieldsAtSomeTime,
        APP_FLAG,
        ModuleKind::Common,
        _0,
        ActionKind::Create,
        _1
    )]
    CommonEntityCreateCheckMustExistAtSomeTime(ObjectKind, &'static str),

    #[display(fmt = "{}{}{}{}{}##[{}] not found", StatusCodeKind::NotFound, APP_FLAG, ModuleKind::Common, ActionKind::Modify, _0, _1)]
    CommonEntityModifyCheckNotFound(ObjectKind, &'static str),
    #[display(
        fmt = "{}{}{}{}{}##[{}] already exists",
        StatusCodeKind::ConflictExists,
        APP_FLAG,
        ModuleKind::Common,
        _0,
        ActionKind::Modify,
        _1
    )]
    CommonEntityModifyCheckExists(ObjectKind, &'static str),
    #[display(
        fmt = "{}{}{}{}{}##{} must exist at the same time",
        StatusCodeKind::ConflictExistFieldsAtSomeTime,
        APP_FLAG,
        ModuleKind::Common,
        _0,
        ActionKind::Modify,
        _1
    )]
    CommonEntityModifyCheckExistFieldsAtSomeTime(ObjectKind, &'static str),

    #[display(fmt = "{}{}{}{}{}##[{}] not found", StatusCodeKind::NotFound, APP_FLAG, ModuleKind::Common, ActionKind::FetchList, _0, _1)]
    CommonEntityFetchListCheckNotFound(ObjectKind, &'static str),
    #[display(fmt = "{}{}{}{}{}##[{}] not found", StatusCodeKind::NotFound, APP_FLAG, ModuleKind::Common, ActionKind::Delete, _0, _1)]
    CommonEntityDeleteCheckNotFound(ObjectKind, &'static str),
    #[display(
        fmt = "{}{}{}{}{}##Please delete the associated [{}] data first",
        StatusCodeKind::ConflictExistAssociatedData,
        APP_FLAG,
        ModuleKind::Common,
        _0,
        ActionKind::Delete,
        _1
    )]
    CommonEntityDeleteCheckExistAssociatedData(ObjectKind, &'static str),

    #[display(
        fmt = "{}{}{}{}50##AccountIdent [{}] doesn't exist or has expired",
        StatusCodeKind::NotFound,
        APP_FLAG,
        ModuleKind::Common,
        ObjectKind::AccountIdent,
        _0
    )]
    CommonLoginCheckAccountNotFoundOrExpired(String),
    #[display(
        fmt = "{}{}{}{}51##AccountIdent [kind] not exists",
        StatusCodeKind::NotFound,
        APP_FLAG,
        ModuleKind::Common,
        ObjectKind::AccountIdent
    )]
    CommonAccountIdentValidCheckNotFound(),
    #[display(
        fmt = "{}{}{}{}52##AccountIdent [{}] invalid format",
        StatusCodeKind::BadRequest,
        APP_FLAG,
        ModuleKind::Common,
        ObjectKind::AccountIdent,
        _0
    )]
    CommonAccountIdentValidCheckInvalidFormat(&'static str),
    #[display(
        fmt = "{}{}{}{}53##Verification code [{}] over the maximum times",
        StatusCodeKind::Conflict,
        APP_FLAG,
        ModuleKind::Common,
        ObjectKind::AccountIdent,
        _0
    )]
    CommonAccountIdentValidCheckVCodeOverMaxTimes(String),
    #[display(
        fmt = "{}{}{}{}54##Verification code [{}] doesn't exist or has expired",
        StatusCodeKind::Conflict,
        APP_FLAG,
        ModuleKind::Common,
        ObjectKind::AccountIdent,
        _0
    )]
    CommonAccountIdentValidCheckInvalidVCodeNotFoundOrExpired(String),
    #[display(
        fmt = "{}{}{}{}55##Username [{}] or Password error",
        StatusCodeKind::Conflict,
        APP_FLAG,
        ModuleKind::Common,
        ObjectKind::AccountIdent,
        _0
    )]
    CommonAccountIdentValidCheckUserOrPasswordError(String),
    #[display(
        fmt = "{}{}{}{}56##Password can't be empty",
        StatusCodeKind::BadRequest,
        APP_FLAG,
        ModuleKind::Common,
        ObjectKind::AccountIdent
    )]
    CommonAccountIdentValidCheckPasswordNotEmpty(),
    #[display(
        fmt = "{}{}{}{}57##Account Token [{}] doesn't exist or has expired",
        StatusCodeKind::Conflict,
        APP_FLAG,
        ModuleKind::Common,
        ObjectKind::AccountIdent,
        _0
    )]
    CommonAccountIdentValidCheckInvalidAccessTokenNotFoundOrExpired(String),
    #[display(
        fmt = "{}{}{}{}58##Unsupported authentication kind [{}]",
        StatusCodeKind::NotFound,
        APP_FLAG,
        ModuleKind::Common,
        ObjectKind::AccountIdent,
        _0
    )]
    CommonAccountIdentValidCheckUnsupportedAuthKind(String),

    #[display(
        fmt = "{}{}{}{}{}##[{}] not found",
        StatusCodeKind::NotFound,
        APP_FLAG,
        ModuleKind::OAuth,
        ObjectKind::AccountIdent,
        ActionKind::FetchOne,
        _0
    )]
    CommonOAuthFetchAccountIdentKindNotFound(String),
    #[display(
        fmt = "{}{}{}{}{}##[ResourceSubject] not found",
        StatusCodeKind::NotFound,
        APP_FLAG,
        ModuleKind::OAuth,
        ObjectKind::ResourceSubject,
        ActionKind::FetchOne
    )]
    CommonOAuthFetchResourceSubjectNotFound(),
    #[display(
        fmt = "{}{}{}{}{}##Account Status Disabled",
        StatusCodeKind::Conflict,
        APP_FLAG,
        ModuleKind::OAuth,
        ObjectKind::Account,
        ActionKind::FetchOne
    )]
    CommonOAuthAccountDisabled(),
    #[display(
        fmt = "{}{}{}{}{}##[{}] {}",
        StatusCodeKind::UnKnown,
        APP_FLAG,
        ModuleKind::OAuth,
        ObjectKind::OAuthInfo,
        ActionKind::FetchOne,
        _0,
        _1
    )]
    CommonOAuthFetchOAuthInfoError(String, String),
    #[display(
        fmt = "{}{}{}{}{}##[{}] {}",
        StatusCodeKind::Success,
        APP_FLAG,
        ModuleKind::OAuth,
        ObjectKind::OAuthInfo,
        ActionKind::FetchOne,
        _0,
        _1
    )]
    CommonOAuthFetchOAuthInfoTrace(String, String),
    #[display(
        fmt = "{}{}{}{}{}##[{}] {}",
        StatusCodeKind::UnKnown,
        APP_FLAG,
        ModuleKind::OAuth,
        ObjectKind::AccessToken,
        ActionKind::FetchOne,
        _0,
        _1
    )]
    CommonOAuthFetchAccessTokenError(String, String),
    #[display(
        fmt = "{}{}{}{}{}##[{}] {}",
        StatusCodeKind::Success,
        APP_FLAG,
        ModuleKind::OAuth,
        ObjectKind::AccessToken,
        ActionKind::FetchOne,
        _0,
        _1
    )]
    CommonOAuthFetchAccessTokenTrace(String, String),
}

impl<T> From<IamOutput> for BIOSResult<T> {
    fn from(output: IamOutput) -> Self {
        Err(BIOSError::_Inner(output.to_string()))
    }
}
