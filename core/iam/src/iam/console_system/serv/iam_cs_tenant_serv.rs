use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::db::reldb_client::TardisRelDBClientTransaction;
use tardis::db::sea_orm::*;
use tardis::web::web_resp::TardisPage;
use tardis::TardisFuns;

use crate::iam::console_system::dto::iam_cs_tenant_dto::{IamCsTenantAddReq, IamCsTenantDetailResp, IamCsTenantModifyReq, IamCsTenantSummaryResp};
use crate::iam::domain::iam_tenant;
use crate::rbum::dto::filer_dto::RbumBasicFilterReq;
use crate::rbum::serv::rbum_item_serv;

pub async fn add_iam_tenant(iam_tenant_add_req: &IamCsTenantAddReq, db: &TardisRelDBClientTransaction, cxt: &TardisContext) -> TardisResult<String> {
    let id = rbum_item_serv::add_rbum_item(&TardisFuns::field.uuid_str(), iam_tenant::RBUM_KIND_ID, &iam_tenant_add_req.basic, None, db, cxt).await?;
    let iam_tenant_id = db
        .insert_one(
            iam_tenant::ActiveModel {
                id: Set(id),
                ..Default::default()
            },
            cxt,
        )
        .await?
        .last_insert_id;
    Ok(iam_tenant_id)
}

pub async fn modify_iam_tenant(id: &str, iam_tenant_modify_req: &IamCsTenantModifyReq, db: &TardisRelDBClientTransaction, cxt: &TardisContext) -> TardisResult<()> {
    rbum_item_serv::modify_rbum_item(id, &iam_tenant_modify_req.basic, db, cxt).await?;
    Ok(())
}

pub async fn delete_iam_tenant(id: &str, db: &TardisRelDBClientTransaction, cxt: &TardisContext) -> TardisResult<u64> {
    rbum_item_serv::delete_rbum_item(id, db, cxt).await?;
    db.soft_delete(iam_tenant::Entity::find().filter(iam_tenant::Column::Id.eq(id)), &cxt.account_id).await
}

pub async fn peek_iam_tenant(id: &str, db: &TardisRelDBClientTransaction, cxt: &TardisContext) -> TardisResult<IamCsTenantSummaryResp> {
    let basic = rbum_item_serv::peek_rbum_item(
        id,
        &RbumBasicFilterReq {
            rel_cxt_app: false,
            rel_cxt_tenant: false,
            rel_cxt_creator: false,
            rel_cxt_updater: false,
            scope_kind: None,
            kind_id: Some(iam_tenant::RBUM_KIND_ID.to_string()),
            domain_id: None,
            disabled: false,
        },
        db,
        cxt,
    )
    .await?;
    Ok(IamCsTenantSummaryResp { basic })
}

pub async fn get_iam_tenant(id: &str, db: &TardisRelDBClientTransaction, cxt: &TardisContext) -> TardisResult<IamCsTenantDetailResp> {
    let basic = rbum_item_serv::get_rbum_item(
        id,
        &RbumBasicFilterReq {
            rel_cxt_app: false,
            rel_cxt_tenant: false,
            rel_cxt_creator: false,
            rel_cxt_updater: false,
            scope_kind: None,
            kind_id: Some(iam_tenant::RBUM_KIND_ID.to_string()),
            domain_id: None,
            disabled: false,
        },
        db,
        cxt,
    )
    .await?;
    Ok(IamCsTenantDetailResp { basic })
}

pub async fn find_iam_tenants(page_number: u64, page_size: u64, db: &TardisRelDBClientTransaction, cxt: &TardisContext) -> TardisResult<TardisPage<IamCsTenantDetailResp>> {
    let basic = rbum_item_serv::find_rbum_items(
        &RbumBasicFilterReq {
            rel_cxt_app: false,
            rel_cxt_tenant: false,
            rel_cxt_creator: false,
            rel_cxt_updater: false,
            scope_kind: None,
            kind_id: Some(iam_tenant::RBUM_KIND_ID.to_string()),
            domain_id: None,
            disabled: false,
        },
        page_number,
        page_size,
        db,
        cxt,
    )
    .await?;
    Ok(TardisPage {
        page_number: basic.page_number,
        page_size: basic.page_size,
        total_size: basic.total_size,
        records: basic.records.into_iter().map(|r| IamCsTenantDetailResp { basic: r }).collect(),
    })
}
