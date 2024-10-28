use bios_basic::rbum::{dto::{rbum_cert_dto::{RbumCertAddReq, RbumCertModifyReq}, rbum_filer_dto::{RbumBasicFilterReq, RbumCertFilterReq}}, rbum_enumeration::{RbumCertRelKind, RbumCertStatusKind}, serv::{rbum_cert_serv::RbumCertServ, rbum_crud_serv::RbumCrudOperation as _}};
use tardis::{basic::{dto::TardisContext, field::TrimString, result::TardisResult}, TardisFuns, TardisFunsInst};

use crate::{dto::stats_conf_dto::{StatsSyncDbConfigAddReq, StatsSyncDbConfigModifyReq}, stats_constants::DOMAIN_CODE, stats_enumeration::StatsSyncDbConfigSupplierKind};

pub(crate) async fn db_config_add(add_req: StatsSyncDbConfigAddReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    // 使用rel_rbum_id kind supplier 来作为unique key
    let mut rbum_cert_add_req = RbumCertAddReq {
        ak: TrimString(add_req.db_user),
        sk: Some(TrimString(add_req.db_password)),
        conn_uri: Some(add_req.db_url),
        rel_rbum_id: add_req.fact_conf_key,
        kind: Some(DOMAIN_CODE.to_string()),
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
    let rbum_cert_list = RbumCertServ::find_rbums(&RbumCertFilterReq {
        basic: RbumBasicFilterReq {
            ids: Some(vec![rbum_cert_add_req.rel_rbum_id.clone()]),
            ..Default::default()
        },
        kind: Some(DOMAIN_CODE.to_string()),
        suppliers: Some(vec![rbum_cert_add_req.supplier.clone().expect("supplier is required")]),
        ..Default::default()
    }, None, None, funs, ctx).await?;
    if !rbum_cert_list.is_empty() {
        return Err(funs.err().conflict(&RbumCertServ::get_obj_name(), "add", "rbum cert already exists", "409-rbum-cert-already-exists"));
    }
    RbumCertServ::add_rbum(&mut rbum_cert_add_req, funs, ctx).await?;
    return Ok(());
}
pub(crate) async fn db_config_modify(modify_req: StatsSyncDbConfigModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
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
    todo!()
}
