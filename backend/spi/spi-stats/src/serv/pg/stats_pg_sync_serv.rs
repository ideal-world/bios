use std::collections::HashMap;

use bios_basic::{
    rbum::{
        dto::{
            rbum_cert_dto::{RbumCertAddReq, RbumCertModifyReq},
            rbum_filer_dto::{RbumBasicFilterReq, RbumCertFilterReq},
        },
        rbum_enumeration::{RbumCertRelKind, RbumCertStatusKind},
        serv::{rbum_cert_serv::RbumCertServ, rbum_crud_serv::RbumCrudOperation},
    },
    spi::{
        spi_constants::SPI_PG_KIND_CODE,
        spi_funs::SpiBsInst,
        spi_initializer::common_pg::{self, package_table_name},
    },
};
use tardis::{
    basic::{dto::TardisContext, error::TardisError, field::TrimString, result::TardisResult},
    config::config_dto::{CompatibleType, DBModuleConfig},
    db::{
        reldb_client::{TardisRelDBClient, TardisRelDBlConnection},
        sea_orm::{QueryResult, Value},
    },
    log,
    regex::Regex,
    TardisFunsInst,
};

use crate::{
    dto::stats_conf_dto::{StatsSyncDbConfigAddReq, StatsSyncDbConfigExt, StatsSyncDbConfigInfoResp, StatsSyncDbConfigInfoWithSkResp, StatsSyncDbConfigModifyReq},
    stats_constants::DOMAIN_CODE,
    stats_enumeration::{StatsDataType, StatsDataTypeKind, StatsFactColKind},
};

use super::{stats_pg_conf_dim_serv, stats_pg_conf_fact_col_serv};

pub(crate) async fn db_config_add(add_req: StatsSyncDbConfigAddReq, funs: &TardisFunsInst, ctx: &TardisContext, _inst: &SpiBsInst) -> TardisResult<String> {
    // 使用rel_rbum_id kind supplier 来作为unique key
    let mut rbum_cert_add_req = RbumCertAddReq {
        ak: TrimString(add_req.db_user),
        sk: Some(TrimString(add_req.db_password)),
        conn_uri: Some(add_req.db_url),
        rel_rbum_id: "".to_string(),
        kind: Some(SPI_PG_KIND_CODE.to_string()),
        supplier: Some(DOMAIN_CODE.to_string()),
        ext: serde_json::to_string(&StatsSyncDbConfigExt {
            max_connections: add_req.max_connections,
            min_connections: add_req.min_connections,
        })
        .ok(),
        sk_invisible: None,
        ignore_check_sk: false,
        start_time: None,
        end_time: None,
        status: RbumCertStatusKind::Enabled,
        vcode: None,
        rel_rbum_cert_conf_id: None,
        rel_rbum_kind: RbumCertRelKind::Item,
        is_outside: true,
    };
    let rbum_cert = RbumCertServ::add_rbum(&mut rbum_cert_add_req, funs, ctx).await?;
    Ok(rbum_cert)
}

pub(crate) async fn db_config_modify(modify_req: StatsSyncDbConfigModifyReq, funs: &TardisFunsInst, ctx: &TardisContext, _inst: &SpiBsInst) -> TardisResult<()> {
    if RbumCertServ::find_one_rbum(
        &RbumCertFilterReq {
            basic: RbumBasicFilterReq {
                ids: Some(vec![modify_req.id.clone()]),
                ..Default::default()
            },
            ..Default::default()
        },
        funs,
        ctx,
    )
    .await?
    .is_some()
    {
        let mut rbum_cert_modify_req = RbumCertModifyReq {
            ak: modify_req.db_user.map(TrimString),
            sk: modify_req.db_password.map(TrimString),
            conn_uri: modify_req.db_url,
            sk_invisible: None,
            ignore_check_sk: false,
            ext: None,
            start_time: None,
            end_time: None,
            status: None,
        };
        RbumCertServ::modify_rbum(&modify_req.id, &mut rbum_cert_modify_req, funs, ctx).await?;
    } else {
        return Err(funs.err().not_found(&RbumCertServ::get_obj_name(), "modify", "rbum cert not found", "404-rbum-cert-not-found"));
    }
    Ok(())
}

pub(crate) async fn db_config_list(funs: &TardisFunsInst, ctx: &TardisContext, inst: &SpiBsInst) -> TardisResult<Vec<StatsSyncDbConfigInfoResp>> {
    let rbum_cert_list = RbumCertServ::find_detail_rbums(
        &RbumCertFilterReq {
            kind: Some(SPI_PG_KIND_CODE.to_string()),
            suppliers: Some(vec![DOMAIN_CODE.to_string()]),
            ..Default::default()
        },
        None,
        None,
        funs,
        ctx,
    )
    .await?;

    return Ok(rbum_cert_list
        .iter()
        .map(|rbum_cert| {
            let ext = serde_json::from_str::<StatsSyncDbConfigExt>(&rbum_cert.ext).ok();
            StatsSyncDbConfigInfoResp {
                id: rbum_cert.id.clone(),
                db_url: rbum_cert.conn_uri.clone(),
                db_user: rbum_cert.ak.clone(),
                max_connections: ext.clone().and_then(|ext| ext.max_connections),
                min_connections: ext.clone().and_then(|ext| ext.min_connections),
            }
        })
        .collect());
}

async fn get_db_conn_by_cert_id(cert_id: &str, funs: &TardisFunsInst, ctx: &TardisContext, inst: &SpiBsInst) -> TardisResult<TardisRelDBlConnection> {
    let db_config = get_db_config(cert_id, funs, ctx, inst).await?;
    let data_source_conn = TardisRelDBClient::init(&DBModuleConfig {
        url: format!("postgres://{}:{}@{}", db_config.db_user, db_config.db_password, db_config.db_url),
        max_connections: db_config.max_connections.unwrap_or(20),
        min_connections: db_config.min_connections.unwrap_or(5),
        connect_timeout_sec: None,
        idle_timeout_sec: None,
        compatible_type: CompatibleType::default(),
    })
    .await?
    .conn();
    Ok(data_source_conn)
}

async fn get_db_config(cert_id: &str, funs: &TardisFunsInst, ctx: &TardisContext, _inst: &SpiBsInst) -> TardisResult<StatsSyncDbConfigInfoWithSkResp> {
    if let Some(rbum_cert) = RbumCertServ::find_one_detail_rbum(
        &RbumCertFilterReq {
            basic: RbumBasicFilterReq {
                ids: Some(vec![cert_id.to_string()]),
                ..Default::default()
            },
            kind: Some(SPI_PG_KIND_CODE.to_string()),
            suppliers: Some(vec![DOMAIN_CODE.to_string()]),
            ..Default::default()
        },
        funs,
        ctx,
    )
    .await?
    {
        let db_password = RbumCertServ::show_sk(cert_id, &RbumCertFilterReq::default(), funs, ctx).await?;
        let ext = serde_json::from_str::<StatsSyncDbConfigExt>(&rbum_cert.ext).ok();
        let max_connections = ext.clone().and_then(|ext| ext.max_connections);
        let min_connections = ext.clone().and_then(|ext| ext.min_connections);
        Ok(StatsSyncDbConfigInfoWithSkResp {
            id: cert_id.to_string(),
            db_url: rbum_cert.conn_uri.clone(),
            db_user: rbum_cert.ak.clone(),
            db_password,
            max_connections,
            min_connections,
        })
    } else {
        return Err(funs.err().not_found(&RbumCertServ::get_obj_name(), "find", "rbum cert not found", "404-rbum-cert-not-found"));
    }
}

pub(crate) async fn fact_record_sync(fact_key: &str, funs: &TardisFunsInst, ctx: &TardisContext, inst: &SpiBsInst) -> TardisResult<()> {
    let bs_inst = inst.inst::<TardisRelDBClient>();
    let (mut conn, _) = common_pg::init_conn(bs_inst).await?;

    let Some(fact_conf) = conn.query_one("SELECT rel_cert_id,sync_sql FROM starsys_stats_conf_fact WHERE key = $1", vec![Value::from(fact_key)]).await? else {
        return Err(funs.err().not_found("starsys_stats_conf_fact", "find", "fact conf not found", "404-fact-conf-not-found"));
    };

    if let Some(cert_id) = fact_conf.try_get::<Option<String>>("", "rel_cert_id")? {
        if let Some(sync_sql) = fact_conf.try_get::<Option<String>>("", "sync_sql")? {
            let db_source_conn = get_db_conn_by_cert_id(&cert_id, funs, ctx, inst).await?;
            let select_param_fields = find_select_param_fields_from_sql(&sync_sql);
            if select_param_fields.is_empty() {
                return Err(funs.err().bad_request("starsys_stats_conf_fact", "sync", "sync_sql is not a valid sql", "400-spi-stats-sync-sql-not-valid"));
            }
            let select_param_fields_type = find_param_fields_type(fact_key, &select_param_fields, funs, ctx, inst).await.map_err(|_| {
                funs.err().bad_request(
                    "starsys_stats_conf_fact_col",
                    "find",
                    "Fail to get param fields type",
                    "400-spi-stats-fail-to-get-param-fields-type",
                )
            })?;

            let table_name = package_table_name(&format!("stats_inst_fact_{fact_key}"), ctx);

            let db_source_list = db_source_conn.query_all(&sync_sql, vec![]).await?;
            for db_source_record in db_source_list {
                let (mut new_conn, _) = common_pg::init_conn(bs_inst).await?;
                new_conn.begin().await?;
                let mut select_param_value_map = HashMap::new();
                for select_param_field in &select_param_fields {
                    let value = select_param_fields_type
                        .get(select_param_field)
                        .expect("should have select param field type")
                        .result_to_sea_orm_value(&db_source_record, select_param_field)?;
                    select_param_value_map.insert(select_param_field.clone(), value);
                }
                // check idempotent_id record is exist
                if let Some(idempotent_id) = select_param_value_map.get("idempotent_id") {
                    let (sql, params);
                    if new_conn.query_one(&format!("SELECT key FROM {table_name} WHERE idempotent_id = $1"), vec![idempotent_id.clone()]).await?.is_some() {
                        //then update
                        (sql, params) = generate_fact_sql_and_params(true, &table_name, &select_param_fields, &select_param_value_map, idempotent_id)?;
                    } else {
                        //then insert
                        (sql, params) = generate_fact_sql_and_params(false, &table_name, &select_param_fields, &select_param_value_map, idempotent_id)?;
                    }
                    new_conn.execute_one(&sql, params).await?;
                    new_conn.commit().await?;
                } else {
                    log::warn!("[spi-stats] idempotent_id not found for fact: {}", fact_key);
                }
            }
        } else {
            log::warn!("[spi-stats] sync_sql not found for fact: {}", fact_key);
        }
    } else {
        log::warn!("[spi-stats] cert_id not found for fact: {}", fact_key);
    }

    let fact_col_list = conn.query_all("SELECT key FROM starsys_stats_conf_fact_col WHERE rel_conf_fact_key = $1", vec![Value::from(fact_key)]).await?;
    for col in fact_col_list.iter() {
        let col_key = col.try_get::<String>("", "key")?;
        do_fact_col_record_sync(fact_key, &col_key, &mut conn, funs, ctx, inst).await?;
    }

    Ok(())
}

pub(crate) async fn fact_col_record_sync(fact_key: &str, col_key: &str, funs: &TardisFunsInst, ctx: &TardisContext, inst: &SpiBsInst) -> TardisResult<()> {
    let bs_inst = inst.inst::<TardisRelDBClient>();
    let (mut conn, _) = common_pg::init_conn(bs_inst).await?;
    conn.begin().await?;
    do_fact_col_record_sync(fact_key, col_key, &mut conn, funs, ctx, inst).await?;
    conn.commit().await?;
    Ok(())
}

pub(crate) async fn do_fact_col_record_sync(
    fact_conf_key: &str,
    col_conf_key: &str,
    conn: &mut TardisRelDBlConnection,
    funs: &TardisFunsInst,
    ctx: &TardisContext,
    inst: &SpiBsInst,
) -> TardisResult<()> {
    let Some(fact_col) = conn
        .query_one(
            "SELECT key,rel_cert_id,rel_sql,rel_field FROM starsys_stats_conf_fact_col WHERE rel_conf_fact_key = $1 AND key = $2",
            vec![Value::from(fact_conf_key), Value::from(col_conf_key)],
        )
        .await?
    else {
        return Err(funs.err().not_found("starsys_stats_conf_fact_col", "find", "fact col not found", "404-fact-col-not-found"));
    };
    let fact_col_key = fact_col.try_get::<String>("", "key")?;
    if let Some(cert_id) = fact_col.try_get::<Option<String>>("", "rel_cert_id")? {
        if let Some(sql) = fact_col.try_get::<Option<String>>("", "rel_sql")? {
            let data_source_conn = get_db_conn_by_cert_id(&cert_id, funs, ctx, inst).await?;

            let table_name = package_table_name(&format!("stats_inst_fact_{fact_conf_key}"), ctx);
            let param_fields = find_param_fields_from_sql(&sql);
            let select_param_field = find_single_select_param_fields_from_sql(&sql, funs)?;

            let mut all_param_fields = vec![select_param_field.clone()];
            all_param_fields.extend(param_fields.clone());
            let param_fields_type = find_param_fields_type(fact_conf_key, &all_param_fields, funs, ctx, inst).await.map_err(|_| {
                funs.err().bad_request(
                    "starsys_stats_conf_fact_col",
                    "find",
                    "Fail to get param fields type",
                    "400-spi-stats-fail-to-get-param-fields-type",
                )
            })?;

            let mut page_number = 1;
            let page_size = 500;
            loop {
                let bs_inst = inst.inst::<TardisRelDBClient>();
                let (mut conn, _) = common_pg::init_conn(bs_inst).await?;
                conn.begin().await?;
                let fact_record_list = conn
                    .query_all(
                        &format!(
                            "SELECT key,{},count(*) OVER() AS total  FROM {table_name} LIMIT {} OFFSET {}",
                            param_fields.join(","),
                            page_size,
                            (page_number - 1) * page_size
                        ),
                        vec![Value::from(fact_conf_key), Value::from(col_conf_key)],
                    )
                    .await?;

                for fact_record in &fact_record_list {
                    let key = fact_record.try_get::<String>("", "key")?;
                    let (sql, params) = generate_sql_and_params(&sql, &param_fields, &param_fields_type, fact_record)?;
                    if let Some(rel_record) = data_source_conn.query_one(&sql, params).await? {
                        let rel_record_value =
                            param_fields_type.get(&select_param_field).expect("should have select param field type").result_to_sea_orm_value(&rel_record, &select_param_field)?;
                        conn.execute_one(
                            &format!("INSERT INTO {table_name} ({fact_col_key}) VALUES ($1) where key = $2"),
                            vec![rel_record_value, Value::from(key)],
                        )
                        .await?;
                    }
                }

                if fact_record_list.len() < page_size || fact_record_list[0].try_get::<i64>("", "total")? <= page_size as i64 * (page_number - 1) as i64 {
                    break;
                }
                page_number += 1;
                conn.commit().await?;
            }
        } else {
            log::warn!("[spi-stats] cert_id not found for fact_col: {}", col_conf_key);
        }
    } else {
        log::warn!("[spi-stats] cert_id not found for fact_col: {}", col_conf_key);
    }
    Ok(())
}

/// find single select param fields from sql
/// eg. 'select id from table where id= ${app_id} and name = ${name}'  return 'id'
fn find_single_select_param_fields_from_sql(sql: &str, funs: &TardisFunsInst) -> TardisResult<String> {
    let param_fields = find_select_param_fields_from_sql(sql);
    if param_fields.is_empty() {
        return Err(funs.err().bad_request(
            "starsys_stats_conf_fact_col",
            "find",
            "The rel_sql is not a valid sql.",
            "400-spi-stats-fact-col-conf-rel-sql-not-valid",
        ));
    }
    if param_fields.len() > 1 {
        return Err(funs.err().bad_request(
            "starsys_stats_conf_fact_col",
            "find",
            "The rel_sql is not a valid sql.",
            "400-spi-stats-fact-col-conf-rel-sql-not-valid",
        ));
    }
    Ok(param_fields.first().cloned().expect("should have one param field"))
}

/// find select param fields from sql
/// eg. 'select id,address from table where id= ${app_id} and name = ${name}'  return 'id','address'
/// eg. 'select id,test as address from table where id= ${app_id} and name = ${name}'  return 'id','address'
/// eg. 'select * from table where id= ${app_id} and name = ${name}'  return ''
fn find_select_param_fields_from_sql(sql: &str) -> Vec<String> {
    let re = Regex::new(r"select\s+([^\*]+?)\s+from").expect("should compile regex");
    let params = re.captures(&sql.to_ascii_lowercase()).map(|cap| cap[1].to_string()).unwrap_or("".to_string());
    params
        .split(',')
        .map(|s| {
            let a = s.split("as").collect::<Vec<&str>>();
            if a.len() > 1 {
                a[1].trim().to_string()
            } else {
                s.trim().to_string()
            }
        })
        .collect()
}

/// find param fields from sql
/// eg. 'where id= ${app_id} and name = ${name}'  return 'app_id','name'
fn find_param_fields_from_sql(sql: &str) -> Vec<String> {
    let re = Regex::new(r"\$\{([^}]+)\}").expect("should compile regex");
    let params = re.captures_iter(sql).map(|cap| cap[1].to_string()).collect();
    params
}

/// find param fields type
/// if param field is dimension, get data_type from dim_conf
/// if param field is measure, get data_type from fact_col
async fn find_param_fields_type(
    fact_conf_key: &str,
    param_fields: &Vec<String>,
    funs: &TardisFunsInst,
    ctx: &TardisContext,
    inst: &SpiBsInst,
) -> TardisResult<HashMap<String, StatsDataType>> {
    let bs_inst = inst.inst::<TardisRelDBClient>();
    let (conn, _) = common_pg::init_conn(bs_inst).await?;

    let mut param_fields_type = HashMap::new();
    let fact_col_list = stats_pg_conf_fact_col_serv::find_by_fact_conf_key(fact_conf_key, funs, ctx, inst).await?;
    for fact_col in fact_col_list {
        if param_fields.contains(&fact_col.key) {
            if StatsFactColKind::Dimension == fact_col.kind {
                let Some(dim_conf_key) = &fact_col.dim_rel_conf_dim_key else {
                    return Err(funs.err().bad_request("fact_inst", "create", "Fail to get dimension config", "400-spi-stats-fail-to-get-dim-config-key"));
                };

                let Some(dim_conf) = stats_pg_conf_dim_serv::get(dim_conf_key, None, None, &conn, ctx, inst).await? else {
                    return Err(funs.err().conflict(
                        "fact_inst",
                        "create",
                        &format!("Fail to get dimension config by key [{dim_conf_key}]"),
                        "409-spi-stats-fail-to-get-dim-config",
                    ));
                };
                if fact_col.dim_multi_values.unwrap_or(false) {
                    param_fields_type.insert(fact_col.key.clone(), StatsDataType::Array(dim_conf.data_type));
                } else {
                    param_fields_type.insert(fact_col.key.clone(), StatsDataType::Single(dim_conf.data_type));
                }
            } else if StatsFactColKind::Measure == fact_col.kind {
                let data_type = fact_col.mes_data_type.unwrap_or(StatsDataTypeKind::String);
                param_fields_type.insert(fact_col.key.clone(), StatsDataType::Single(data_type));
            }
        }
    }
    Ok(param_fields_type)
}

/// generate fact sql and params logic
fn generate_fact_sql_and_params(
    is_update: bool,
    table_name: &str,
    param_fields: &Vec<String>,
    param_fields_type: &HashMap<String, Value>,
    idempotent_id: &Value,
) -> TardisResult<(String, Vec<Value>)> {
    if is_update {
        let mut set_sql = String::new();
        let mut params = vec![];
        for (i, field) in param_fields.iter().enumerate() {
            if i < param_fields.len() - 1 {
                set_sql.push_str(&format!("{field} = ${},", i + 1));
            } else {
                set_sql.push_str(&format!("{field} = ${}", i + 1));
            }
            params.push(param_fields_type.get(field).expect("should have param field type").clone());
        }
        let sql = format!("UPDATE {table_name} SET {set_sql} WHERE idempotent_id = ${}", param_fields.len() + 1);
        params.push(idempotent_id.clone());
        Ok((sql, params))
    } else {
        let mut fields_sql = String::new();
        let mut values_sql = String::new();
        let mut params = vec![];
        for (i, field) in param_fields.iter().enumerate() {
            if i < param_fields.len() - 1 {
                fields_sql.push_str(&format!("{field},"));
                values_sql.push_str(&format!("${},", i + 1));
            } else {
                fields_sql.push_str(field);
                values_sql.push_str(&format!("${}", i + 1));
            }
            params.push(param_fields_type.get(field).expect("should have param field type").clone());
        }
        let sql = format!("INSERT INTO {table_name} ({fields_sql}) VALUES ({values_sql})");
        Ok((sql, params))
    }
}

/// generate sql and params by sql and param fields
fn generate_sql_and_params(
    sql: &str,
    param_fields: &Vec<String>,
    param_fields_type: &HashMap<String, StatsDataType>,
    fact_record: &QueryResult,
) -> TardisResult<(String, Vec<Value>)> {
    let fact_record_map = param_fields
        .iter()
        .map(|field| {
            Ok((
                field.clone(),
                param_fields_type.get(field).expect("should have param field type").result_to_sea_orm_value(fact_record, field)?,
            ))
        })
        .collect::<TardisResult<HashMap<_, _>>>()?;
    do_generate_sql_and_params(sql, param_fields, &fact_record_map)
}

fn do_generate_sql_and_params(sql: &str, param_fields: &Vec<String>, fact_record: &HashMap<String, Value>) -> TardisResult<(String, Vec<Value>)> {
    let mut sql = sql.to_string();
    let mut params = vec![];
    for (i, param) in param_fields.iter().enumerate() {
        sql = sql.replace(&format!("${{{}}}", param), &format!("${}", i + 1));
        params.push(fact_record.get(param).unwrap_or_else(|| panic!("param {} not found", param)).clone());
    }
    Ok((sql, params))
}

/// validate fact sql
/// validate sql is select statement and not select *
pub(crate) fn validate_fact_sql(sql: &str) -> TardisResult<bool> {
    let re = Regex::new(r"^select\s+[^*][\w\s,]+\s+from").expect("should compile regex");
    if re.is_match(&sql.trim().to_lowercase()) {
        let param_fields = find_param_fields_from_sql(sql);
        if param_fields.contains(&"idempotent_id".to_string()) {
            return Ok(true);
        } else {
            return Err(TardisError::bad_request(
                "[spi-stats] The sync_sql must contain idempotent_id",
                "400-spi-stats-sync-sql-must-contain-idempotent-id",
            ));
        }
    }
    Ok(false)
}

/// validate fact col sql
/// validate sql is select statement and only select one field
pub(crate) fn validate_fact_col_sql(sql: &str) -> bool {
    let re = Regex::new(r"^select\s+\$([^,]+)\s+from").expect("should compile regex");
    re.is_match(sql)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use tardis::{
        chrono::{DateTime, Utc},
        db::sea_orm::Value,
    };

    use crate::serv::pg::stats_pg_sync_serv::{
        do_generate_sql_and_params, find_select_param_fields_from_sql, generate_fact_sql_and_params, validate_fact_col_sql, validate_fact_sql,
    };

    use super::find_param_fields_from_sql;
    #[test]
    fn test_find_select_params_from_sql() {
        let sql = "select id,address from table where id= ${app_id} and name = ${name}";
        let params = find_select_param_fields_from_sql(sql);
        assert_eq!(params, vec!["id", "address"]);
        let sql = "select id,test as address from table where id= ${app_id} and name = ${name}";
        let params = find_select_param_fields_from_sql(sql);
        assert_eq!(params, vec!["id", "address"]);
        let sql = "SELECT id,test AS address FROM table WHERE id= ${app_id} and name = ${name}";
        let params = find_select_param_fields_from_sql(sql);
        assert_eq!(params, vec!["id", "address"]);
        let sql = "SELECT id,ext->>'address' AS address FROM table WHERE id= ${app_id} and name = ${name}";
        let params = find_select_param_fields_from_sql(sql);
        assert_eq!(params, vec!["id", "address"]);
        let sql = "select * from table where id= ${app_id} and name = ${name}";
        let params = find_select_param_fields_from_sql(sql);
        assert_eq!(params, vec![""]);
    }

    #[test]
    fn test_find_params_from_sql() {
        let sql = "select id,address from table where id= ${app_id} and name = ${name}";
        let params = find_param_fields_from_sql(sql);
        assert_eq!(params, vec!["app_id", "name"]);
        let sql = "select * from table inner join table2 on table.id = table2.id and table.name = ${name} where id = ${app_id}";
        let params = find_param_fields_from_sql(sql);
        assert_eq!(params, vec!["name", "app_id"]);
        let sql = "select * from table";
        let params = find_param_fields_from_sql(sql);
        assert_eq!(params, Vec::<String>::new());
    }

    #[test]
    fn test_validate_fact_sql() {
        let sql = "select id from table";
        assert!(validate_fact_sql(sql).unwrap());
        let sql = "select id,name from table";
        assert!(validate_fact_sql(sql).unwrap());
        let sql = "select * from table";
        assert!(validate_fact_sql(sql).unwrap());
        let sql = "update table set id = ${id} where id = ${id}";
        assert!(validate_fact_sql(sql).unwrap());
    }

    #[test]
    fn test_validate_fact_col_sql() {
        let sql = "select id from table";
        assert!(validate_fact_col_sql(sql));
        let sql = "select id,name from table";
        assert!(validate_fact_col_sql(sql));
        let sql = "update table set id = ${id} where id = ${id}";
        assert!(validate_fact_col_sql(sql));
    }

    #[test]
    fn test_generate_fact_sql_and_params() {
        let table_name = "table";
        let param_fields = vec!["idempotent_id".to_string(), "name".to_string(), "age".to_string(), "ct".to_string()];
        let mut param_fields_type = HashMap::new();
        param_fields_type.insert("idempotent_id".to_string(), Value::from("1"));
        param_fields_type.insert("name".to_string(), Value::from("name1"));
        param_fields_type.insert("age".to_string(), Value::from(18));
        param_fields_type.insert("ct".to_string(), Value::from(DateTime::<Utc>::from_timestamp(1715260800, 0)));
        let idempotent_id = Value::from("1");
        let (sql, params) = generate_fact_sql_and_params(true, table_name, &param_fields, &param_fields_type, &idempotent_id).unwrap();
        assert_eq!(sql, "UPDATE table SET idempotent_id = $1,name = $2,age = $3,ct = $4 WHERE idempotent_id = $5");
        assert_eq!(
            params,
            vec![
                Value::from("1"),
                Value::from("name1"),
                Value::from(18),
                Value::from(DateTime::<Utc>::from_timestamp(1715260800, 0)),
                Value::from("1")
            ]
        );

        let (sql, params) = generate_fact_sql_and_params(false, table_name, &param_fields, &param_fields_type, &idempotent_id).unwrap();
        assert_eq!(sql, "INSERT INTO table (idempotent_id,name,age,ct) VALUES ($1,$2,$3,$4)");
        assert_eq!(
            params,
            vec![
                Value::from("1"),
                Value::from("name1"),
                Value::from(18),
                Value::from(DateTime::<Utc>::from_timestamp(1715260800, 0))
            ]
        );
    }

    #[test]
    fn test_generate_sql_and_params() {
        let sql = "select id from table where id = ${id} and name = ${name} and age = ${age} and ct = ${ct}";
        let param_fields = find_param_fields_from_sql(sql);
        let mut fact_record = HashMap::new();
        fact_record.insert("ct".to_string(), Value::from(DateTime::<Utc>::from_timestamp(1715260800, 0)));
        fact_record.insert("id".to_string(), Value::from("1"));
        fact_record.insert("age".to_string(), Value::from(18));
        fact_record.insert("name".to_string(), Value::from("name1"));
        let (sql, params) = do_generate_sql_and_params(sql, &param_fields, &fact_record).unwrap();
        assert_eq!(sql, "select id from table where id = $1 and name = $2 and age = $3 and ct = $4");
        assert_eq!(
            params,
            vec![
                Value::from("1"),
                Value::from("name1"),
                Value::from(18),
                Value::from(DateTime::<Utc>::from_timestamp(1715260800, 0))
            ]
        );
    }
}
