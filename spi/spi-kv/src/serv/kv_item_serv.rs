use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::serde_json::json;
use tardis::web::web_resp::TardisPage;
use tardis::{TardisFuns, TardisFunsInst};

use bios_basic::spi::spi_constants;
use bios_basic::spi::spi_funs::SpiBsInstExtractor;
use bios_basic::spi_dispatch_service;

use crate::dto::kv_item_dto::{
    KvItemAddOrModifyReq, KvItemDetailResp, KvItemMatchReq, KvItemSummaryResp, KvNameAddOrModifyReq, KvNameFindResp, KvTagAddOrModifyReq, KvTagFindResp,
};
use crate::{kv_constants, kv_initializer};

use super::pg;

spi_dispatch_service! {
    @mgr: true,
    @init: kv_initializer::init_fun,
    @dispatch: {
        #[cfg(feature = "spi-pg")]
        spi_constants::SPI_PG_KIND_CODE => pg::kv_pg_item_serv,
    },
    @method: {
        add_or_modify_item(add_or_modify_req: &mut KvItemAddOrModifyReq) -> TardisResult<()>;
        get_item(key: String, extract: Option<String>) -> TardisResult<Option<KvItemDetailResp>>;
        find_items(keys: Vec<String>, extract: Option<String>) -> TardisResult<Vec<KvItemSummaryResp>>;
        match_items(match_req: KvItemMatchReq) -> TardisResult<TardisPage<KvItemSummaryResp>>;
        delete_item(key: String) -> TardisResult<()>;
    }
}

pub async fn add_or_modify_key_name(add_or_modify_req: &mut KvNameAddOrModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    let req = KvItemAddOrModifyReq {
        key: format!("{}{}", kv_constants::KEY_PREFIX_BY_KEY_NAME, add_or_modify_req.key).into(),
        value: json!(add_or_modify_req.name),
        scope_level: add_or_modify_req.scope_level,
        info: None,
    };
    let inst = funs.init(ctx, true, kv_initializer::init_fun).await?;
    match inst.kind_code() {
        #[cfg(feature = "spi-pg")]
        spi_constants::SPI_PG_KIND_CODE => pg::kv_pg_item_serv::add_or_modify_item(&req, funs, ctx, &inst).await,
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
}

pub async fn find_key_names(keys: Vec<String>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Vec<KvNameFindResp>> {
    let keys = keys.into_iter().map(|key| format!("{}{}", kv_constants::KEY_PREFIX_BY_KEY_NAME, key)).collect();
    let inst = funs.init(ctx, true, kv_initializer::init_fun).await?;
    match inst.kind_code() {
        #[cfg(feature = "spi-pg")]
        spi_constants::SPI_PG_KIND_CODE => pg::kv_pg_item_serv::find_items(keys, None, funs, ctx, &inst).await,
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
    .and_then(|items| {
        items
            .into_iter()
            .map::<TardisResult<KvNameFindResp>, _>(|item| {
                Ok(KvNameFindResp {
                    key: item.key.strip_prefix(kv_constants::KEY_PREFIX_BY_KEY_NAME).unwrap_or("").to_string(),
                    name: item.value.as_str().unwrap_or("").to_string(),
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
        scope_level: add_or_modify_req.scope_level,
        info: None,
    };
    let inst = funs.init(ctx, true, kv_initializer::init_fun).await?;
    match inst.kind_code() {
        #[cfg(feature = "spi-pg")]
        spi_constants::SPI_PG_KIND_CODE => pg::kv_pg_item_serv::add_or_modify_item(&req, funs, ctx, &inst).await,
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
}

pub async fn find_tags(keys: Vec<String>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Vec<KvTagFindResp>> {
    let keys = keys.iter().map(|r| format!("{}{}", kv_constants::KEY_PREFIX_BY_TAG, r)).collect::<Vec<_>>();
    let inst = funs.init(ctx, true, kv_initializer::init_fun).await?;
    match inst.kind_code() {
        #[cfg(feature = "spi-pg")]
        spi_constants::SPI_PG_KIND_CODE => pg::kv_pg_item_serv::find_items(keys, None, funs, ctx, &inst).await,
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
    .and_then(|items| {
        Ok(items
            .into_iter()
            .map(|item| {
                Ok(KvTagFindResp {
                    key: item.key.strip_prefix(kv_constants::KEY_PREFIX_BY_TAG).unwrap_or("").to_string(),
                    items: TardisFuns::json.json_to_obj(item.value)?,
                    create_time: item.create_time,
                    update_time: item.update_time,
                })
            })
            .collect::<TardisResult<Vec<_>>>()?)
    })
}

pub async fn page_tags(
    key_prefix: String,
    page_number: u32,
    page_size: u16,
    desc_sort_by_create: Query<Option<bool>>,
    desc_sort_by_create: Query<Option<bool>>,
    funs: &TardisFunsInst,
    ctx: &TardisContext,
) -> TardisResult<TardisPage<KvTagFindResp>> {
    let key_prefix = format!("{}{}", kv_constants::KEY_PREFIX_BY_TAG, key_prefix);
    let inst = funs.init(ctx, true, kv_initializer::init_fun).await?;
    match inst.kind_code() {
        #[cfg(feature = "spi-pg")]
        spi_constants::SPI_PG_KIND_CODE => {
            pg::kv_pg_item_serv::match_items(
                KvItemMatchReq {
                    key_prefix,
                    page_number,
                    page_size,
                    desc_sort_by_create,
                    desc_sort_by_create,
                    ..Default::default()
                },
                funs,
                ctx,
                &inst,
            )
            .await
        }
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
    .and_then(|items| {
        Ok(TardisPage {
            page_size: items.page_size,
            page_number: items.page_number,
            total_size: items.total_size,
            records: items
                .records
                .into_iter()
                .map(|item| {
                    Ok(KvTagFindResp {
                        key: item.key.strip_prefix(kv_constants::KEY_PREFIX_BY_TAG).unwrap_or("").to_string(),
                        items: TardisFuns::json.json_to_obj(item.value)?,
                        create_time: item.create_time,
                        update_time: item.update_time,
                    })
                })
                .collect::<TardisResult<Vec<_>>>()?,
        })
    })
}
