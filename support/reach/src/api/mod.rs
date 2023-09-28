mod cc;
use std::sync::{OnceLock, Arc};

pub use cc::*;

mod ct;
pub use ct::*;
use tardis::{basic::result::TardisResult, web::web_server::TardisWebServer};

use crate::{consts::DOMAIN_CODE, client::SendChannelMap};

pub type ReachApi = (ReachCcApi, ReachCtApi);
pub async fn init(web_server: &TardisWebServer) -> TardisResult<()> {
    web_server.add_module(DOMAIN_CODE, ReachApi::default()).await;
    Ok(())
}
