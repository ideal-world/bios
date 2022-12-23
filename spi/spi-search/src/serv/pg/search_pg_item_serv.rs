use bios_basic::spi::spi_funs::SpiBsInstExtractor;
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    db::reldb_client::TardisRelDBClient,
    web::web_resp::TardisPage,
    TardisFunsInst,
};

use crate::dto::search_item_dto::{SearchItemAddOrModifyReq, SearchItemQueryReq, SearchItemQueryResp};

use super::search_pg_initializer;

pub async fn add_or_modify(add_or_modify_req: &mut SearchItemAddOrModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    let bs_inst = funs.bs(ctx).await?.inst::<TardisRelDBClient>();
    let conn = search_pg_initializer::init_conn(bs_inst).await?;
    //  conn.execute_one(&ddl_req.sql, params).await?;
    Ok(())
}

pub async fn delete(tag: &str, key: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    let bs_inst = funs.bs(ctx).await?.inst::<TardisRelDBClient>();
    let conn = search_pg_initializer::init_conn(bs_inst).await?;
    Ok(())
}

pub async fn query(query_req: &mut SearchItemQueryReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<TardisPage<SearchItemQueryResp>> {
    let bs_inst = funs.bs(ctx).await?.inst::<TardisRelDBClient>();
    let conn = search_pg_initializer::init_conn(bs_inst).await?;
    Ok(TardisPage {
        page_size: 0,
        page_number: 0,
        total_size: 0,
        records: vec![],
    })
}
