use bios_basic::{
    rbum::{
        dto::{
            rbum_cert_dto::{RbumCertAddReq, RbumCertModifyReq},
            rbum_filer_dto::{RbumBasicFilterReq, RbumCertFilterReq},
        },
        rbum_enumeration::{RbumCertRelKind, RbumCertStatusKind},
        serv::{rbum_cert_serv::RbumCertServ, rbum_crud_serv::RbumCrudOperation as _},
    },
    spi::{spi_constants::SPI_PG_KIND_CODE, spi_funs::SpiBsInst, spi_initializer::common_pg},
};
use tardis::{
    basic::{dto::TardisContext, field::TrimString, result::TardisResult}, db::{reldb_client::TardisRelDBClient, sea_orm::Value}, web::web_resp::TardisPage, TardisFuns, TardisFunsInst
};

use crate::{
    dto::stats_conf_dto::{StatsSyncDbConfigAddReq, StatsSyncDbConfigInfoResp, StatsSyncDbConfigModifyReq},
    stats_enumeration::StatsSyncDbConfigSupplierKind,
};

pub(crate) async fn db_config_add(add_req: StatsSyncDbConfigAddReq, funs: &TardisFunsInst, ctx: &TardisContext, _inst: &SpiBsInst) -> TardisResult<()> {
    // 使用rel_rbum_id kind supplier 来作为unique key
    let mut rbum_cert_add_req = RbumCertAddReq {
        ak: TrimString(add_req.db_user),
        sk: Some(TrimString(add_req.db_password)),
        conn_uri: Some(add_req.db_url),
        rel_rbum_id: add_req.fact_conf_key,
        kind: Some(SPI_PG_KIND_CODE.to_string()),
        supplier: Some(StatsSyncDbConfigSupplierKind::Fact.to_string()),
        ext: None,
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
    if let Some(fact_conf_col_key) = add_req.fact_conf_col_key {
        rbum_cert_add_req.rel_rbum_id = format!("{}_{}", rbum_cert_add_req.rel_rbum_id, fact_conf_col_key);
        rbum_cert_add_req.supplier = Some(StatsSyncDbConfigSupplierKind::FactCol.to_string());
    }
    let rbum_cert_list = RbumCertServ::find_rbums(
        &RbumCertFilterReq {
            basic: RbumBasicFilterReq {
                ids: Some(vec![rbum_cert_add_req.rel_rbum_id.clone()]),
                ..Default::default()
            },
            kind: Some(SPI_PG_KIND_CODE.to_string()),
            suppliers: Some(vec![rbum_cert_add_req.supplier.clone().expect("supplier is required")]),
            ..Default::default()
        },
        None,
        None,
        funs,
        ctx,
    )
    .await?;
    if !rbum_cert_list.is_empty() {
        return Err(funs.err().conflict(&RbumCertServ::get_obj_name(), "add", "rbum cert already exists", "409-rbum-cert-already-exists"));
    }
    RbumCertServ::add_rbum(&mut rbum_cert_add_req, funs, ctx).await?;
    return Ok(());
}

pub(crate) async fn db_config_modify(modify_req: StatsSyncDbConfigModifyReq, funs: &TardisFunsInst, ctx: &TardisContext, _inst: &SpiBsInst) -> TardisResult<()> {
    let mut rel_rbum_id = modify_req.fact_conf_key.clone();
    let mut supplier = StatsSyncDbConfigSupplierKind::Fact.to_string();
    if let Some(fact_conf_col_key) = modify_req.fact_conf_col_key {
        rel_rbum_id = format!("{}_{}", modify_req.fact_conf_key, fact_conf_col_key);
        supplier = StatsSyncDbConfigSupplierKind::FactCol.to_string();
    }
    if let Some(rbum_cert) = RbumCertServ::find_one_rbum(
        &RbumCertFilterReq {
            basic: RbumBasicFilterReq {
                ids: Some(vec![rel_rbum_id.clone()]),
                ..Default::default()
            },
            kind: Some(SPI_PG_KIND_CODE.to_string()),
            suppliers: Some(vec![supplier]),
            ..Default::default()
        },
        funs,
        ctx,
    )
    .await?
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
        RbumCertServ::modify_rbum(&rbum_cert.id, &mut rbum_cert_modify_req, funs, ctx).await?;
    } else {
        return Err(funs.err().not_found(&RbumCertServ::get_obj_name(), "modify", "rbum cert not found", "404-rbum-cert-not-found"));
    }
    return Ok(());
}

pub(crate) async fn db_config_paginate(page_number: u32, page_size: u32, funs: &TardisFunsInst, ctx: &TardisContext, inst: &SpiBsInst) -> TardisResult<TardisPage<StatsSyncDbConfigInfoResp>> {

  let rbum_cert_list = RbumCertServ::paginate_rbums(
    &RbumCertFilterReq {
        kind: Some(SPI_PG_KIND_CODE.to_string()),
        ..Default::default()
    },
    page_number,
    page_size,
    None,
    None,
    funs,
    ctx,
)
.await?;

    todo!()
}

async fn find_db_config(fact_conf_key: String, fact_conf_col_key: Option<String>, funs: &TardisFunsInst, ctx: &TardisContext, _inst: &SpiBsInst) -> TardisResult<()> {
    let mut rel_rbum_id = fact_conf_key.clone();
    let mut supplier = StatsSyncDbConfigSupplierKind::Fact.to_string();
    if let Some(fact_conf_col_key) = fact_conf_col_key {
        rel_rbum_id = format!("{}_{}", fact_conf_key, fact_conf_col_key);
        supplier = StatsSyncDbConfigSupplierKind::FactCol.to_string();
    }
    if let Some(rbum_cert) = RbumCertServ::find_one_rbum(
        &RbumCertFilterReq {
            basic: RbumBasicFilterReq {
                ids: Some(vec![rel_rbum_id.to_string()]),
                ..Default::default()
            },
            kind: Some(SPI_PG_KIND_CODE.to_string()),
            suppliers: Some(vec![supplier.to_string()]),
            ..Default::default()
        },
        funs,
        ctx,
    )
    .await?
    {
        return Ok(rbum_cert);
    } else {
    }
}

pub(crate) async fn fact_record_sync(fact_key: &str, funs: &TardisFunsInst, ctx: &TardisContext, inst: &SpiBsInst) -> TardisResult<()> {
    let bs_inst = inst.inst::<TardisRelDBClient>();
    let (conn, _) = common_pg::init_conn(bs_inst).await?;

    conn.begin().await?;

    todo!();
    let fact_col_list = conn
        .query_all(
            &format!("SELECT key FROM starsys_stats_conf_fact_col WHERE rel_conf_fact_key = $1"),
            vec![Value::from(fact_key)],
        )
        .await?;
    for col in fact_col_list.iter() {
        let col_key = col.try_get::<String>("", "key")?;
        fact_col_record_sync(fact_key, &col_key, funs, ctx, inst).await?;
    }
    conn.commit().await?;
    Ok(())
}

pub(crate) async fn fact_col_record_sync(fact_key: &str, col_key: &str, funs: &TardisFunsInst, ctx: &TardisContext, inst: &SpiBsInst) -> TardisResult<()> {
    let bs_inst = inst.inst::<TardisRelDBClient>();
    let (conn, _) = common_pg::init_conn(bs_inst).await?;

    todo!()
}

async fn do_fact_col_record_sync(fact_key: &str, col_key: &str, funs: &TardisFunsInst, ctx: &TardisContext, inst: &SpiBsInst) -> TardisResult<()> {
    let bs_inst = inst.inst::<TardisRelDBClient>();
    let (conn, _) = common_pg::init_conn(bs_inst).await?;

    todo!()
}
