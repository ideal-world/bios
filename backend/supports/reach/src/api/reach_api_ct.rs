mod reach_api_ct_message;
pub use reach_api_ct_message::ReachMessageCtApi;
mod reach_api_ct_msg_log;
pub use reach_api_ct_msg_log::ReachMsgLogCtApi;
mod reach_api_ct_msg_signature;
pub use reach_api_ct_msg_signature::ReachMsgSignatureCtApi;
mod reach_api_ct_msg_template;
pub use reach_api_ct_msg_template::ReachMessageTemplateCtApi;
mod reach_api_ct_trigger_global;
pub use reach_api_ct_trigger_global::ReachTriggerGlobalConfigCtApi;
use tardis::basic::{error::TardisError, result::TardisResult};
mod reach_api_ct_trigger_instance;
pub use reach_api_ct_trigger_instance::ReachTriggerInstanceConfigCtApi;
mod reach_api_ct_vcode_strategy;
pub use reach_api_ct_vcode_strategy::ReachVcodeStrategyCtApi;

pub type ReachCtApi = (
    ReachMessageCtApi,
    ReachMsgSignatureCtApi,
    ReachMsgLogCtApi,
    ReachMessageTemplateCtApi,
    ReachTriggerGlobalConfigCtApi,
    ReachTriggerInstanceConfigCtApi,
    ReachVcodeStrategyCtApi,
);

fn map_notfound_to_false(e: TardisError) -> TardisResult<bool> {
    if e.code.contains("404") {
        Ok(false)
    } else {
        Err(e)
    }
}
