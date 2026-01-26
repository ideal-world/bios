mod reach_api_cc;

pub use reach_api_cc::*;

mod reach_api_ct;
pub use reach_api_ct::*;

mod reach_api_ci;
pub use reach_api_ci::*;

mod reach_api_cs;
pub use reach_api_cs::*;
use tardis::{basic::result::TardisResult, web::web_server::TardisWebServer};

use crate::reach_constants::DOMAIN_CODE;

pub type ReachApi = (ReachCcApi, ReachCtApi, ReachCcApi, ReachCsApi, ReachMessageCiApi, ReachTriggerInstanceConfigCiApi);
pub async fn init(web_server: &TardisWebServer) -> TardisResult<()> {
    web_server.add_module(DOMAIN_CODE, ReachApi::default()).await;
    Ok(())
}
