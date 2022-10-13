use bios_chat_basic::chat_constants;
use tardis::{basic::result::TardisResult, web::web_server::TardisWebServer, TardisFunsInst};

use crate::{chat_config::ChatConfig, console_common::api::chat_cc_message};

pub async fn init(web_server: &TardisWebServer) -> TardisResult<()> {
    let funs = chat_constants::get_tardis_inst();
    init_db(funs).await?;
    init_api(web_server).await
}

async fn init_api(web_server: &TardisWebServer) -> TardisResult<()> {
    web_server.add_module(chat_constants::COMPONENT_CODE, (chat_cc_message::ChatCcMessageApi)).await;
    Ok(())
}

pub async fn init_db(mut funs: TardisFunsInst) -> TardisResult<()> {
    bios_basic::rbum::rbum_initializer::init(funs.module_code(), funs.conf::<ChatConfig>().rbum.clone()).await?;
    funs.begin().await?;
    // TODO
    funs.commit().await?;
    Ok(())
}
