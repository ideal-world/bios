use bios_sdk_invoke::{
    clients::event_client::{asteroid_mq_sdk::model::Interest, mq_client_node_opt, mq_error, ContextHandler, SPI_RPC_TOPIC},
    dto::search_item_dto::{SearchEventItemDeleteReq, SearchEventItemModifyReq, SearchItemAddReq},
};

use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    log::instrument,
};

use crate::{search_initializer::get_tardis_inst, serv};

async fn handle_add_event(req: SearchItemAddReq, ctx: TardisContext) -> TardisResult<()> {
    let funs = get_tardis_inst();
    serv::search_item_serv::add(&mut req.into(), &funs, &ctx).await?;
    Ok(())
}

async fn handle_modify_event(req: SearchEventItemModifyReq, ctx: TardisContext) -> TardisResult<()> {
    let funs = get_tardis_inst();
    serv::search_item_serv::modify(&req.tag, &req.key, &mut req.item.into(), &funs, &ctx).await?;
    Ok(())
}

async fn handle_delete_event(req: SearchEventItemDeleteReq, ctx: TardisContext) -> TardisResult<()> {
    let funs = get_tardis_inst();
    serv::search_item_serv::delete(&req.tag, &req.key, &funs, &ctx).await?;
    Ok(())
}

pub async fn handle_events() -> TardisResult<()> {
    // if let Some(node) = mq_client_node_opt() {
    //     node.create_endpoint(SPI_RPC_TOPIC, [Interest::new("search/*")])
    //         .await
    //         .map_err(mq_error)?
    //         .into_event_loop()
    //         .with_handler(ContextHandler(handle_modify_event))
    //         .with_handler(ContextHandler(handle_add_event))
    //         .with_handler(ContextHandler(handle_delete_event))
    //         .spawn();
    // }

    Ok(())
}
