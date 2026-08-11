use bios_basic::spi::{
    spi_funs::SpiBsInst,
    spi_initializer::common_pg::{self, package_table_name},
};
use serde::Serialize;
use tardis::{
    basic::{dto::TardisContext, error::TardisError, result::TardisResult},
    chrono::{DateTime, Utc},
    db::{
        reldb_client::{TardisRelDBClient, TardisRelDBlConnection},
        sea_orm::Value,
    },
    serde_json,
    TardisFunsInst,
};

use crate::{
    dto::{
        stats_conf_dto::{StatsConfFactColInfoResp, StatsConfFactInfoResp},
        stats_schema_dto::{StatsMdlExportModel, StatsMdlExportResp},
    },
    stats_enumeration::{StatsDataTypeKind, StatsFactColKind},
};

use super::{stats_pg_conf_fact_col_serv, stats_pg_conf_fact_serv};

/// 物理事实表名前缀：`{prefix}_stats_inst_fact_{fact_key}`（prefix 即 GLOBAL_STORAGE_FLAG，如 starsys）
fn mdl_model_prefix(ctx: &TardisContext) -> String {
    package_table_name("stats_inst_fact", ctx)
        .rsplit_once('.')
        .map(|(_, table)| table.to_string())
        .unwrap_or_else(|| "stats_inst_fact".to_string())
}

/// 物理事实表名：`starsys_stats_inst_fact_{fact_key}`（模型 name / 文件名 / table_reference.table 均用它）
fn mdl_fact_table_name(fact_conf_key: &str, ctx: &TardisContext) -> String {
    format!("{}_{fact_conf_key}", mdl_model_prefix(ctx))
}

/// 归一化事实 key：兼容传入完整物理表名（starsys_stats_inst_fact_xxx）或短 key（xxx）
fn normalize_fact_conf_key(requested: &str, ctx: &TardisContext) -> String {
    let prefix = format!("{}_", mdl_model_prefix(ctx));
    requested
        .strip_prefix(&prefix)
        .map(|s| s.to_string())
        .unwrap_or_else(|| requested.to_string())
}

/// Export MDL（Metric Definition Language）for all/single online facts.
///
/// 导出 MDL（指标定义语言）语义层模型（WrenAI 逐模型格式，与 hai-wren `models/*.yml` 对齐）：
/// - `fact_conf_key = None`：导出全部 online 事实
/// - `fact_conf_key = Some(k)`：导出单个事实（不要求 online；k 可为短 key 或完整物理表名）
/// - `since = Some(ts)`：增量导出，仅返回该时间之后有变更的事实（事实本身或其列/明细配置）
pub(crate) async fn mdl_export(
    fact_conf_key: Option<String>,
    since: Option<String>,
    funs: &TardisFunsInst,
    ctx: &TardisContext,
    inst: &SpiBsInst,
) -> TardisResult<StatsMdlExportResp> {
    let fact_conf_key = fact_conf_key.map(|key| normalize_fact_conf_key(&key, ctx));
    let facts = stats_pg_conf_fact_serv::find(
        fact_conf_key.as_ref().map(|key| vec![key.clone()]),
        None,
        None,
        if fact_conf_key.is_some() { None } else { Some(true) },
        None,
        None,
        funs,
        ctx,
        inst,
    )
    .await?;

    let since_ts = since
        .as_deref()
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&Utc));

    let bs_inst = inst.inst::<TardisRelDBClient>();
    let (conn, schema_name) = common_pg::init_conn(bs_inst).await?;

    let mut models = vec![];
    for fact in facts {
        // 增量导出：事实本身或其相关配置（列/明细）在 since 之后有变更才导出
        if let Some(since_ts) = since_ts {
            let related_changed = fact_related_updated_since(&fact.key, since_ts, &conn, ctx).await?;
            if fact.update_time < since_ts && !related_changed {
                continue;
            }
        }
        match build_fact_mdl(&fact, &schema_name, funs, ctx, inst).await {
            Ok(content) => models.push(StatsMdlExportModel {
                // fact_key = 物理表名：sync-mdl.sh 以其为文件名，wren-mcp 按物理表名读取 models/{table}.yml
                fact_key: mdl_fact_table_name(&fact.key, ctx),
                content,
            }),
            Err(err) => {
                tardis::log::warn!("[spi-stats] mdl-export skip fact {}: {err}", fact.key);
            }
        }
    }
    Ok(StatsMdlExportResp { models })
}

/// Describe the statistics semantic layer as AI readable JSON（设计文档 §4.3 P1）
///
/// 以 AI 可读的 JSON 描述统计语义层（仅 online 事实）
pub(crate) async fn describe(funs: &TardisFunsInst, ctx: &TardisContext, inst: &SpiBsInst) -> TardisResult<serde_json::Value> {
    let facts = stats_pg_conf_fact_serv::find(None, None, None, Some(true), None, None, funs, ctx, inst).await?;
    let mut fact_items = vec![];
    for fact in &facts {
        let cols = stats_pg_conf_fact_col_serv::find_by_fact_conf_key(&fact.key, funs, ctx, inst).await?;
        let mut columns = vec![];
        for col in cols {
            columns.push(serde_json::json!({
                "key": col.key,
                "show_name": col.show_name,
                "kind": col.kind.to_string(),
                "data_type": col.dim_data_type.as_ref().or(col.mes_data_type.as_ref()).map(|t| t.to_string()),
                "unit": col.mes_unit,
                "dim_rel_conf_dim_key": col.dim_rel_conf_dim_key,
                "rel_field": col.rel_field,
            }));
        }
        fact_items.push(serde_json::json!({
            "fact_key": fact.key,
            "show_name": fact.show_name,
            "query_limit": fact.query_limit,
            "is_online": fact.is_online,
            "table": mdl_fact_table_name(&fact.key, ctx),
            "columns": columns,
        }));
    }
    Ok(serde_json::json!({ "facts": fact_items }))
}

/// Check whether the fact or its related configurations（fact_col / fact_detail）were updated after `since_ts`.
///
/// 判断事实或其相关配置（fact_col / fact_detail）是否在 since_ts 之后有变更
async fn fact_related_updated_since(fact_conf_key: &str, since_ts: DateTime<Utc>, conn: &TardisRelDBlConnection, ctx: &TardisContext) -> TardisResult<bool> {
    if common_pg::check_table_exit("stats_conf_fact_col", conn, ctx).await? {
        let table = package_table_name("stats_conf_fact_col", ctx);
        let count = conn
            .count_by_sql(
                &format!("SELECT 1 FROM {table} WHERE rel_conf_fact_key = $1 AND update_time >= $2"),
                vec![Value::from(fact_conf_key), Value::from(since_ts)],
            )
            .await?;
        if count > 0 {
            return Ok(true);
        }
    }
    if common_pg::check_table_exit("stats_conf_fact_detail", conn, ctx).await? {
        let table = package_table_name("stats_conf_fact_detail", ctx);
        let count = conn
            .count_by_sql(
                &format!("SELECT 1 FROM {table} WHERE rel_conf_fact_key = $1 AND update_time >= $2"),
                vec![Value::from(fact_conf_key), Value::from(since_ts)],
            )
            .await?;
        if count > 0 {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Build a single fact's MDL YAML content（WrenAI 逐模型格式，与 hai-wren `models/*.yml` 对齐）。
///
/// 生成单个事实的 MDL YAML 内容
async fn build_fact_mdl(
    fact: &StatsConfFactInfoResp,
    schema_name: &str,
    funs: &TardisFunsInst,
    ctx: &TardisContext,
    inst: &SpiBsInst,
) -> TardisResult<String> {
    let fact_cols = stats_pg_conf_fact_col_serv::find_by_fact_conf_key(&fact.key, funs, ctx, inst).await?;

    let table_name = mdl_fact_table_name(&fact.key, ctx);

    let mut columns = vec![
        // 固定列（实例表固定结构）
        MdlColumn {
            name: "key".to_string(),
            r#type: "VARCHAR".to_string(),
            description: "主键（记录唯一标识）".to_string(),
            is_primary_key: Some(true),
            not_null: Some(true),
        },
        MdlColumn {
            name: "own_paths".to_string(),
            r#type: "VARCHAR".to_string(),
            description: "归属路径（数据隔离域）".to_string(),
            is_primary_key: None,
            not_null: Some(true),
        },
        MdlColumn {
            name: "idempotent_id".to_string(),
            r#type: "VARCHAR".to_string(),
            description: "幂等 ID（默认空串）".to_string(),
            is_primary_key: None,
            not_null: Some(true),
        },
        MdlColumn {
            name: "ct".to_string(),
            r#type: "TIMESTAMPTZ".to_string(),
            description: "记录落库时间（默认当前时间）".to_string(),
            is_primary_key: None,
            not_null: Some(true),
        },
        MdlColumn {
            name: "ext".to_string(),
            r#type: "JSON".to_string(),
            description: "扩展字段（JSONB）".to_string(),
            is_primary_key: None,
            not_null: Some(true),
        },
    ];

    for col in &fact_cols {
        columns.push(MdlColumn {
            name: col.key.clone(),
            r#type: mdl_column_type(col),
            description: col.show_name.clone(),
            is_primary_key: None,
            not_null: Some(true),
        });
    }

    let mdl_model = MdlModel {
        name: table_name.clone(),
        description: fact
            .remark
            .clone()
            .filter(|remark| !remark.is_empty())
            .unwrap_or_else(|| fact.show_name.clone()),
        keywords: vec![fact.show_name.clone()],
        table_reference: MdlTableReference {
            catalog: "".to_string(),
            schema: schema_name.to_string(),
            table: table_name,
        },
        primary_key: "key".to_string(),
        columns,
    };

    serde_yaml::to_string(&mdl_model).map_err(|err| TardisError::internal_error(&format!("serialize mdl yaml failed: {err}"), "500-spi-stats-mdl-export-yaml"))
}

/// 事实列 → MDL 列类型（WrenAI 类型；维度列多值时用 ARRAY<...>）
fn mdl_column_type(col: &StatsConfFactColInfoResp) -> String {
    let data_type = match col.kind {
        StatsFactColKind::Dimension => col.dim_data_type.as_ref().map(mdl_type_of).unwrap_or("VARCHAR"),
        StatsFactColKind::Measure => col.mes_data_type.as_ref().map(mdl_type_of).unwrap_or("DOUBLE"),
        StatsFactColKind::Ext => "VARCHAR",
    };
    if col.kind == StatsFactColKind::Dimension && col.dim_multi_values.unwrap_or(false) {
        format!("ARRAY<{data_type}>")
    } else {
        data_type.to_string()
    }
}

/// spi-stats `StatsDataTypeKind` → WrenAI MDL 列类型（与 hai-wren `models/*.yml` 对齐）
fn mdl_type_of(data_type: &StatsDataTypeKind) -> &'static str {
    match data_type {
        StatsDataTypeKind::String => "VARCHAR",
        StatsDataTypeKind::Int => "INTEGER",
        StatsDataTypeKind::Float => "REAL",
        StatsDataTypeKind::Double => "DOUBLE",
        StatsDataTypeKind::Boolean => "BOOLEAN",
        StatsDataTypeKind::Date => "DATE",
        StatsDataTypeKind::DateTime => "TIMESTAMPTZ",
    }
}

// ═══════════════════════════════════════════════════════════════════
// MDL YAML 结构体（WrenAI 逐模型格式，与 hai-wren `models/*.yml` 对齐）
// ═══════════════════════════════════════════════════════════════════

#[derive(Serialize)]
struct MdlModel {
    name: String,
    description: String,
    keywords: Vec<String>,
    table_reference: MdlTableReference,
    primary_key: String,
    columns: Vec<MdlColumn>,
}

#[derive(Serialize)]
struct MdlTableReference {
    catalog: String,
    schema: String,
    table: String,
}

#[derive(Serialize)]
struct MdlColumn {
    name: String,
    r#type: String,
    description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    is_primary_key: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    not_null: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mdl_type_of() {
        assert_eq!(mdl_type_of(&StatsDataTypeKind::String), "VARCHAR");
        assert_eq!(mdl_type_of(&StatsDataTypeKind::Int), "INTEGER");
        assert_eq!(mdl_type_of(&StatsDataTypeKind::Float), "REAL");
        assert_eq!(mdl_type_of(&StatsDataTypeKind::Double), "DOUBLE");
        assert_eq!(mdl_type_of(&StatsDataTypeKind::Boolean), "BOOLEAN");
        assert_eq!(mdl_type_of(&StatsDataTypeKind::Date), "DATE");
        assert_eq!(mdl_type_of(&StatsDataTypeKind::DateTime), "TIMESTAMPTZ");
    }

    #[test]
    fn test_mdl_yaml_serialization() {
        let mdl_model = MdlModel {
            name: "starsys_stats_inst_fact_demo".to_string(),
            description: "示例事实表".to_string(),
            keywords: vec!["示例".to_string()],
            table_reference: MdlTableReference {
                catalog: "".to_string(),
                schema: "public".to_string(),
                table: "starsys_stats_inst_fact_demo".to_string(),
            },
            primary_key: "key".to_string(),
            columns: vec![
                MdlColumn {
                    name: "key".to_string(),
                    r#type: "VARCHAR".to_string(),
                    description: "主键（记录唯一标识）".to_string(),
                    is_primary_key: Some(true),
                    not_null: Some(true),
                },
                MdlColumn {
                    name: "attr_a".to_string(),
                    r#type: "ARRAY<VARCHAR>".to_string(),
                    description: "属性 A 列表".to_string(),
                    is_primary_key: None,
                    not_null: Some(true),
                },
                MdlColumn {
                    name: "value_num".to_string(),
                    r#type: "DOUBLE".to_string(),
                    description: "数值列".to_string(),
                    is_primary_key: None,
                    not_null: Some(true),
                },
            ],
        };
        let yaml = serde_yaml::to_string(&mdl_model).expect("serialize mdl yaml");
        println!("{yaml}");
        assert!(yaml.contains("name: starsys_stats_inst_fact_demo"));
        assert!(yaml.contains("table_reference:"));
        assert!(yaml.contains("schema: public"));
        assert!(yaml.contains("table: starsys_stats_inst_fact_demo"));
        assert!(yaml.contains("primary_key: key"));
        assert!(yaml.contains("is_primary_key: true"));
        assert!(yaml.contains("not_null: true"));
        assert!(yaml.contains("ARRAY<VARCHAR>"));
        assert!(yaml.contains("keywords:"));
    }

    #[test]
    fn test_mdl_fact_table_name() {
        let ctx = TardisContext {
            own_paths: "".to_string(),
            ak: "".to_string(),
            roles: vec![],
            groups: vec![],
            owner: "".to_string(),
            ..Default::default()
        };
        // 只验证前缀（starsys_）与 key 后缀，schema 前缀依赖上下文隔离标识
        let table = mdl_fact_table_name("demo", &ctx);
        assert!(table.starts_with("starsys_"));
        assert!(table.ends_with("_stats_inst_fact_demo"));
        // 归一化：完整物理表名 → 短 key
        let key = normalize_fact_conf_key(&table, &ctx);
        assert_eq!(key, "demo");
        // 短 key 原样返回
        assert_eq!(normalize_fact_conf_key("demo", &ctx), "demo");
    }
}
