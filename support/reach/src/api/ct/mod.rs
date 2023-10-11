mod message;
pub use message::ReachMessageCtApi;
mod msg_log;
pub use msg_log::ReachMsgLogCtApi;
mod msg_signature;
pub use msg_signature::ReachMsgSignatureCtApi;
mod msg_template;
pub use msg_template::ReachMessageTemplateCtApi;
mod trigger_global;
use tardis::basic::{error::TardisError, result::TardisResult};
pub use trigger_global::ReachTriggerGlobalConfigCtApi;
mod trigger_instance;
pub use trigger_instance::ReachTriggerInstanceConfigCtApi;

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
