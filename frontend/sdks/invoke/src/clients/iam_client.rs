use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};
use tardis::{
    basic::dto::TardisContext,
    web::{poem_openapi, web_resp::Void},
    TardisFunsInst,
};

use crate::impl_tardis_api_client;

use super::SimpleInvokeClient;

#[derive(Clone)]
pub struct IamClient<'a> {
    pub funs: &'a TardisFunsInst,
    pub ctx: &'a TardisContext,
    pub bios_ctx: Option<TardisContext>,
    pub account: &'a str,
    pub base_url: &'a str,
}

impl<'a> IamClient<'a> {
    pub fn new(account: &'a str, funs: &'a TardisFunsInst, ctx: &'a TardisContext, bios_ctx: Option<TardisContext>, url: &'a str) -> Self {
        Self {
            funs,
            ctx,
            bios_ctx,
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

    fn get_bios_ctx(&self) -> &Option<TardisContext> {
        &self.bios_ctx
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

/// 赋予或覆盖账号对一个产品的临时访问。`expire_at` 为 Unix 秒，`0` 表示不过期。
#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamCiTempProductGrantReq {
    pub account_id: String,
    pub product_id: String,
    pub tenant_id: String,
    pub expire_at: i64,
}

/// 整体替换全系统可临时访问的接口。元素为 `{res_uri}##{res_action}`，空列表表示清空。
#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamCiTempApiReplaceReq {
    pub apis: Vec<String>,
}

impl_tardis_api_client! {
    IamClient<'_>:
    {find_all_app_ctx, get ["/ci/app/all/ctx"] Vec<String>}
    {get_account, get ["/ct/account", id] {tenant_id} IamAccountDetailAggResp}
    {batch_decode_cert, post ["/ci/cert/decode"] IamCertDecodeRequest => HashMap<String, String>}
    {grant_temp_product, put ["/ci/temp/product"] IamCiTempProductGrantReq => Void}
    {revoke_temp_product, delete ["/ci/temp/product", account_id, product_id] Void}
    {replace_temp_apis, put ["/ci/temp/api"] IamCiTempApiReplaceReq => Void}
}
