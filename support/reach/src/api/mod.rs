mod cc;
use std::sync::OnceLock;

pub use cc::*;

mod ct;
pub use ct::*;
use tardis::{basic::result::TardisResult, web::web_server::TardisWebServer};

use crate::{consts::DOMAIN_CODE, client::SendChannelMap};

pub type ReachApi = (ReachCcApi, ReachCtApi);
pub(crate) static REACH_SEND_CHANNEL_MAP: OnceLock<SendChannelMap> = OnceLock::new();
pub async fn init(web_server: &TardisWebServer, send_channels: SendChannelMap) -> TardisResult<()> {
    REACH_SEND_CHANNEL_MAP.get_or_init(||send_channels);
    web_server.add_module(DOMAIN_CODE, ReachApi::default()).await;
    Ok(())
}
