use crate::dto::stats_record_dto::{StatsDimRecordAddReq, StatsFactRecordLoadReq, StatsFactRecordsLoadReq};
use crate::stats_enumeration::{StatsDataTypeKind, StatsFactColKind};
use crate::stats_initializer;
use bios_basic::spi::spi_constants;
use bios_basic::spi::spi_funs::SpiBsInstExtractor;
use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::chrono::{DateTime, NaiveDate, Utc};
use tardis::db::sea_orm::Value;
use tardis::TardisFunsInst;

use super::pg;
use super::stats_conf_serv::{CONF_DIMS, CONF_FACTS};

pub(crate) async fn fact_load_record(fact_key: String, record_key: String, add_req: StatsFactRecordLoadReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    let conf_fact_lock = CONF_FACTS.read().await;
    let (_, fact_col_conf_set) = conf_fact_lock.get(&fact_key).ok_or(funs.err().not_found(
        "fact_record",
        "load",
        &format!("The fact instance table [{}] not exists.", &fact_key),
        "404-spi-stats-fact-inst-not-exist",
    ))?;

    let mut fields = vec!["key".to_string(), "own_paths".to_string(), "st".to_string()];
    let mut values = vec![Value::from(&record_key), Value::from(add_req.own_paths), Value::from(add_req.ct)];
    let req_data = add_req.data.as_object().unwrap();

    for (req_fact_col_key, req_fact_col_value) in req_data {
        let fact_col_conf = fact_col_conf_set.iter().find(|c| &c.key == req_fact_col_key).ok_or(funs.err().not_found(
            "fact_record",
            "load",
            &format!("The fact column config [{req_fact_col_key}] not exists."),
            "404-spi-stats-fact-col-conf-not-exist",
        ))?;

        if fact_col_conf.kind == StatsFactColKind::Dimension && CONF_DIMS.read().await.get(fact_col_conf.dim_rel_conf_dim_key.as_ref().unwrap()).unwrap().stable_ds {
            let dim_record_id = match funs.init(ctx, true, stats_initializer::init_fun).await?.as_str() {
                #[cfg(feature = "spi-pg")]
                spi_constants::SPI_PG_KIND_CODE => {
                    // TODO support other data type
                    pg::stats_pg_record_serv::dim_get_inst_record_id(fact_col_conf.dim_rel_conf_dim_key.as_ref().unwrap(), req_fact_col_value.as_str().unwrap(), funs, ctx).await?
                }
                kind_code => return Err(funs.bs_not_implemented(kind_code)),
            }
            .ok_or(funs.err().not_found(
                "fact_record",
                "load",
                &format!(
                    "The parent dimension instance record [{}] not exists.",
                    &fact_col_conf.dim_rel_conf_dim_key.as_ref().unwrap()
                ),
                "404-spi-stats-dim-inst-record-not-exist",
            ))?;
            // Replace dimension instance record id to dimension instance record key
            fields.push(req_fact_col_key.to_string());
            values.push(Value::from(dim_record_id));
        } else if fact_col_conf.kind == StatsFactColKind::Dimension {
            fields.push(req_fact_col_key.to_string());
            if fact_col_conf.dim_multi_values.unwrap_or(false) {
                values.push(Value::from(
                    req_fact_col_value.as_array().unwrap().iter().map(|v| v.as_str().unwrap().to_string()).collect::<Vec<String>>(),
                ));
            } else {
                values.push(Value::from(req_fact_col_value.as_str().unwrap()));
            }
        } else {
            fields.push(req_fact_col_key.to_string());
            match fact_col_conf.mes_data_type {
                Some(StatsDataTypeKind::Number) => values.push(Value::from(req_fact_col_value.as_i64().unwrap())),
                Some(StatsDataTypeKind::Boolean) => values.push(Value::from(req_fact_col_value.as_bool().unwrap())),
                Some(StatsDataTypeKind::DateTime) => values.push(Value::from(req_fact_col_value.as_str().unwrap())),
                Some(StatsDataTypeKind::Date) => values.push(Value::from(req_fact_col_value.as_str().unwrap())),
                _ => values.push(Value::from(req_fact_col_value.as_str().unwrap())),
            }
        }

        // TODO check data type
    }
    if fact_col_conf_set.len() != req_data.len() {
        let latest_data = match funs.init(ctx, true, stats_initializer::init_fun).await?.as_str() {
            #[cfg(feature = "spi-pg")]
            spi_constants::SPI_PG_KIND_CODE => pg::stats_pg_record_serv::fact_get_latest_record_raw(&fact_key, &record_key, funs, ctx).await?,
            kind_code => return Err(funs.bs_not_implemented(kind_code)),
        };
        if let Some(latest_data) = latest_data {
            for fact_col_conf in fact_col_conf_set {
                if !req_data.contains_key(&fact_col_conf.key) {
                    fields.push(fact_col_conf.key.to_string());
                    match fact_col_conf.mes_data_type {
                        Some(StatsDataTypeKind::Number) => values.push(Value::from(latest_data.try_get::<i32>("", &fact_col_conf.key).unwrap())),
                        Some(StatsDataTypeKind::Boolean) => values.push(Value::from(latest_data.try_get::<bool>("", &fact_col_conf.key).unwrap())),
                        Some(StatsDataTypeKind::DateTime) => values.push(Value::from(latest_data.try_get::<DateTime<Utc>>("", &fact_col_conf.key).unwrap())),
                        Some(StatsDataTypeKind::Date) => values.push(Value::from(latest_data.try_get::<NaiveDate>("", &fact_col_conf.key).unwrap())),
                        _ => values.push(Value::from(latest_data.try_get::<String>("", &fact_col_conf.key).unwrap())),
                    }
                }
            }
        } else {
            return Err(funs.err().not_found(
                "fact_record",
                "load",
                &format!("The fact latest instance record [{}][{}] not exists.", &fact_key, &record_key),
                "404-spi-stats-fact-inst-record-not-exist",
            ));
        }
    }
    match funs.init(ctx, true, stats_initializer::init_fun).await?.as_str() {
        #[cfg(feature = "spi-pg")]
        spi_constants::SPI_PG_KIND_CODE => pg::stats_pg_record_serv::fact_load_record(&fact_key, fields, values, funs, ctx).await,
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
}

pub(crate) async fn fact_delete_record(fact_key: String, record_key: String, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    let conf_fact_lock = CONF_FACTS.read().await;
    let (_, _) = conf_fact_lock.get(&fact_key).ok_or(funs.err().not_found(
        "fact_record",
        "delete",
        &format!("The fact instance table [{}] not exists.", &fact_key),
        "404-spi-stats-fact-inst-not-exist",
    ))?;

    match funs.init(ctx, true, stats_initializer::init_fun).await?.as_str() {
        #[cfg(feature = "spi-pg")]
        spi_constants::SPI_PG_KIND_CODE => pg::stats_pg_record_serv::fact_delete_record(&fact_key, &record_key, funs, ctx).await,
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
}

pub(crate) async fn fact_load_records(fact_key: String, add_req_set: Vec<StatsFactRecordsLoadReq>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    let conf_fact_lock = CONF_FACTS.read().await;
    let (_, fact_col_conf_set) = conf_fact_lock.get(&fact_key).ok_or(funs.err().not_found(
        "fact_record",
        "load_set",
        &format!("The fact instance table [{}] not exists.", &fact_key),
        "404-spi-stats-fact-inst-not-exist",
    ))?;

    let mut has_fields_init = false;
    let mut fields = vec!["key".to_string(), "own_paths".to_string(), "st".to_string()];
    let mut value_sets = vec![];

    for add_req in add_req_set {
        let req_data = add_req.data.as_object().unwrap();
        let mut values = vec![Value::from(&add_req.key), Value::from(add_req.own_paths), Value::from(add_req.ct)];

        for fact_col_conf in fact_col_conf_set {
            let req_fact_col_value = req_data.get(&fact_col_conf.key).ok_or(funs.err().bad_request(
                "fact_record",
                "load_set",
                &format!(
                    "The fact instance record [{}][{}] is missing a required column [{}].",
                    fact_key, add_req.key, fact_col_conf.key
                ),
                "400-spi-stats-fact-inst-record-missing-column",
            ))?;

            if !has_fields_init {
                fields.push(fact_col_conf.key.to_string());
            }

            if fact_col_conf.kind == StatsFactColKind::Dimension && CONF_DIMS.read().await.get(fact_col_conf.dim_rel_conf_dim_key.as_ref().unwrap()).unwrap().stable_ds {
                let dim_record_id = match funs.init(ctx, true, stats_initializer::init_fun).await?.as_str() {
                    #[cfg(feature = "spi-pg")]
                    spi_constants::SPI_PG_KIND_CODE => {
                        // TODO support other data type
                        pg::stats_pg_record_serv::dim_get_inst_record_id(fact_col_conf.dim_rel_conf_dim_key.as_ref().unwrap(), req_fact_col_value.as_str().unwrap(), funs, ctx)
                            .await?
                    }
                    kind_code => return Err(funs.bs_not_implemented(kind_code)),
                }
                .ok_or(funs.err().not_found(
                    "fact_record",
                    "load_set",
                    &format!(
                        "The parent dimension instance record [{}] not exists.",
                        &fact_col_conf.dim_rel_conf_dim_key.as_ref().unwrap()
                    ),
                    "404-spi-stats-dim-inst-record-not-exist",
                ))?;
                // Replace dimension instance record id to dimension instance record key
                if !has_fields_init {
                    fields.push(fact_col_conf.key.to_string());
                }
                values.push(Value::from(dim_record_id));
            } else if fact_col_conf.kind == StatsFactColKind::Dimension {
                if !has_fields_init {
                    fields.push(fact_col_conf.key.to_string());
                }
                if fact_col_conf.dim_multi_values.unwrap_or(false) {
                    values.push(Value::from(
                        req_fact_col_value.as_array().unwrap().iter().map(|v| v.as_str().unwrap().to_string()).collect::<Vec<String>>(),
                    ));
                } else {
                    values.push(Value::from(req_fact_col_value.as_str().unwrap()));
                }
            } else {
                if !has_fields_init {
                    fields.push(fact_col_conf.key.to_string());
                }
                match fact_col_conf.mes_data_type {
                    Some(StatsDataTypeKind::Number) => values.push(Value::from(req_fact_col_value.as_i64().unwrap())),
                    Some(StatsDataTypeKind::Boolean) => values.push(Value::from(req_fact_col_value.as_bool().unwrap())),
                    Some(StatsDataTypeKind::DateTime) => values.push(Value::from(req_fact_col_value.as_str().unwrap())),
                    Some(StatsDataTypeKind::Date) => values.push(Value::from(req_fact_col_value.as_str().unwrap())),
                    _ => values.push(Value::from(req_fact_col_value.as_str().unwrap())),
                }
            }
            // TODO check data type
        }
        value_sets.push(values);
        has_fields_init = true;
    }

    match funs.init(ctx, true, stats_initializer::init_fun).await?.as_str() {
        #[cfg(feature = "spi-pg")]
        spi_constants::SPI_PG_KIND_CODE => pg::stats_pg_record_serv::fact_load_records(&fact_key, fields, value_sets, funs, ctx).await,
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
}

pub(crate) async fn fact_delete_records(fact_key: String, delete_keys: Vec<String>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    let conf_fact_lock = CONF_FACTS.read().await;
    let (_, _) = conf_fact_lock.get(&fact_key).ok_or(funs.err().not_found(
        "fact_record",
        "delete_set",
        &format!("The fact instance table [{}] not exists.", &fact_key),
        "404-spi-stats-fact-inst-not-exist",
    ))?;

    match funs.init(ctx, true, stats_initializer::init_fun).await?.as_str() {
        #[cfg(feature = "spi-pg")]
        spi_constants::SPI_PG_KIND_CODE => pg::stats_pg_record_serv::fact_delete_records(&fact_key, &delete_keys, funs, ctx).await,
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
}

pub(crate) async fn fact_clean_records(fact_key: String, before_ct: Option<DateTime<Utc>>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    let conf_fact_lock = CONF_FACTS.read().await;
    let (_, _) = conf_fact_lock.get(&fact_key).ok_or(funs.err().not_found(
        "fact_record",
        "clean",
        &format!("The fact instance table [{}] not exists.", &fact_key),
        "404-spi-stats-fact-inst-not-exist",
    ))?;

    match funs.init(ctx, true, stats_initializer::init_fun).await?.as_str() {
        #[cfg(feature = "spi-pg")]
        spi_constants::SPI_PG_KIND_CODE => pg::stats_pg_record_serv::fact_clean_records(&fact_key, before_ct, funs, ctx).await,
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
}

pub(crate) async fn dim_add_record(dim_conf_key: String, record_key: String, add_req: StatsDimRecordAddReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    let conf_dim_lock = CONF_DIMS.read().await;
    let dim_conf = conf_dim_lock.get(&dim_conf_key).ok_or(funs.err().not_found(
        "dim_record",
        "add",
        &format!("The dimension instance table [{}] not exists.", &dim_conf_key),
        "404-spi-stats-dim-inst-not-exist",
    ))?;
    if dim_conf.hierarchy.is_empty() && add_req.parent_key.is_some() {
        return Err(funs.err().bad_request(
            "dim_record",
            "add",
            &format!("The dimension config [{}] not allow hierarchy.", &dim_conf_key),
            "400-spi-stats-dim-conf-not-hierarchy",
        ));
    }
    match funs.init(ctx, true, stats_initializer::init_fun).await?.as_str() {
        #[cfg(feature = "spi-pg")]
        spi_constants::SPI_PG_KIND_CODE => pg::stats_pg_record_serv::dim_add_record(record_key, add_req, dim_conf, funs, ctx).await,
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
}

pub(crate) async fn dim_delete_record(dim_conf_key: String, record_key: String, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    let conf_dim_lock = CONF_DIMS.read().await;
    let dim_conf = conf_dim_lock.get(&dim_conf_key).ok_or(funs.err().not_found(
        "dim_record",
        "delete",
        &format!("The dimension instance table [{}] not exists.", &dim_conf_key),
        "404-spi-stats-dim-inst-not-exist",
    ))?;
    match funs.init(ctx, true, stats_initializer::init_fun).await?.as_str() {
        #[cfg(feature = "spi-pg")]
        spi_constants::SPI_PG_KIND_CODE => pg::stats_pg_record_serv::dim_delete_record(record_key, dim_conf, funs, ctx).await,
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
}
