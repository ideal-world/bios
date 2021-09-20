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

use derive_more::Display;

use bios::basic::result::ActionKind;
use bios::basic::result::StatusCodeKind;

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
}

#[derive(Display, Debug)]
pub enum IamOutput {
    // -------------------- App Console --------------------
    #[display(fmt = "{}{}{}{}{}##[{}] not found", StatusCodeKind::NotFound, APP_FLAG, ModuleKind::AppConsole, ActionKind::Create, _0, _1)]
    AppConsoleEntityCreateCheckNotFound(ObjectKind, ObjectKind),
    #[display(fmt = "{}{}{}{}{}##[{}] not found", StatusCodeKind::NotFound, APP_FLAG, ModuleKind::AppConsole, ActionKind::Create, _0, _1)]
    AppConsoleEntityCreateCheckNotFoundField(ObjectKind, &'static str),
    #[display(
        fmt = "{}{}{}{}{}##[{}] already exists",
        StatusCodeKind::ConflictExists,
        APP_FLAG,
        ModuleKind::AppConsole,
        ActionKind::Create,
        _0,
        _1
    )]
    AppConsoleEntityCreateCheckExists(ObjectKind, ObjectKind),
    #[display(
        fmt = "{}{}{}{}{}##[{}] already exists",
        StatusCodeKind::ConflictExists,
        APP_FLAG,
        ModuleKind::AppConsole,
        ActionKind::Create,
        _0,
        _1
    )]
    AppConsoleEntityCreateCheckMustExistAtSomeTime(ObjectKind, &'static str),

    #[display(fmt = "{}{}{}{}{}##[{}] not found", StatusCodeKind::NotFound, APP_FLAG, ModuleKind::AppConsole, ActionKind::Modify, _0, _1)]
    AppConsoleEntityModifyCheckNotFound(ObjectKind, ObjectKind),
    #[display(
        fmt = "{}{}{}{}{}##[{}] already exists",
        StatusCodeKind::ConflictExists,
        APP_FLAG,
        ModuleKind::AppConsole,
        ActionKind::Modify,
        _0,
        _1
    )]
    AppConsoleEntityModifyCheckExists(ObjectKind, ObjectKind),
    #[display(
        fmt = "{}{}{}{}{}##{} must exist at the same time",
        StatusCodeKind::ConflictExistFieldsAtSomeTime,
        APP_FLAG,
        ModuleKind::AppConsole,
        ActionKind::Modify,
        _0,
        _1
    )]
    AppConsoleEntityModifyCheckExistFieldsAtSomeTime(ObjectKind, &'static str),

    #[display(fmt = "{}{}{}{}{}##[{}] not found", StatusCodeKind::NotFound, APP_FLAG, ModuleKind::AppConsole, ActionKind::FetchList, _0, _1)]
    AppConsoleEntityFetchListCheckNotFound(ObjectKind, ObjectKind),
    #[display(fmt = "{}{}{}{}{}##[{}] not found", StatusCodeKind::NotFound, APP_FLAG, ModuleKind::AppConsole, ActionKind::Delete, _0, _1)]
    AppConsoleEntityDeleteCheckNotFound(ObjectKind, ObjectKind),
    #[display(
        fmt = "{}{}{}{}{}##Please delete the associated [{}] data first",
        StatusCodeKind::ConflictExistAssociatedData,
        APP_FLAG,
        ModuleKind::AppConsole,
        ActionKind::Delete,
        _0,
        _1
    )]
    AppConsoleEntityDeleteCheckExistAssociatedData(ObjectKind, ObjectKind),

    // -------------------- System Console --------------------
    #[display(fmt = "{}{}{}{}{}##[{}] not found", StatusCodeKind::NotFound, APP_FLAG, ModuleKind::SystemConsole, ActionKind::Create, _0, _1)]
    SystemConsoleEntityCreateCheckNotFound(ObjectKind, ObjectKind),
    #[display(fmt = "{}{}{}{}{}##[{}] not found", StatusCodeKind::NotFound, APP_FLAG, ModuleKind::SystemConsole, ActionKind::Create, _0, _1)]
    SystemConsoleEntityCreateCheckNotFoundField(ObjectKind, &'static str),
    #[display(
        fmt = "{}{}{}{}{}##[{}] already exists",
        StatusCodeKind::ConflictExists,
        APP_FLAG,
        ModuleKind::SystemConsole,
        ActionKind::Create,
        _0,
        _1
    )]
    SystemConsoleEntityCreateCheckExists(ObjectKind, ObjectKind),
    #[display(
        fmt = "{}{}{}{}{}##[{}] already exists",
        StatusCodeKind::ConflictExists,
        APP_FLAG,
        ModuleKind::SystemConsole,
        ActionKind::Create,
        _0,
        _1
    )]
    SystemConsoleEntityCreateCheckMustExistAtSomeTime(ObjectKind, &'static str),

    #[display(fmt = "{}{}{}{}{}##[{}] not found", StatusCodeKind::NotFound, APP_FLAG, ModuleKind::SystemConsole, ActionKind::Modify, _0, _1)]
    SystemConsoleEntityModifyCheckNotFound(ObjectKind, ObjectKind),
    #[display(
        fmt = "{}{}{}{}{}##[{}] already exists",
        StatusCodeKind::ConflictExists,
        APP_FLAG,
        ModuleKind::SystemConsole,
        ActionKind::Modify,
        _0,
        _1
    )]
    SystemConsoleEntityModifyCheckExists(ObjectKind, ObjectKind),
    #[display(
        fmt = "{}{}{}{}{}##{} must exist at the same time",
        StatusCodeKind::ConflictExistFieldsAtSomeTime,
        APP_FLAG,
        ModuleKind::SystemConsole,
        ActionKind::Modify,
        _0,
        _1
    )]
    SystemConsoleEntityModifyCheckExistFieldsAtSomeTime(ObjectKind, &'static str),

    #[display(
        fmt = "{}{}{}{}{}##[{}] not found",
        StatusCodeKind::NotFound,
        APP_FLAG,
        ModuleKind::SystemConsole,
        ActionKind::FetchList,
        _0,
        _1
    )]
    SystemConsoleEntityFetchListCheckNotFound(ObjectKind, ObjectKind),
    #[display(fmt = "{}{}{}{}{}##[{}] not found", StatusCodeKind::NotFound, APP_FLAG, ModuleKind::SystemConsole, ActionKind::Delete, _0, _1)]
    SystemConsoleEntityDeleteCheckNotFound(ObjectKind, ObjectKind),
    #[display(
        fmt = "{}{}{}{}{}##Please delete the associated [{}] data first",
        StatusCodeKind::ConflictExistAssociatedData,
        APP_FLAG,
        ModuleKind::SystemConsole,
        ActionKind::Delete,
        _0,
        _1
    )]
    SystemConsoleEntityDeleteCheckExistAssociatedData(ObjectKind, ObjectKind),

    // -------------------- Tenant Console --------------------
    #[display(fmt = "{}{}{}{}{}##[{}] not found", StatusCodeKind::NotFound, APP_FLAG, ModuleKind::TenantConsole, ActionKind::Create, _0, _1)]
    TenantConsoleEntityCreateCheckNotFound(ObjectKind, ObjectKind),
    #[display(fmt = "{}{}{}{}{}##[{}] not found", StatusCodeKind::NotFound, APP_FLAG, ModuleKind::TenantConsole, ActionKind::Create, _0, _1)]
    TenantConsoleEntityCreateCheckNotFoundField(ObjectKind, &'static str),
    #[display(
        fmt = "{}{}{}{}{}##[{}] already exists",
        StatusCodeKind::ConflictExists,
        APP_FLAG,
        ModuleKind::TenantConsole,
        ActionKind::Create,
        _0,
        _1
    )]
    TenantConsoleEntityCreateCheckExists(ObjectKind, ObjectKind),
    #[display(
        fmt = "{}{}{}{}{}##[{}] already exists",
        StatusCodeKind::ConflictExists,
        APP_FLAG,
        ModuleKind::TenantConsole,
        ActionKind::Create,
        _0,
        _1
    )]
    TenantConsoleEntityCreateCheckMustExistAtSomeTime(ObjectKind, &'static str),

    #[display(fmt = "{}{}{}{}{}##[{}] not found", StatusCodeKind::NotFound, APP_FLAG, ModuleKind::TenantConsole, ActionKind::Modify, _0, _1)]
    TenantConsoleEntityModifyCheckNotFound(ObjectKind, ObjectKind),
    #[display(
        fmt = "{}{}{}{}{}##[{}] already exists",
        StatusCodeKind::ConflictExists,
        APP_FLAG,
        ModuleKind::TenantConsole,
        ActionKind::Modify,
        _0,
        _1
    )]
    TenantConsoleEntityModifyCheckExists(ObjectKind, ObjectKind),
    #[display(
        fmt = "{}{}{}{}{}##{} must exist at the same time",
        StatusCodeKind::ConflictExistFieldsAtSomeTime,
        APP_FLAG,
        ModuleKind::TenantConsole,
        ActionKind::Modify,
        _0,
        _1
    )]
    TenantConsoleEntityModifyCheckExistFieldsAtSomeTime(ObjectKind, &'static str),

    #[display(
        fmt = "{}{}{}{}{}##[{}] not found",
        StatusCodeKind::NotFound,
        APP_FLAG,
        ModuleKind::TenantConsole,
        ActionKind::FetchList,
        _0,
        _1
    )]
    TenantConsoleEntityFetchListCheckNotFound(ObjectKind, ObjectKind),
    #[display(fmt = "{}{}{}{}{}##[{}] not found", StatusCodeKind::NotFound, APP_FLAG, ModuleKind::TenantConsole, ActionKind::Delete, _0, _1)]
    TenantConsoleEntityDeleteCheckNotFound(ObjectKind, ObjectKind),
    #[display(
        fmt = "{}{}{}{}{}##Please delete the associated [{}] data first",
        StatusCodeKind::ConflictExistAssociatedData,
        APP_FLAG,
        ModuleKind::TenantConsole,
        ActionKind::Delete,
        _0,
        _1
    )]
    TenantConsoleEntityDeleteCheckExistAssociatedData(ObjectKind, ObjectKind),

    // -------------------- Common --------------------
    #[display(fmt = "{}{}{}{}{}##[{}] not found", StatusCodeKind::NotFound, APP_FLAG, ModuleKind::Common, ActionKind::Create, _0, _1)]
    CommonEntityCreateCheckNotFound(ObjectKind, ObjectKind),
    #[display(fmt = "{}{}{}{}{}##[{}] not found", StatusCodeKind::NotFound, APP_FLAG, ModuleKind::Common, ActionKind::Create, _0, _1)]
    CommonEntityCreateCheckNotFoundField(ObjectKind, &'static str),
    #[display(
        fmt = "{}{}{}{}{}##[{}] already exists",
        StatusCodeKind::ConflictExists,
        APP_FLAG,
        ModuleKind::Common,
        ActionKind::Create,
        _0,
        _1
    )]
    CommonEntityCreateCheckExists(ObjectKind, ObjectKind),
    #[display(
        fmt = "{}{}{}{}{}##[{}] already exists",
        StatusCodeKind::ConflictExists,
        APP_FLAG,
        ModuleKind::Common,
        ActionKind::Create,
        _0,
        _1
    )]
    CommonEntityCreateCheckMustExistAtSomeTime(ObjectKind, &'static str),

    #[display(fmt = "{}{}{}{}{}##[{}] not found", StatusCodeKind::NotFound, APP_FLAG, ModuleKind::Common, ActionKind::Modify, _0, _1)]
    CommonEntityModifyCheckNotFound(ObjectKind, ObjectKind),
    #[display(
        fmt = "{}{}{}{}{}##[{}] already exists",
        StatusCodeKind::ConflictExists,
        APP_FLAG,
        ModuleKind::Common,
        ActionKind::Modify,
        _0,
        _1
    )]
    CommonEntityModifyCheckExists(ObjectKind, ObjectKind),
    #[display(
        fmt = "{}{}{}{}{}##{} must exist at the same time",
        StatusCodeKind::ConflictExistFieldsAtSomeTime,
        APP_FLAG,
        ModuleKind::Common,
        ActionKind::Modify,
        _0,
        _1
    )]
    CommonEntityModifyCheckExistFieldsAtSomeTime(ObjectKind, &'static str),

    #[display(fmt = "{}{}{}{}{}##[{}] not found", StatusCodeKind::NotFound, APP_FLAG, ModuleKind::Common, ActionKind::FetchList, _0, _1)]
    CommonEntityFetchListCheckNotFound(ObjectKind, ObjectKind),
    #[display(fmt = "{}{}{}{}{}##[{}] not found", StatusCodeKind::NotFound, APP_FLAG, ModuleKind::Common, ActionKind::Delete, _0, _1)]
    CommonEntityDeleteCheckNotFound(ObjectKind, ObjectKind),
    #[display(
        fmt = "{}{}{}{}{}##Please delete the associated [{}] data first",
        StatusCodeKind::ConflictExistAssociatedData,
        APP_FLAG,
        ModuleKind::Common,
        ActionKind::Delete,
        _0,
        _1
    )]
    CommonEntityDeleteCheckExistAssociatedData(ObjectKind, ObjectKind),

    #[display(fmt = "{}{}{}50{}##Account doesn't exist or has expired", StatusCodeKind::NotFound, APP_FLAG, ModuleKind::Common, _0)]
    CommonLoginCheckNotFoundOrExpired(ObjectKind),
    #[display(
        fmt = "{}{}{}51{}##AccountIdent [kind] not exists",
        StatusCodeKind::NotFound,
        APP_FLAG,
        ModuleKind::Common,
        ObjectKind::AccountIdent
    )]
    CommonAccountIdentValidCheckNotFound(),
    #[display(
        fmt = "{}{}{}51{}##AccountIdent [{}] invalid format",
        StatusCodeKind::BadRequest,
        APP_FLAG,
        ModuleKind::Common,
        ObjectKind::AccountIdent,
        _0
    )]
    CommonAccountIdentValidCheckInvalidFormat(&'static str),
}
