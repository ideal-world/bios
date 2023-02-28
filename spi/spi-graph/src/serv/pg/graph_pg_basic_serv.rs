use bios_basic::spi::{spi_funs::SpiBsInstExtractor, spi_initializer};
use itertools::Itertools;
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    db::{
        reldb_client::TardisRelDBClient,
        sea_orm::{self, Value},
    },
    TardisFunsInst,
};

use crate::dto::graph_dto::{GraphNodeVersionResp, GraphRelAddReq, GraphRelDetailResp, GraphRelUpgardeVersionReq};

use super::graph_pg_initializer;

pub async fn add_rel(add_req: &GraphRelAddReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    let bs_inst = funs.bs(ctx).await?.inst::<TardisRelDBClient>();
    let (mut conn, table_name) = graph_pg_initializer::init_table_and_conn(bs_inst, ctx, true).await?;
    conn.begin().await?;
    conn.execute_one(
        &format!(
            r#"INSERT INTO {table_name}
        (tag, from_key, from_version, to_key, to_version, reverse)
    VALUES
        ($1, $2, $3, $4, $5, false)
        "#
        ),
        vec![
            Value::from(add_req.tag.as_str()),
            Value::from(add_req.from_key.to_string()),
            Value::from(add_req.from_version.as_str()),
            Value::from(add_req.to_key.to_string()),
            Value::from(add_req.to_version.as_str()),
        ],
    )
    .await?;
    conn.execute_one(
        &format!(
            r#"INSERT INTO {table_name}
        (tag, from_key, from_version, to_key, to_version, reverse)
    VALUES
        ($1, $2, $3, $4, $5, true)
        "#
        ),
        vec![
            Value::from(add_req.tag.as_str()),
            Value::from(add_req.to_key.to_string()),
            Value::from(add_req.to_version.as_str()),
            Value::from(add_req.from_key.to_string()),
            Value::from(add_req.from_version.as_str()),
        ],
    )
    .await?;
    conn.commit().await?;
    Ok(())
}

pub async fn upgrade_version(upgrade_version_req: &GraphRelUpgardeVersionReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    let mut where_fragments: Vec<String> = Vec::new();
    let mut sql_vals: Vec<Value> = vec![];
    sql_vals.push(Value::from(upgrade_version_req.new_version.to_string()));
    sql_vals.push(Value::from(upgrade_version_req.key.to_string()));
    sql_vals.push(Value::from(upgrade_version_req.old_version.as_str()));
    for del_rel in &upgrade_version_req.del_rels {
        let mut where_del_rel_fragments: Vec<String> = Vec::new();
        if let Some(tag) = &del_rel.tag {
            sql_vals.push(Value::from(tag.as_str()));
            where_del_rel_fragments.push(format!("tag = ${}", sql_vals.len()));
        }
        if let Some(rel_key) = &del_rel.rel_key {
            sql_vals.push(Value::from(rel_key.to_string()));
            where_del_rel_fragments.push(format!("rel_key = ${}", sql_vals.len()));
        }
        if let Some(rel_version) = &del_rel.rel_version {
            sql_vals.push(Value::from(rel_version.to_string()));
            where_del_rel_fragments.push(format!("rel_version = ${}", sql_vals.len()));
        }
        where_fragments.push(where_del_rel_fragments.join(" AND ").to_string());
    }
    let where_fragments = if where_fragments.is_empty() {
        "".to_string()
    } else {
        format!("AND NOT ({})", where_fragments.join(" OR "))
    };

    let bs_inst = funs.bs(ctx).await?.inst::<TardisRelDBClient>();
    let (mut conn, table_name) = graph_pg_initializer::init_table_and_conn(bs_inst, ctx, false).await?;
    conn.begin().await?;
    conn.execute_one(
        format!(
            r#"INSERT INTO {table_name}
SELECT tag, from_key, $1, to_key, to_version, reverse
FROM {table_name}
WHERE from_key = $2 AND from_version = $3 {}"#,
            where_fragments.replace("rel_key", "to_key").replace("rel_version", "to_version"),
        )
        .as_str(),
        sql_vals.clone(),
    )
    .await?;
    conn.execute_one(
        format!(
            r#"INSERT INTO {table_name}
SELECT tag, from_key, from_version, to_key, $1, reverse
FROM {table_name}
WHERE to_key = $2 AND to_version = $3 {}"#,
            where_fragments.replace("rel_key", "from_key").replace("rel_version", "from_version"),
        )
        .as_str(),
        sql_vals,
    )
    .await?;
    conn.commit().await?;
    Ok(())
}

pub async fn find_versions(tag: String, key: String, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Vec<GraphNodeVersionResp>> {
    let bs_inst = funs.bs(ctx).await?.inst::<TardisRelDBClient>();
    let (conn, table_name) = graph_pg_initializer::init_table_and_conn(bs_inst, ctx, true).await?;
    let result = conn
        .find_dtos_by_sql(
            &format!(
                r#"SELECT DISTINCT ON(from_version) from_version AS version, ts FROM {table_name}
            WHERE tag = $1 AND from_key = $2 AND reverse = false
            ORDER BY from_version DESC
                "#
            ),
            vec![Value::from(tag.as_str()), Value::from(key.as_str())],
        )
        .await?;
    Ok(result)
}

pub async fn find_rels(from_key: String, from_version: String, depth: Option<u8>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<GraphRelDetailResp> {
    let bs_inst = funs.bs(ctx).await?.inst::<TardisRelDBClient>();
    let mut sql_vals: Vec<Value> = vec![];
    if let Some(schema_name) = spi_initializer::common_pg::get_schema_name_from_ext(bs_inst.1) {
        sql_vals.push(Value::from(schema_name));
    } else {
        sql_vals.push(Value::from("public".to_string()));
    }
    sql_vals.push(Value::from(from_key.as_str()));
    sql_vals.push(Value::from(from_version.as_str()));
    if let Some(depth) = depth {
        sql_vals.push(Value::from(depth));
    }
    let (conn, _) = graph_pg_initializer::init_table_and_conn(bs_inst, ctx, false).await?;
    let result = conn.find_dtos_by_sql(
        &format!(r#"SELECT o_tag AS tag, o_from_key AS from_key, o_from_version AS from_version, o_to_key AS to_key, o_to_version AS to_version, o_reverse AS reverse FROM public.GRAPH_SEARCH($1, $2, $3{}) ORDER BY O_DEPTH, O_TAG"#, if depth.is_some() { ", $4" } else { "" }),
        sql_vals.clone(),
    )
    .await?;

    let result = package_rels(&from_key, &from_version, &result);
    Ok(result)
}

fn package_rels(from_key: &str, from_version: &str, records: &[GraphRelRecord]) -> GraphRelDetailResp {
    let form_rels = records
        .iter()
        .filter(|r| r.from_key == from_key && r.from_version == from_version && !r.reverse)
        .group_by(|r| r.tag.clone())
        .into_iter()
        .map(|(tag, rr)| {
            let rels = rr.map(|rrr| package_rels(&rrr.to_key, &rrr.to_version, records)).collect();
            (tag, rels)
        })
        .collect();
    let to_rels = records
        .iter()
        .filter(|r| r.from_key == from_key && r.from_version == from_version && r.reverse)
        .group_by(|r| r.tag.clone())
        .into_iter()
        .map(|(tag, rr)| {
            let rels = rr.map(|rrr| package_rels(&rrr.to_key, &rrr.to_version, records)).collect();
            (tag, rels)
        })
        .collect();
    GraphRelDetailResp {
        key: from_key.to_string(),
        version: from_version.to_string(),
        form_rels,
        to_rels,
    }
}

pub async fn delete_rels(
    tag: String,
    from_key: Option<String>,
    to_key: Option<String>,
    from_version: Option<String>,
    to_version: Option<String>,
    funs: &TardisFunsInst,
    ctx: &TardisContext,
) -> TardisResult<()> {
    let mut where_fragments: Vec<String> = Vec::new();
    let mut sql_vals: Vec<Value> = vec![];

    sql_vals.push(Value::from(tag));
    where_fragments.push(format!("tag = ${}", sql_vals.len()));

    if let Some(from_key) = from_key {
        sql_vals.push(Value::from(from_key));
        where_fragments.push(format!("key1 = ${}", sql_vals.len()));
    }
    if let Some(to_key) = to_key {
        sql_vals.push(Value::from(to_key));
        where_fragments.push(format!("key2 = ${}", sql_vals.len()));
    }
    if let Some(from_version) = from_version {
        sql_vals.push(Value::from(from_version));
        where_fragments.push(format!("version1 = ${}", sql_vals.len()));
    }
    if let Some(to_version) = to_version {
        sql_vals.push(Value::from(to_version));
        where_fragments.push(format!("version2 = ${}", sql_vals.len()));
    }

    let bs_inst = funs.bs(ctx).await?.inst::<TardisRelDBClient>();
    let (mut conn, table_name) = graph_pg_initializer::init_table_and_conn(bs_inst, ctx, false).await?;
    conn.begin().await?;
    conn.execute_one(
        &format!(
            "DELETE FROM {table_name} WHERE {} AND reverse = false",
            where_fragments.join(" AND ").replace("key1", "from_key").replace("key2", "to_key").replace("version1", "from_version").replace("version2", "to_version")
        ),
        sql_vals.clone(),
    )
    .await?;
    conn.execute_one(
        &format!(
            "DELETE FROM {table_name} WHERE {} AND reverse = true",
            where_fragments.join(" AND ").replace("key1", "to_key").replace("key2", "from_key").replace("version1", "to_version").replace("version2", "from_version")
        ),
        sql_vals,
    )
    .await?;
    conn.commit().await?;
    Ok(())
}

#[derive(sea_orm::FromQueryResult)]
struct GraphRelRecord {
    pub tag: String,
    pub from_key: String,
    pub from_version: String,
    pub to_key: String,
    pub to_version: String,
    pub reverse: bool,
}
