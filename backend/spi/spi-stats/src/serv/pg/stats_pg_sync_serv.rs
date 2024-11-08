use bios_basic::{
    rbum::{
        dto::{
            rbum_cert_dto::{RbumCertAddReq, RbumCertModifyReq},
            rbum_filer_dto::{RbumBasicFilterReq, RbumCertFilterReq},
        },
        rbum_enumeration::{RbumCertRelKind, RbumCertStatusKind},
        serv::{rbum_cert_serv::RbumCertServ, rbum_crud_serv::RbumCrudOperation},
    },
    spi::{spi_constants::SPI_PG_KIND_CODE, spi_funs::SpiBsInst, spi_initializer::common_pg},
};
use tardis::{
    basic::{dto::TardisContext, field::TrimString, result::TardisResult},
    db::{reldb_client::TardisRelDBClient, sea_orm::Value},
    TardisFunsInst,
};

use crate::{
    dto::stats_conf_dto::{StatsSyncDbConfigAddReq, StatsSyncDbConfigExt, StatsSyncDbConfigInfoResp, StatsSyncDbConfigInfoWithSkResp, StatsSyncDbConfigModifyReq},
    stats_constants::DOMAIN_CODE,
};

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
    return Ok(rbum_cert);
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
    return Ok(());
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

async fn find_db_config(cert_id: &str, funs: &TardisFunsInst, ctx: &TardisContext, _inst: &SpiBsInst) -> TardisResult<StatsSyncDbConfigInfoWithSkResp> {
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
        return Ok(StatsSyncDbConfigInfoWithSkResp {
            id: cert_id.to_string(),
            db_url: rbum_cert.conn_uri.clone(),
            db_user: rbum_cert.ak.clone(),
            db_password: db_password,
            max_connections,
            min_connections,
        });
    } else {
        return Err(funs.err().not_found(&RbumCertServ::get_obj_name(), "find", "rbum cert not found", "404-rbum-cert-not-found"));
    }
}

pub(crate) async fn fact_record_sync(fact_key: &str, funs: &TardisFunsInst, ctx: &TardisContext, inst: &SpiBsInst) -> TardisResult<()> {
    // let bs_inst = inst.inst::<TardisRelDBClient>();
    // let (conn, _) = common_pg::init_conn(bs_inst).await?;

    // conn.begin().await?;

    // todo!();
    // let fact_col_list = conn
    //     .query_all(
    //         &format!("SELECT key FROM starsys_stats_conf_fact_col WHERE rel_conf_fact_key = $1"),
    //         vec![Value::from(fact_key)],
    //     )
    //     .await?;
    // for col in fact_col_list.iter() {
    //     let col_key = col.try_get::<String>("", "key")?;
    //     fact_col_record_sync(fact_key, &col_key, funs, ctx, inst).await?;
    // }
    // conn.commit().await?;
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
