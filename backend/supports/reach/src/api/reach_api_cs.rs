pub use reach_api_cs_msg_template::ReachMessageTemplateCsApi;
use tardis::basic::{error::TardisError, result::TardisResult};
mod reach_api_cs_msg_template;

pub type ReachCsApi = ReachMessageTemplateCsApi;

fn map_notfound_to_false(e: TardisError) -> TardisResult<bool> {
    if e.code.contains("404") {
        Ok(false)
    } else {
        Err(e)
    }
}
