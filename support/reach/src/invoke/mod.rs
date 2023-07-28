mod config;
use bios_sdk_invoke::clients::InvokeClient;
pub use config::Config;
use tardis::{basic::dto::TardisContext, TardisFunsInst};
pub mod api;

pub struct Client<'a> {
    config: &'a Config,
    ctx: &'a TardisContext,
    funs: &'a TardisFunsInst,
}

impl<'a> Client<'a> {
    pub fn new(config: &'a Config, ctx: &'a TardisContext, funs: &'a TardisFunsInst) -> Self {
        Self { config, funs, ctx }
    }
}

impl InvokeClient for Client<'_> {
    const DOMAIN_CODE: &'static str = crate::consts::DOMAIN_CODE;

    fn get_ctx(&self) -> &tardis::basic::dto::TardisContext {
        self.ctx
    }

    fn get_base_url(&self) -> &str {
        &self.config.base_url
    }
}
