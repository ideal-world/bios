use std::default::Default;

use serde::{Deserialize, Serialize};
use tardis::web::poem_openapi;

use bios_basic::rbum::dto::rbum_filer_dto::{RbumBasicFilterReq, RbumItemFilterFetcher, RbumItemRelFilterReq};

use crate::iam_enumeration::IamResKind;

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct IamAccountFilterReq {
    pub basic: RbumBasicFilterReq,
    pub rel: Option<RbumItemRelFilterReq>,
    pub rel2: Option<RbumItemRelFilterReq>,
    pub icon: Option<String>,
}

impl RbumItemFilterFetcher for IamAccountFilterReq {
    fn basic(&self) -> &RbumBasicFilterReq {
        &self.basic
    }
    fn rel(&self) -> &Option<RbumItemRelFilterReq> {
        &self.rel
    }
    fn rel2(&self) -> &Option<RbumItemRelFilterReq> {
        &self.rel2
    }
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct IamAppFilterReq {
    pub basic: RbumBasicFilterReq,
    pub rel: Option<RbumItemRelFilterReq>,
    pub rel2: Option<RbumItemRelFilterReq>,
    pub icon: Option<String>,
    pub sort: Option<u32>,
    pub contact_phone: Option<String>,
}

impl RbumItemFilterFetcher for IamAppFilterReq {
    fn basic(&self) -> &RbumBasicFilterReq {
        &self.basic
    }
    fn rel(&self) -> &Option<RbumItemRelFilterReq> {
        &self.rel
    }
    fn rel2(&self) -> &Option<RbumItemRelFilterReq> {
        &self.rel2
    }
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct IamTenantFilterReq {
    pub basic: RbumBasicFilterReq,
    pub rel: Option<RbumItemRelFilterReq>,
    pub rel2: Option<RbumItemRelFilterReq>,
    pub icon: Option<String>,
    pub sort: Option<u32>,
    pub contact_phone: Option<String>,
}

impl RbumItemFilterFetcher for IamTenantFilterReq {
    fn basic(&self) -> &RbumBasicFilterReq {
        &self.basic
    }
    fn rel(&self) -> &Option<RbumItemRelFilterReq> {
        &self.rel
    }
    fn rel2(&self) -> &Option<RbumItemRelFilterReq> {
        &self.rel2
    }
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct IamResFilterReq {
    pub basic: RbumBasicFilterReq,
    pub rel: Option<RbumItemRelFilterReq>,
    pub rel2: Option<RbumItemRelFilterReq>,
    pub kind: Option<IamResKind>,
    pub icon: Option<String>,
    pub sort: Option<u32>,
    pub method: Option<String>,
}

impl RbumItemFilterFetcher for IamResFilterReq {
    fn basic(&self) -> &RbumBasicFilterReq {
        &self.basic
    }
    fn rel(&self) -> &Option<RbumItemRelFilterReq> {
        &self.rel
    }
    fn rel2(&self) -> &Option<RbumItemRelFilterReq> {
        &self.rel2
    }
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct IamRoleFilterReq {
    pub basic: RbumBasicFilterReq,
    pub rel: Option<RbumItemRelFilterReq>,
    pub rel2: Option<RbumItemRelFilterReq>,
    pub icon: Option<String>,
    pub sort: Option<u32>,
}

impl RbumItemFilterFetcher for IamRoleFilterReq {
    fn basic(&self) -> &RbumBasicFilterReq {
        &self.basic
    }
    fn rel(&self) -> &Option<RbumItemRelFilterReq> {
        &self.rel
    }
    fn rel2(&self) -> &Option<RbumItemRelFilterReq> {
        &self.rel2
    }
}
