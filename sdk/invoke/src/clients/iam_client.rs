use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    TardisFunsInst, web::poem_openapi,
};

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
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamAccountSummaryAggResp {
    pub roles: HashMap<String, String>,
    pub certs: HashMap<String, String>,
    pub orgs: Vec<String>,
}

impl IamClient<'_> {
    pub async fn get_account(&self, id: &str, tenant_id: &str) -> TardisResult<IamAccountSummaryAggResp> {
        let ctx = self.get_tardis_context_header()?;
        let url = format!(
            "{base_url}/{account}/{id}?tenant_id={tenant_id}",
            base_url = self.base_url,
            account = self.account,
            id = id,
            tenant_id = tenant_id
        );
        let resp = self.funs.web_client().get::<IamAccountSummaryAggResp>(&url, Some(vec![ctx])).await?;
        let resp_body = resp.body.ok_or_else(|| self.funs.err().internal_error("iam-client", "get_account", "response", "500-iam_client-request_fail"))?;
        Ok(resp_body)
    }
}
