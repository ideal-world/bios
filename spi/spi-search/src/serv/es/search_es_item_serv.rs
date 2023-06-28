use std::collections::linked_list::IterMut;

use bios_basic::{basic_enumeration::BasicQueryOpKind, helper::db_helper, spi::spi_funs::SpiBsInstExtractor};
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    chrono::Utc,
    db::{
        reldb_client::{TardisRelDBClient, TardisRelDBlConnection},
        sea_orm::Value,
    },
    serde_json,
    web::web_resp::TardisPage,
    TardisFuns, TardisFunsInst, search::search_client::TardisSearchClient,
};

use crate::dto::search_item_dto::{SearchItemAddReq, SearchItemModifyReq, SearchItemSearchQScopeKind, SearchItemSearchReq, SearchItemSearchResp};

use super::search_es_initializer;

pub async fn add(add_req: &mut SearchItemAddReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    let data = format!(r#"{{"data": {}}}"#, TardisFuns::json.obj_to_string(add_req)?);
    let client = funs.bs(ctx).await?.inst::<TardisSearchClient>().0;
    search_es_initializer::init_index(client, &add_req.tag).await?;
    client.create_record(&add_req.tag, &data).await?;
    Ok(())
}

pub async fn modify(tag: &str, key: &str, modify_req: &mut SearchItemModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    panic!("not implemented")
}

pub async fn delete(tag: &str, key: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    panic!("not implemented")
}

pub async fn search(search_req: &mut SearchItemSearchReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<TardisPage<SearchItemSearchResp>> {
    let client = funs.bs(ctx).await?.inst::<TardisSearchClient>().0;
    let result = client.raw_search(&search_req.tag, r#"{}"#).await?
    .iter()
    .map(|item| TardisFuns::json.str_to_obj::<SearchItemSearchResp>(item))
    .collect::<Result<Vec<_>, _>>()?;

    Ok(TardisPage {
        page_size: search_req.page.size as u64,
        page_number: search_req.page.number as u64,
        total_size: 1 as u64,
        records: result,
    })
    // todo!()
}
