use serde::{Deserialize, Serialize};
use tardis::basic::field::TrimString;
use tardis::chrono::{DateTime, Utc};
use tardis::db::sea_orm::FromQueryResult;
use tardis::web::poem_openapi::Object;

use bios_basic::rbum::enumeration::RbumScopeKind;

#[derive(Serialize, Deserialize, Debug)]
pub struct IamAccountAddReq {
    pub code: Option<TrimString>,
    pub name: TrimString,
    pub icon: Option<String>,

    pub scope_kind: Option<RbumScopeKind>,
    pub disabled: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct IamAccountModifyReq {
    pub name: Option<TrimString>,
    pub icon: Option<String>,

    pub scope_kind: Option<RbumScopeKind>,
    pub disabled: Option<bool>,
}

#[derive(Object, FromQueryResult, Serialize, Deserialize, Debug)]
pub struct IamAccountSummaryResp {
    pub id: String,
    pub name: String,
    pub icon: String,

    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,

    pub scope_kind: String,
    pub disabled: bool,
}

#[derive(Object, FromQueryResult, Serialize, Deserialize, Debug)]
pub struct IamAccountDetailResp {
    pub id: String,
    pub name: String,
    pub icon: String,

    pub updater_code: String,
    pub updater_name: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,

    pub scope_kind: String,
    pub disabled: bool,
}
