use bios_basic::spi::spi_constants;
use bios_basic::spi::spi_funs::SpiBsInstExtractor;
use tardis::basic::dto::TardisContext;
use tardis::basic::error::TardisError;
use tardis::basic::result::TardisResult;
use tardis::serde_json::json;
use tardis::web::web_resp::TardisPage;
use tardis::{TardisFuns, TardisFunsInst};

use crate::dto::kv_item_dto::{
    KvItemAddOrModifyReq, KvItemDetailResp, KvItemMatchReq, KvItemSummaryResp, KvNameAddOrModifyReq, KvNameFindResp, KvTagAddOrModifyReq, KvTagFindResp,
};
use crate::{kv_constants, kv_initializer};

use super::pg;

pub async fn add_or_modify_item(add_or_modify_req: &mut KvItemAddOrModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    match funs.init(ctx, true, kv_initializer::init_fun).await?.as_str() {
        #[cfg(feature = "spi-pg")]
        spi_constants::SPI_PG_KIND_CODE => pg::kv_pg_item_serv::add_or_modify_item(add_or_modify_req, funs, ctx).await,
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
}

pub async fn get_item(key: String, extract: Option<String>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Option<KvItemDetailResp>> {
    match funs.init(ctx, true, kv_initializer::init_fun).await?.as_str() {
        #[cfg(feature = "spi-pg")]
        spi_constants::SPI_PG_KIND_CODE => pg::kv_pg_item_serv::get_item(key, extract, funs, ctx).await,
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
}

pub async fn find_items(keys: Vec<String>, extract: Option<String>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Vec<KvItemSummaryResp>> {
    match funs.init(ctx, true, kv_initializer::init_fun).await?.as_str() {
        #[cfg(feature = "spi-pg")]
        spi_constants::SPI_PG_KIND_CODE => pg::kv_pg_item_serv::find_items(keys, extract, funs, ctx).await,
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
}

pub async fn match_items(match_req: KvItemMatchReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<TardisPage<KvItemSummaryResp>> {
    match funs.init(ctx, true, kv_initializer::init_fun).await?.as_str() {
        #[cfg(feature = "spi-pg")]
        spi_constants::SPI_PG_KIND_CODE => pg::kv_pg_item_serv::match_items(match_req, funs, ctx).await,
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
}

pub async fn delete_item(key: String, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    match funs.init(ctx, true, kv_initializer::init_fun).await?.as_str() {
        #[cfg(feature = "spi-pg")]
        spi_constants::SPI_PG_KIND_CODE => pg::kv_pg_item_serv::delete_item(key, funs, ctx).await,
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
}

pub async fn add_or_modify_key_name(add_or_modify_req: &mut KvNameAddOrModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    let req = KvItemAddOrModifyReq {
        key: format!("{}{}", kv_constants::KEY_PREFIX_BY_KEY_NAME, add_or_modify_req.key).into(),
        value: json!(add_or_modify_req.name),
        info: None,
    };
    match funs.init(ctx, true, kv_initializer::init_fun).await?.as_str() {
        #[cfg(feature = "spi-pg")]
        spi_constants::SPI_PG_KIND_CODE => pg::kv_pg_item_serv::add_or_modify_item(&req, funs, ctx).await,
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
}

pub async fn find_key_names(keys: Vec<String>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Vec<KvNameFindResp>> {
    let keys = keys.into_iter().map(|key| format!("{}{}", kv_constants::KEY_PREFIX_BY_KEY_NAME, key)).collect();
    match funs.init(ctx, true, kv_initializer::init_fun).await?.as_str() {
        #[cfg(feature = "spi-pg")]
        spi_constants::SPI_PG_KIND_CODE => pg::kv_pg_item_serv::find_items(keys, None, funs, ctx).await,
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
    .and_then(|items| {
        items
            .into_iter()
            .map::<TardisResult<KvNameFindResp>, _>(|item| {
                Ok(KvNameFindResp {
                    key: item
                        .key
                        .strip_prefix(kv_constants::KEY_PREFIX_BY_KEY_NAME)
                        .ok_or(TardisError::internal_error(
                            &format!(
                                "{ty} key is not start with {keyname}:{keyval}",
                                ty = stringify!(KvNameFindResp),
                                keyname = stringify!(kv_constants::KEY_PREFIX_BY_KEY_NAME),
                                keyval = item.key
                            ),
                            "",
                        ))?
                        .to_string(),
                    name: item.value.as_str().ok_or(TardisError::internal_error(
                        &format!(
                            "{ty}'s value is not a str, key: {key}, value: {value}",
                            ty = stringify!(KvNameFindResp),
                            key = item.key,
                            value = item.value
                    ), ""))?.to_string(),
                    create_time: item.create_time,
                    update_time: item.update_time,
                })
            })
            .collect::<TardisResult<Vec<KvNameFindResp>>>()
    })
}

pub async fn add_or_modify_tag(add_or_modify_req: &mut KvTagAddOrModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    let req = KvItemAddOrModifyReq {
        key: format!("{}{}", kv_constants::KEY_PREFIX_BY_TAG, add_or_modify_req.key).into(),
        value: TardisFuns::json.obj_to_json(&add_or_modify_req.items)?,
        info: None,
    };
    match funs.init(ctx, true, kv_initializer::init_fun).await?.as_str() {
        #[cfg(feature = "spi-pg")]
        spi_constants::SPI_PG_KIND_CODE => pg::kv_pg_item_serv::add_or_modify_item(&req, funs, ctx).await,
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
}

pub async fn find_tags(key_prefix: String, page_number: u32, page_size: u16, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<TardisPage<KvTagFindResp>> {
    let key_prefix = format!("{}{}", kv_constants::KEY_PREFIX_BY_TAG, key_prefix);
    match funs.init(ctx, true, kv_initializer::init_fun).await?.as_str() {
        #[cfg(feature = "spi-pg")]
        spi_constants::SPI_PG_KIND_CODE => {
            pg::kv_pg_item_serv::match_items(
                KvItemMatchReq {
                    key_prefix,
                    page_number,
                    page_size,
                    ..Default::default()
                },
                funs,
                ctx,
            )
            .await
        }
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
    .and_then(|items| Ok(TardisPage {
        page_size: items.page_size,
        page_number: items.page_number,
        total_size: items.total_size,
        records: items
            .records
            .into_iter()
            .map(|item| Ok(KvTagFindResp {
                key: item.key.strip_prefix(kv_constants::KEY_PREFIX_BY_TAG).ok_or(TardisError::internal_error(
                    &format!(
                        "{ty} key is not start with {keyname}:{keyval}",
                        ty = stringify!(KvTagFindResp),
                        keyname = stringify!(kv_constants::KEY_PREFIX_BY_TAG),
                        keyval = item.key
                    ),
                    "",
                ))?.to_string(),
                items: TardisFuns::json.json_to_obj(item.value)?,
                create_time: item.create_time,
                update_time: item.update_time,
            }))
            .collect::<TardisResult<Vec<_>>>()?,
    }))
}
