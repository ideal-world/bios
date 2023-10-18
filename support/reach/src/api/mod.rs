mod cc;

pub use cc::*;

mod ct;
pub use ct::*;

mod ci;
pub use ci::*;
use tardis::{basic::result::TardisResult, web::web_server::TardisWebServer};

use crate::consts::DOMAIN_CODE;

pub type ReachApi = (ReachCcApi, ReachCtApi, ReachMessageCiApi);
pub async fn init(web_server: &TardisWebServer) -> TardisResult<()> {
    web_server.add_module(DOMAIN_CODE, ReachApi::default()).await;
    Ok(())
}
