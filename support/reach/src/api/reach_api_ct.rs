mod reach_api_ct_message;
pub use reach_api_ct_message::ReachMessageCtApi;
mod reach_api_ct_msg_log;
pub use reach_api_ct_msg_log::ReachMsgLogCtApi;
mod reach_api_ct_msg_signature;
pub use reach_api_ct_msg_signature::ReachMsgSignatureCtApi;
mod reach_api_ct_msg_template;
pub use reach_api_ct_msg_template::ReachMessageTemplateCtApi;
mod reach_api_ct_trigger_global;
use tardis::basic::{error::TardisError, result::TardisResult};
pub use reach_api_ct_trigger_global::ReachTriggerGlobalConfigCtApi;
mod reach_api_ct_trigger_instance;
pub use reach_api_ct_trigger_instance::ReachTriggerInstanceConfigCtApi;

pub type ReachCtApi = (
    ReachMessageCtApi,
    ReachMsgSignatureCtApi,
    ReachMsgLogCtApi,
    ReachMessageTemplateCtApi,
    ReachTriggerGlobalConfigCtApi,
    ReachTriggerInstanceConfigCtApi,
);

fn map_notfound_to_false(e: TardisError) -> TardisResult<bool> {
    if e.code.contains("404") {
        Ok(false)
    } else {
        Err(e)
    }
}
