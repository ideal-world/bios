use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::db::reldb_client::TardisRelDBlConnection;
use tardis::db::sea_orm::*;
use tardis::web::web_resp::TardisPage;
use tardis::TardisFuns;

use bios_basic::rbum::constants::RBUM_KIND_ID_IAM_TENANT;
use bios_basic::rbum::dto::filer_dto::RbumBasicFilterReq;
use bios_basic::rbum::serv::rbum_item_serv;

use crate::console_system::dto::iam_cs_tenant_dto::{IamCsTenantAddReq, IamCsTenantDetailResp, IamCsTenantModifyReq, IamCsTenantSummaryResp};
use crate::domain::iam_tenant;

pub async fn add_iam_tenant<'a>(iam_tenant_add_req: &IamCsTenantAddReq, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<String> {
    let id = rbum_item_serv::add_rbum_item(&TardisFuns::field.uuid_str(), RBUM_KIND_ID_IAM_TENANT, &iam_tenant_add_req.basic, None, db, cxt).await?;
    let iam_tenant_id = db.insert_one(iam_tenant::ActiveModel { id: Set(id) }, cxt).await?.last_insert_id;
    Ok(iam_tenant_id)
}

pub async fn modify_iam_tenant<'a>(id: &str, iam_tenant_modify_req: &IamCsTenantModifyReq, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<()> {
    rbum_item_serv::modify_rbum_item(id, &iam_tenant_modify_req.basic, db, cxt).await?;
    Ok(())
}

pub async fn delete_iam_tenant<'a>(id: &str, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<u64> {
    rbum_item_serv::delete_rbum_item(id, db, cxt).await?;
    db.soft_delete(iam_tenant::Entity::find().filter(iam_tenant::Column::Id.eq(id)), &cxt.account_id).await
}

pub async fn peek_iam_tenant<'a>(id: &str, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<IamCsTenantSummaryResp> {
    let basic = rbum_item_serv::peek_rbum_item(
        id,
        &RbumBasicFilterReq {
            rel_cxt_app: false,
            rel_cxt_tenant: false,
            rel_cxt_creator: false,
            rel_cxt_updater: false,
            scope_kind: None,
            kind_id: Some(RBUM_KIND_ID_IAM_TENANT.to_string()),
            domain_id: None,
            disabled: false,
        },
        db,
        cxt,
    )
    .await?;
    Ok(IamCsTenantSummaryResp { basic })
}

pub async fn get_iam_tenant<'a>(id: &str, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<IamCsTenantDetailResp> {
    let basic = rbum_item_serv::get_rbum_item(
        id,
        &RbumBasicFilterReq {
            rel_cxt_app: false,
            rel_cxt_tenant: false,
            rel_cxt_creator: false,
            rel_cxt_updater: false,
            scope_kind: None,
            kind_id: Some(RBUM_KIND_ID_IAM_TENANT.to_string()),
            domain_id: None,
            disabled: false,
        },
        db,
        cxt,
    )
    .await?;
    Ok(IamCsTenantDetailResp { basic })
}

pub async fn find_iam_tenants<'a>(
    page_number: u64,
    page_size: u64,
    desc_sort_by_update: Option<bool>,
    db: &TardisRelDBlConnection<'a>,
    cxt: &TardisContext,
) -> TardisResult<TardisPage<IamCsTenantDetailResp>> {
    let basic = rbum_item_serv::find_rbum_items(
        &RbumBasicFilterReq {
            rel_cxt_app: false,
            rel_cxt_tenant: false,
            rel_cxt_creator: false,
            rel_cxt_updater: false,
            scope_kind: None,
            kind_id: Some(RBUM_KIND_ID_IAM_TENANT.to_string()),
            domain_id: None,
            disabled: false,
        },
        page_number,
        page_size,
        desc_sort_by_update,
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
