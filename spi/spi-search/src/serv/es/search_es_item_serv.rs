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
    TardisFuns, TardisFunsInst,
};

use crate::dto::search_item_dto::{SearchItemAddReq, SearchItemModifyReq, SearchItemSearchQScopeKind, SearchItemSearchReq, SearchItemSearchResp};

use super::search_es_initializer;

pub async fn add(add_req: &mut SearchItemAddReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    let params = TardisFuns::json.obj_to_string(add_req)?;
    let mut params = Vec::new();
    params.push(Value::from(add_req.kind.to_string()));
    params.push(Value::from(add_req.key.to_string()));
    params.push(Value::from(add_req.title.as_str()));
    params.push(Value::from(add_req.title.as_str()));
    params.push(Value::from(add_req.content.as_str()));
    params.push(Value::from(add_req.owner.as_ref().unwrap_or(&"".to_string()).as_str()));
    params.push(Value::from(add_req.own_paths.as_ref().unwrap_or(&"".to_string()).as_str()));
    params.push(Value::from(if let Some(create_time) = add_req.create_time { create_time } else { Utc::now() }));
    params.push(Value::from(if let Some(update_time) = add_req.update_time { update_time } else { Utc::now() }));
    params.push(Value::from(if let Some(ext) = &add_req.ext {
        ext.clone()
    } else {
        TardisFuns::json.str_to_json("{}")?
    }));
    if let Some(visit_keys) = &add_req.visit_keys {
        params.push(Value::from(visit_keys.to_sql()));
    };


    let client = TardisFuns::search();
    search_es_initializer::init_index(client, &add_req.tag).await?;
    Ok(())
}

pub async fn modify(tag: &str, key: &str, modify_req: &mut SearchItemModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    let client = TardisFuns::search();
    search_es_initializer::init_index(client, tag).await?;

    Ok(())
}

pub async fn delete(tag: &str, key: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    let client = TardisFuns::search();
    search_es_initializer::init_index(client, tag).await?;
    Ok(())
}
