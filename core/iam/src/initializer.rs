use tardis::basic::error::TardisError;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::db::reldb_client::TardisActiveModel;
use tardis::TardisFuns;
use tardis::web::web_server::TardisWebServer;

use bios_basic::rbum::dto::rbum_kind_dto::RbumKindAddReq;
use bios_basic::rbum::enumeration::RbumScopeKind;
use bios_basic::rbum::initializer::get_sys_admin_context;
use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;
use bios_basic::rbum::serv::rbum_domain_serv::RbumDomainServ;
use bios_basic::rbum::serv::rbum_kind_serv::RbumKindServ;

use crate::console_system::api::iam_cs_tenant_api;
use crate::constants::*;
use crate::domain::iam_tenant;

pub async fn init_db() -> TardisResult<()> {
    bios_basic::rbum::initializer::init_db().await?;
    let mut tx = TardisFuns::reldb().conn();
    tx.begin().await?;
    tx.create_table(&iam_tenant::ActiveModel::create_table_statement(TardisFuns::reldb().backend())).await?;

    let cxt = get_sys_admin_context().await?;
    let rbum_tenant_kind_id = RbumKindServ::get_rbum_kind_id_by_uri_scheme(RBUM_KIND_SCHEME_IAM_TENANT, &tx, &cxt)
        .await?
        .ok_or_else(|| TardisError::InternalError(format!("rbum kind {} not found", RBUM_KIND_SCHEME_IAM_TENANT)))?;
    let rbum_app_kind_id = RbumKindServ::get_rbum_kind_id_by_uri_scheme(RBUM_KIND_SCHEME_IAM_APP, &tx, &cxt)
        .await?
        .ok_or_else(|| TardisError::InternalError(format!("rbum kind {} not found", RBUM_KIND_SCHEME_IAM_APP)))?;
    let rbum_account_kind_id = RbumKindServ::get_rbum_kind_id_by_uri_scheme(RBUM_KIND_SCHEME_IAM_ACCOUNT, &tx, &cxt)
        .await?
        .ok_or_else(|| TardisError::InternalError(format!("rbum kind {} not found", RBUM_KIND_SCHEME_IAM_ACCOUNT)))?;
    let rbum_role_kind_id = if let Some(rbum_role_kind_id) = RbumKindServ::get_rbum_kind_id_by_uri_scheme(RBUM_KIND_SCHEME_IAM_ROLE, &tx, &cxt).await? {
        rbum_role_kind_id
    } else {
        RbumKindServ::add_rbum(
            &mut RbumKindAddReq {
                uri_scheme: TrimString(RBUM_KIND_SCHEME_IAM_ROLE.to_string()),
                name: TrimString(RBUM_KIND_SCHEME_IAM_ROLE.to_string()),
                note: None,
                icon: None,
                sort: None,
                ext_table_name: Some(RBUM_KIND_SCHEME_IAM_ROLE.to_string().to_lowercase()),
                scope_kind: Some(RbumScopeKind::Global),
            },
            &tx,
            &cxt,
        )
        .await?
    };
    let rbum_group_kind_id = if let Some(rbum_group_kind_id) = RbumKindServ::get_rbum_kind_id_by_uri_scheme(RBUM_KIND_SCHEME_IAM_GROUP, &tx, &cxt).await? {
        rbum_group_kind_id
    } else {
        RbumKindServ::add_rbum(
            &mut RbumKindAddReq {
                uri_scheme: TrimString(RBUM_KIND_SCHEME_IAM_GROUP.to_string()),
                name: TrimString(RBUM_KIND_SCHEME_IAM_GROUP.to_string()),
                note: None,
                icon: None,
                sort: None,
                ext_table_name: Some(RBUM_KIND_SCHEME_IAM_GROUP.to_string().to_lowercase()),
                scope_kind: Some(RbumScopeKind::Global),
            },
            &tx,
            &cxt,
        )
        .await?
    };
    let rbum_res_http_kind_id = if let Some(rbum_res_http_kind_id) = RbumKindServ::get_rbum_kind_id_by_uri_scheme(RBUM_KIND_SCHEME_IAM_RES_HTTP, &tx, &cxt).await? {
        rbum_res_http_kind_id
    } else {
        RbumKindServ::add_rbum(
            &mut RbumKindAddReq {
                uri_scheme: TrimString(RBUM_KIND_SCHEME_IAM_RES_HTTP.to_string()),
                name: TrimString(RBUM_KIND_SCHEME_IAM_RES_HTTP.to_string()),
                note: None,
                icon: None,
                sort: None,
                ext_table_name: Some(RBUM_KIND_SCHEME_IAM_RES_HTTP.to_string().to_lowercase()),
                scope_kind: Some(RbumScopeKind::Global),
            },
            &tx,
            &cxt,
        )
        .await?
    };

    let rbum_iam_domain_id = RbumDomainServ::get_rbum_domain_id_by_uri_authority(bios_basic::Components::Iam.to_string().as_str(), &tx, &cxt)
        .await?
        .ok_or_else(|| TardisError::InternalError(format!("rbum domain {} not found", bios_basic::Components::Iam.to_string())))?;

    set_basic_info(BasicInfoPub {
        rbum_tenant_kind_id,
        rbum_app_kind_id,
        rbum_account_kind_id,
        rbum_role_kind_id,
        rbum_group_kind_id,
        rbum_res_http_kind_id,
        rbum_iam_domain_id,
    })?;
    tx.commit().await?;
    Ok(())
}

pub async fn init_api(web_server: &mut TardisWebServer) -> TardisResult<()> {
    web_server.add_module("iam", (iam_cs_tenant_api::IamCsTenantApi));
    Ok(())
}
