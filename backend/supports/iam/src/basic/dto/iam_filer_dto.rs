use std::default::Default;

use serde::{Deserialize, Serialize};
use tardis::web::poem_openapi;

use bios_basic::rbum::dto::rbum_filer_dto::{RbumBasicFilterReq, RbumItemFilterFetcher, RbumItemRelFilterReq, RbumSetItemRelFilterReq};

use crate::{
    basic::dto::iam_app_dto::IamAppKind,
    iam_enumeration::{IamAccountStatusKind, IamResKind, IamRoleKind, IamSubDeployHostKind},
};

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct IamConfigFilterReq {
    pub basic: RbumBasicFilterReq,
    pub code: Option<String>,
    pub rel_item_id: Option<String>,
    pub disabled: Option<bool>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct IamSubDeployFilterReq {
    pub basic: RbumBasicFilterReq,
    pub rel: Option<RbumItemRelFilterReq>,
    pub rel2: Option<RbumItemRelFilterReq>,
    pub province: Option<String>,
    pub access_url: Option<String>,
    pub extend_sub_deploy_id: Option<String>,
}

impl RbumItemFilterFetcher for IamSubDeployFilterReq {
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
pub struct IamSubDeployHostFilterReq {
    pub basic: RbumBasicFilterReq,
    pub sub_deploy_id: Option<String>,
    pub host: Option<String>,
    pub host_type: Option<IamSubDeployHostKind>,
}
#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct IamSubDeployLicenseFilterReq {
    pub basic: RbumBasicFilterReq,
    pub sub_deploy_id: Option<String>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct IamAccountFilterReq {
    pub basic: RbumBasicFilterReq,
    pub rel: Option<RbumItemRelFilterReq>,
    pub rel2: Option<RbumItemRelFilterReq>,
    pub rel3: Option<RbumItemRelFilterReq>,
    pub rel4: Option<RbumItemRelFilterReq>,
    pub set_rel: Option<RbumSetItemRelFilterReq>,
    pub icon: Option<String>,
    pub status: Option<IamAccountStatusKind>,
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
    pub sort: Option<i64>,
    pub kind: Option<IamAppKind>,
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
    pub sort: Option<i64>,
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
    pub sort: Option<i64>,
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
    pub kind: Option<IamRoleKind>,
    pub in_base: Option<bool>,
    pub in_embed: Option<bool>,
    pub extend_role_id: Option<String>,
    pub rel: Option<RbumItemRelFilterReq>,
    pub rel2: Option<RbumItemRelFilterReq>,
    pub icon: Option<String>,
    pub sort: Option<i64>,
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
