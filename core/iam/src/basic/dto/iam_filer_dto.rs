use std::default::Default;

use serde::{Deserialize, Serialize};
use tardis::web::poem_openapi::Object;

use bios_basic::rbum::dto::rbum_filer_dto::{RbumBasicFilterFetcher, RbumBasicFilterReq};

#[derive(Object, Serialize, Deserialize, Debug, Clone,Default)]
#[serde(default)]
pub struct IamAccountFilterReq {
    pub basic: RbumBasicFilterReq,
    pub icon: Option<String>,
    pub contact_phone: Option<String>,
}

impl RbumBasicFilterFetcher for IamAccountFilterReq {
    fn basic(&self) -> &RbumBasicFilterReq {
        &self.basic
    }
}

#[derive(Object, Serialize, Deserialize, Debug, Clone,Default)]
#[serde(default)]
pub struct IamAppFilterReq {
    pub basic: RbumBasicFilterReq,
    pub icon: Option<String>,
    pub sort: Option<u32>,
    pub contact_phone: Option<String>,
}

impl RbumBasicFilterFetcher for IamAppFilterReq {
    fn basic(&self) -> &RbumBasicFilterReq {
        &self.basic
    }
}

#[derive(Object, Serialize, Deserialize, Debug, Clone,Default)]
#[serde(default)]
pub struct IamTenantFilterReq {
    pub basic: RbumBasicFilterReq,
    pub icon: Option<String>,
    pub sort: Option<u32>,
    pub contact_phone: Option<String>,
}

impl RbumBasicFilterFetcher for IamTenantFilterReq {
    fn basic(&self) -> &RbumBasicFilterReq {
        &self.basic
    }
}

#[derive(Object, Serialize, Deserialize, Debug, Clone,Default)]
#[serde(default)]
pub struct IamHttpResFilterReq {
    pub basic: RbumBasicFilterReq,
    pub icon: Option<String>,
    pub sort: Option<u32>,
    pub method: Option<String>,
}

impl RbumBasicFilterFetcher for IamHttpResFilterReq {
    fn basic(&self) -> &RbumBasicFilterReq {
        &self.basic
    }
}

#[derive(Object, Serialize, Deserialize, Debug, Clone,Default)]
#[serde(default)]
pub struct IamRoleFilterReq {
    pub basic: RbumBasicFilterReq,
    pub icon: Option<String>,
    pub sort: Option<u32>,
}

impl RbumBasicFilterFetcher for IamRoleFilterReq {
    fn basic(&self) -> &RbumBasicFilterReq {
        &self.basic
    }
}
