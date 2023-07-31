
use std::collections::HashMap;

use bios_sdk_invoke::{clients::SimpleInvokeClient, impl_taidis_api_client};
use tardis::{basic::dto::TardisContext, TardisFunsInst};

use crate::dto::*;

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
    const DOMAIN_CODE: &'static str = crate::consts::DOMAIN_CODE;

    fn get_ctx(&self) -> &tardis::basic::dto::TardisContext {
        self.ctx
    }

    fn get_base_url(&self) -> &str {
        self.base_url
    }
}

impl_taidis_api_client! {
    Client<'_>:
    // cc
    { general_send, put ["/cc/msg/general", to, msg_template_id] HashMap<String, String> => () }
    { vcode_send, put ["/cc/msg/vcode", to, code] () => () }
    { pwd_send, put ["/cc/msg/pwd", to, code] () => () }
    { mail_pwd_send, put ["/cc/msg/mail", mail] {message, subject} () => () }
    { find_trigger_scene, get ["/cc/trigger/scene/"] Vec<ReachTriggerSceneSummaryResp>}
    { find_trigger_scene_by_code, get ["/cc/trigger/scene/code"] {code} Vec<ReachTriggerSceneSummaryResp>}
    // ct
    { paginate_msg_log, post ["/ct/msg"] {page_number?: u32, page_size?: u32} ReachMessageAddReq => String }
    { add_message, post ["/ct/msg"] ReachMessageAddReq => String }
    { delete_test, delete ["/delete/some/msg"] String }
}
