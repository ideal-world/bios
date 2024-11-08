use bios_basic::spi::{
    spi_funs::SpiBsInst,
    spi_initializer::common_pg::{self, package_table_name},
};
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    db::{
        reldb_client::{TardisRelDBClient, TardisRelDBlConnection},
        sea_orm::Value,
    },
    web::web_resp::TardisPage,
    TardisFunsInst,
};

use super::stats_pg_initializer;
use crate::{
    dto::stats_conf_dto::{StatsConfDimAddReq, StatsConfDimInfoResp, StatsConfDimModifyReq},
    stats_enumeration::StatsDataTypeKind,
};

pub async fn online(dim_conf_key: &str, conn: &TardisRelDBlConnection, ctx: &TardisContext) -> TardisResult<bool> {
    common_pg::check_table_exit(&format!("stats_inst_dim_{dim_conf_key}"), conn, ctx).await
}

pub(crate) async fn add(add_req: &StatsConfDimAddReq, funs: &TardisFunsInst, ctx: &TardisContext, inst: &SpiBsInst) -> TardisResult<()> {
    let bs_inst = inst.inst::<TardisRelDBClient>();
    let (mut conn, table_name) = stats_pg_initializer::init_conf_dim_table_and_conn(bs_inst, ctx, true).await?;
    conn.begin().await?;
    if conn.count_by_sql(&format!("SELECT 1 FROM {table_name} WHERE key = $1"), vec![Value::from(&add_req.key)]).await? != 0 {
        return Err(funs.err().conflict(
            "dim_conf",
            "add",
            "The dimension config already exists, please delete it and then add it.",
            "409-spi-stats-dim-conf-exist",
        ));
    }
    let params = vec![
        Value::from(add_req.key.to_string()),
        Value::from(add_req.show_name.clone()),
        Value::from(add_req.stable_ds),
        Value::from(add_req.data_type.to_string()),
        Value::from(add_req.hierarchy.as_ref().unwrap_or(&vec![]).clone()),
        Value::from(add_req.remark.as_ref().unwrap_or(&"".to_string()).as_str()),
        Value::from(add_req.dim_group_key.as_deref()),
        Value::from(add_req.dynamic_url.as_deref()),
        Value::from(add_req.is_tree.unwrap_or(false)),
        Value::from(add_req.tree_dynamic_url.as_deref()),
        Value::from(add_req.rel_attribute_code.as_ref().unwrap_or(&vec![]).clone()),
        Value::from(add_req.rel_attribute_url.as_deref()),
    ];

    conn.execute_one(
        &format!(
            r#"INSERT INTO {table_name}
(key, show_name, stable_ds, data_type, hierarchy, remark, dim_group_key, dynamic_url, is_tree, tree_dynamic_url, rel_attribute_code, rel_attribute_url)
VALUES
($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
"#,
        ),
        params,
    )
    .await?;
    conn.commit().await?;
    Ok(())
}

pub(crate) async fn modify(dim_conf_key: &str, modify_req: &StatsConfDimModifyReq, funs: &TardisFunsInst, ctx: &TardisContext, inst: &SpiBsInst) -> TardisResult<()> {
    let bs_inst = inst.inst::<TardisRelDBClient>();
    let (mut conn, table_name) = stats_pg_initializer::init_conf_dim_table_and_conn(bs_inst, ctx, true).await?;
    conn.begin().await?;
    if online(dim_conf_key, &conn, ctx).await? {
        return Err(funs.err().conflict(
            "dim_conf",
            "modify",
            "The dimension instance table already exists, please delete it and then modify it.",
            "409-spi-stats-dim-inst-exist",
        ));
    }
    let mut sql_sets = vec![];
    let mut params = vec![Value::from(dim_conf_key.to_string())];
    if let Some(show_name) = &modify_req.show_name {
        sql_sets.push(format!("show_name = ${}", params.len() + 1));
        params.push(Value::from(show_name.to_string()));
    }
    if let Some(stable_ds) = modify_req.stable_ds {
        sql_sets.push(format!("stable_ds = ${}", params.len() + 1));
        params.push(Value::from(stable_ds));
    }
    if let Some(data_type) = &modify_req.data_type {
        sql_sets.push(format!("data_type = ${}", params.len() + 1));
        params.push(Value::from(data_type.to_string()));
    }
    if let Some(hierarchy) = &modify_req.hierarchy {
        sql_sets.push(format!("hierarchy = ${}", params.len() + 1));
        params.push(Value::from(hierarchy.clone()));
    }
    if let Some(remark) = &modify_req.remark {
        sql_sets.push(format!("remark = ${}", params.len() + 1));
        params.push(Value::from(remark.to_string()));
    }
    if let Some(dim_group_key) = &modify_req.dim_group_key {
        sql_sets.push(format!("dim_group_key = ${}", params.len() + 1));
        params.push(Value::from(dim_group_key));
    }
    if let Some(dynamic_url) = &modify_req.dynamic_url {
        sql_sets.push(format!("dynamic_url = ${}", params.len() + 1));
        params.push(Value::from(dynamic_url));
    }
    if let Some(is_tree) = modify_req.is_tree {
        sql_sets.push(format!("is_tree = ${}", params.len() + 1));
        params.push(Value::from(is_tree));
    }
    if let Some(tree_dynamic_url) = &modify_req.tree_dynamic_url {
        sql_sets.push(format!("tree_dynamic_url = ${}", params.len() + 1));
        params.push(Value::from(tree_dynamic_url));
    }
    if let Some(rel_attribute_code) = &modify_req.rel_attribute_code {
        sql_sets.push(format!("rel_attribute_code = ${}", params.len() + 1));
        params.push(Value::from(rel_attribute_code.clone()));
    }
    if let Some(rel_attribute_url) = &modify_req.rel_attribute_url {
        sql_sets.push(format!("rel_attribute_url = ${}", params.len() + 1));
        params.push(Value::from(rel_attribute_url));
    }
    conn.execute_one(
        &format!(
            r#"UPDATE {table_name}
SET {}
WHERE key = $1"#,
            sql_sets.join(",")
        ),
        params,
    )
    .await?;
    conn.commit().await?;
    Ok(())
}

pub(crate) async fn delete(dim_conf_key: &str, funs: &TardisFunsInst, ctx: &TardisContext, inst: &SpiBsInst) -> TardisResult<()> {
    let bs_inst = inst.inst::<TardisRelDBClient>();
    let (mut conn, table_name) = stats_pg_initializer::init_conf_dim_table_and_conn(bs_inst, ctx, true).await?;
    let (_, conf_fact_col_table) = stats_pg_initializer::init_conf_fact_col_table_and_conn(bs_inst, ctx, true).await?;
    // check if has fact col relation on this dim key
    conn.begin().await?;
    let query_result = conn
        .count_by_sql(
            &format!("SELECT 1 FROM {conf_fact_col_table} WHERE dim_rel_conf_dim_key = $1"),
            vec![Value::from(dim_conf_key)],
        )
        .await?;
    if query_result != 0 {
        return Err(funs.err().conflict(
            "dim_conf",
            "delete",
            "This dimension config has been used by some other fact config, please delete the fact config first.",
            "409-spi-stats-dim-conf-used",
        ));
    }
    conn.execute_one(&format!("DELETE FROM {table_name} WHERE key = $1"), vec![Value::from(dim_conf_key)]).await?;
    if online(dim_conf_key, &conn, ctx).await? {
        conn.execute_one(&format!("DROP TABLE {}_{dim_conf_key}", package_table_name("stats_inst_dim", ctx)), vec![]).await?;
    }
    conn.commit().await?;
    Ok(())
}

pub(in crate::serv::pg) async fn get(
    dim_conf_key: &str,
    dim_group_key: Option<String>,
    dim_group_is_empty: Option<bool>,
    conn: &TardisRelDBlConnection,
    ctx: &TardisContext,
    inst: &SpiBsInst,
) -> TardisResult<Option<StatsConfDimInfoResp>> {
    do_paginate(Some(dim_conf_key.to_string()), dim_group_key, dim_group_is_empty, None, 1, 1, None, None, conn, ctx, inst).await.map(|page| page.records.into_iter().next())
}

pub(crate) async fn paginate(
    dim_conf_key: Option<String>,
    dim_group_key: Option<String>,
    dim_group_is_empty: Option<bool>,
    show_name: Option<String>,
    page_number: u32,
    page_size: u32,
    desc_by_create: Option<bool>,
    desc_by_update: Option<bool>,
    _funs: &TardisFunsInst,
    ctx: &TardisContext,
    inst: &SpiBsInst,
) -> TardisResult<TardisPage<StatsConfDimInfoResp>> {
    let bs_inst = inst.inst();
    let (conn, _) = stats_pg_initializer::init_conf_dim_table_and_conn(bs_inst, ctx, true).await?;
    do_paginate(
        dim_conf_key,
        dim_group_key,
        dim_group_is_empty,
        show_name,
        page_number,
        page_size,
        desc_by_create,
        desc_by_update,
        &conn,
        ctx,
        inst,
    )
    .await
}

async fn do_paginate(
    dim_conf_key: Option<String>,
    dim_group_key: Option<String>,
    dim_group_is_empty: Option<bool>,
    show_name: Option<String>,
    page_number: u32,
    page_size: u32,
    desc_by_create: Option<bool>,
    desc_by_update: Option<bool>,
    conn: &TardisRelDBlConnection,
    ctx: &TardisContext,
    _inst: &SpiBsInst,
) -> TardisResult<TardisPage<StatsConfDimInfoResp>> {
    let table_name = package_table_name("stats_conf_dim", ctx);
    let mut sql_where = vec!["1 = 1".to_string()];
    let mut sql_order = vec![];
    let mut params: Vec<Value> = vec![Value::from(page_size), Value::from((page_number - 1) * page_size)];
    if let Some(dim_conf_key) = &dim_conf_key {
        sql_where.push(format!("key = ${}", params.len() + 1));
        params.push(Value::from(dim_conf_key.to_string()));
    }
    if let Some(dim_group_key) = &dim_group_key {
        sql_where.push(format!("dim_group_key = ${}", params.len() + 1));
        params.push(Value::from(dim_group_key.to_string()));
    } else {
        if let Some(dim_group_is_empty) = &dim_group_is_empty {
            if *dim_group_is_empty {
                sql_where.push("dim_group_key = ''".to_string());
            }
        }
    }

    if let Some(show_name) = &show_name {
        sql_where.push(format!("show_name LIKE ${}", params.len() + 1));
        params.push(Value::from(format!("%{show_name}%")));
    }
    if let Some(desc_by_create) = desc_by_create {
        sql_order.push(format!("create_time {}", if desc_by_create { "DESC" } else { "ASC" }));
    }
    if let Some(desc_by_update) = desc_by_update {
        sql_order.push(format!("update_time {}", if desc_by_update { "DESC" } else { "ASC" }));
    }

    let result = conn
        .query_all(
            &format!(
                r#"SELECT key, show_name, stable_ds, data_type, hierarchy, remark, dim_group_key, dynamic_url, is_tree, tree_dynamic_url, rel_attribute_code, rel_attribute_url, create_time, update_time, count(*) OVER() AS total
FROM {table_name}
WHERE 
    {}
    {}
    LIMIT $1 OFFSET $2"#,
                sql_where.join(" AND "),
                if sql_order.is_empty() {
                    "".to_string()
                } else {
                    format!("ORDER BY {}", sql_order.join(","))
                }
            ),
            params,
        )
        .await?;

    let mut total_size: i64 = 0;
    let mut final_result = vec![];
    for item in result {
        if total_size == 0 {
            total_size = item.try_get("", "total")?;
        }
        final_result.push(StatsConfDimInfoResp {
            key: item.try_get("", "key")?,
            show_name: item.try_get("", "show_name")?,
            stable_ds: item.try_get("", "stable_ds")?,
            data_type: item.try_get("", "data_type")?,
            hierarchy: item.try_get("", "hierarchy")?,
            remark: item.try_get("", "remark")?,
            create_time: item.try_get("", "create_time")?,
            update_time: item.try_get("", "update_time")?,
            dim_group_key: item.try_get("", "dim_group_key")?,
            online: online(&item.try_get::<String>("", "key")?, conn, ctx).await?,
            dynamic_url: item.try_get("", "dynamic_url")?,
            is_tree: item.try_get("", "is_tree")?,
            tree_dynamic_url: item.try_get("", "tree_dynamic_url")?,
            rel_attribute_code: item.try_get("", "rel_attribute_code")?,
            rel_attribute_url: item.try_get("", "rel_attribute_url")?,
        });
    }
    Ok(TardisPage {
        page_size: page_size as u64,
        page_number: page_number as u64,
        total_size: total_size as u64,
        records: final_result,
    })
}

/// Create dimension instance table.
///
/// 创建维度实例表
///
/// The table name is `starsys_stats_inst_dim_<dimension key>`
/// The table fields are:
/// - key                   the incoming primary key value / 维度实例主键值
/// - show_name             display name / 显示名称
/// - hierarchy             number of hierarchy levels / 层级数
/// - [key0 .. keyN]        when the hierarchy is greater than 0, it indicates the primary key value of each level, which is used for drilling up and down / 当层级大于0时，表示每个层级的主键值，用于上下钻
/// - ct                    create time / 创建时间
/// - et                    expiration time, when the data of a certain dimension is deleted, et will be set as the deletion time / 过期时间，当某个维度的数据被删除时，et会被设置为删除时间
///
/// # Examples
/// ```
/// CREATE TABLE spi617070303031.starsys_stats_inst_dim_address (
///  key character varying NOT NULL,
///  show_name character varying NOT NULL,
///  hierarchy smallint NOT NULL,
///  key0 character varying NOT NULL DEFAULT '',
///  key1 character varying NOT NULL DEFAULT '',
///  key2 character varying NOT NULL DEFAULT '',
///  ct timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
///  et timestamp with time zone
/// )
/// ```
pub(crate) async fn create_inst(dim_conf_key: &str, funs: &TardisFunsInst, ctx: &TardisContext, inst: &SpiBsInst) -> TardisResult<()> {
    let bs_inst = inst.inst::<TardisRelDBClient>();
    let (mut conn, _) = common_pg::init_conn(bs_inst).await?;
    conn.begin().await?;

    let dim_conf = get(dim_conf_key, None, None, &conn, ctx, inst)
        .await?
        .ok_or_else(|| funs.err().not_found("fact_conf", "create_inst", "The dimension config does not exist.", "404-spi-stats-dim-conf-not-exist"))?;

    if online(dim_conf_key, &conn, ctx).await? {
        return Err(funs.err().conflict(
            "dim_inst",
            "create_inst",
            "The dimension instance table already exists, please delete it and then create it.",
            "409-spi-stats-dim-inst-exist",
        ));
    }
    create_inst_table(&dim_conf, &conn, ctx).await?;
    conn.commit().await?;
    Ok(())
}

async fn create_inst_table(dim_conf: &StatsConfDimInfoResp, conn: &TardisRelDBlConnection, ctx: &TardisContext) -> TardisResult<()> {
    let mut sql = vec![];
    let mut index = vec![];
    sql.push(format!("key {} NOT NULL", dim_conf.data_type.to_pg_data_type()));
    index.push(("key", "btree"));
    if dim_conf.data_type == StatsDataTypeKind::DateTime {
        index.push(("date(timezone('UTC', key))", "btree"));
        index.push(("date_part('hour',timezone('UTC', key))", "btree"));
        index.push(("date_part('day',timezone('UTC', key))", "btree"));
    }
    sql.push("show_name character varying NOT NULL".to_string());
    if !dim_conf.hierarchy.is_empty() {
        sql.push("hierarchy smallint NOT NULL".to_string());
    }
    for hierarchy in 0..dim_conf.hierarchy.len() {
        sql.push(format!("key{hierarchy} character varying NOT NULL DEFAULT ''"));
    }
    sql.push("ct timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP".to_string());
    sql.push("et timestamp with time zone".to_string());

    common_pg::init_table(conn, Some(&dim_conf.key), "stats_inst_dim", sql.join(",\r\n").as_str(), index, None, None, ctx).await?;
    Ok(())
}
