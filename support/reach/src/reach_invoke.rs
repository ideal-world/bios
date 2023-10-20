use bios_sdk_invoke::clients::SimpleInvokeClient;
use tardis::{basic::dto::TardisContext, TardisFunsInst};

pub struct Client<'a> {
    base_url: &'a str,
    ctx: &'a TardisContext,
    funs: &'a TardisFunsInst,
}

impl<'a> Client<'a> {
    pub fn new(base_url: &'a str, ctx: &'a TardisContext, funs: &'a TardisFunsInst) -> Self {
        Self { base_url, funs, ctx }
    }
}

impl SimpleInvokeClient for Client<'_> {
    const DOMAIN_CODE: &'static str = crate::reach_consts::DOMAIN_CODE;

    fn get_ctx(&self) -> &tardis::basic::dto::TardisContext {
        self.ctx
    }

    fn get_base_url(&self) -> &str {
        self.base_url
    }

    fn get_funs(&self) -> &TardisFunsInst {
        self.funs
    }
}
