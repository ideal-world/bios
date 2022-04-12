use serde::{Deserialize, Serialize};
use tardis::web::poem_openapi::Object;

use bios_basic::rbum::dto::rbum_filer_dto::{RbumBasicFilterFetcher, RbumBasicFilterReq};

#[derive(Object, Serialize, Deserialize, Debug, Clone)]
#[serde(default)]
pub struct IamAccountFilterReq {
    pub basic: RbumBasicFilterReq,
    pub icon: Option<String>,
    pub contact_phone: Option<String>,
}

impl Default for IamAccountFilterReq {
    fn default() -> Self {
        Self {
            basic: Default::default(),
            icon: None,
            contact_phone: None,
        }
    }
}

impl RbumBasicFilterFetcher for IamAccountFilterReq {
    fn basic(&self) -> &RbumBasicFilterReq {
        &self.basic
    }
}

#[derive(Object, Serialize, Deserialize, Debug, Clone)]
#[serde(default)]
pub struct IamAppFilterReq {
    pub basic: RbumBasicFilterReq,
    pub icon: Option<String>,
    pub sort: Option<u32>,
    pub contact_phone: Option<String>,
}

impl Default for IamAppFilterReq {
    fn default() -> Self {
        Self {
            basic: Default::default(),
            icon: None,
            sort: None,
            contact_phone: None,
        }
    }
}

impl RbumBasicFilterFetcher for IamAppFilterReq {
    fn basic(&self) -> &RbumBasicFilterReq {
        &self.basic
    }
}

#[derive(Object, Serialize, Deserialize, Debug, Clone)]
#[serde(default)]
pub struct IamTenantFilterReq {
    pub basic: RbumBasicFilterReq,
    pub icon: Option<String>,
    pub sort: Option<u32>,
    pub contact_phone: Option<String>,
}

impl Default for IamTenantFilterReq {
    fn default() -> Self {
        Self {
            basic: Default::default(),
            icon: None,
            sort: None,
            contact_phone: None,
        }
    }
}

impl RbumBasicFilterFetcher for IamTenantFilterReq {
    fn basic(&self) -> &RbumBasicFilterReq {
        &self.basic
    }
}

#[derive(Object, Serialize, Deserialize, Debug, Clone)]
#[serde(default)]
pub struct IamHttpResFilterReq {
    pub basic: RbumBasicFilterReq,
    pub icon: Option<String>,
    pub sort: Option<u32>,
    pub method: Option<String>,
}

impl Default for IamHttpResFilterReq {
    fn default() -> Self {
        Self {
            basic: Default::default(),
            icon: None,
            sort: None,
            method: None,
        }
    }
}

impl RbumBasicFilterFetcher for IamHttpResFilterReq {
    fn basic(&self) -> &RbumBasicFilterReq {
        &self.basic
    }
}

#[derive(Object, Serialize, Deserialize, Debug, Clone)]
#[serde(default)]
pub struct IamRoleFilterReq {
    pub basic: RbumBasicFilterReq,
    pub icon: Option<String>,
    pub sort: Option<u32>,
}

impl Default for IamRoleFilterReq {
    fn default() -> Self {
        Self {
            basic: Default::default(),
            icon: None,
            sort: None,
        }
    }
}

impl RbumBasicFilterFetcher for IamRoleFilterReq {
    fn basic(&self) -> &RbumBasicFilterReq {
        &self.basic
    }
}
