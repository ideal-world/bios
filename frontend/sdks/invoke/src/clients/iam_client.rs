use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};
use tardis::{basic::dto::TardisContext, web::poem_openapi, TardisFunsInst};

use crate::impl_tardis_api_client;

use super::SimpleInvokeClient;

#[derive(Clone)]
pub struct IamClient<'a> {
    pub funs: &'a TardisFunsInst,
    pub ctx: &'a TardisContext,
    pub account: &'a str,
    pub base_url: &'a str,
}

impl<'a> IamClient<'a> {
    pub fn new(account: &'a str, funs: &'a TardisFunsInst, ctx: &'a TardisContext, url: &'a str) -> Self {
        Self {
            funs,
            ctx,
            account,
            base_url: url,
        }
    }
}
impl<'a> SimpleInvokeClient for IamClient<'a> {
    const DOMAIN_CODE: &'static str = "iam";
    fn get_ctx(&self) -> &'a TardisContext {
        self.ctx
    }

    fn get_base_url(&self) -> &str {
        self.base_url
    }

    fn get_funs(&self) -> &tardis::TardisFunsInst {
        self.funs
    }
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamAccountDetailAggResp {
    pub owner_name: Option<String>,
    pub roles: HashMap<String, String>,
    pub certs: HashMap<String, String>,
    pub orgs: Vec<String>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamCertDecodeRequest {
    pub codes: HashSet<String>,
}

impl_tardis_api_client! {
    IamClient<'_>:
    {get_account, get ["/ct/account", id] {tenant_id} IamAccountDetailAggResp}
    {batch_decode_cert, post ["/ci/cert/decode"] IamCertDecodeRequest => HashMap<String, String>}
}
