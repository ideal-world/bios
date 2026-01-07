pub use reach_api_cs_msg_template::ReachMessageTemplateCsApi;
pub use reach_api_cs_msg_signature::ReachMsgSignatureCsApi;
pub use reach_api_cs_message::ReachMessageCsApi;
use tardis::basic::{error::TardisError, result::TardisResult};
mod reach_api_cs_msg_template;
mod reach_api_cs_msg_signature;
mod reach_api_cs_message;

pub type ReachCsApi = (ReachMessageTemplateCsApi, ReachMsgSignatureCsApi, ReachMessageCsApi);

fn map_notfound_to_false(e: TardisError) -> TardisResult<bool> {
    if e.code.contains("404") {
        Ok(false)
    } else {
        Err(e)
    }
}
