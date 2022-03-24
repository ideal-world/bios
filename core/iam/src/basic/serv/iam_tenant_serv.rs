use async_trait::async_trait;
use tardis::basic::dto::TardisContext;
use tardis::basic::error::TardisError;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::db::reldb_client::TardisRelDBlConnection;
use tardis::db::sea_orm::*;
use tardis::db::sea_query::SelectStatement;
use tardis::TardisFuns;

use bios_basic::rbum::dto::filer_dto::RbumItemFilterReq;
use bios_basic::rbum::dto::rbum_item_dto::{RbumItemAddReq, RbumItemModifyReq};
use bios_basic::rbum::helper::rbum_scope_helper;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;

use crate::basic::constants;
use crate::basic::constants::{RBUM_ITEM_ID_TENANT_LEN, RBUM_SCOPE_LEVEL_TENANT};
use crate::basic::domain::iam_tenant;
use crate::basic::dto::iam_tenant_dto::{IamTenantAddReq, IamTenantDetailResp, IamTenantModifyReq, IamTenantSummaryResp};

pub struct IamTenantServ;

#[async_trait]
impl<'a> RbumItemCrudOperation<'a, iam_tenant::ActiveModel, IamTenantAddReq, IamTenantModifyReq, IamTenantSummaryResp, IamTenantDetailResp> for IamTenantServ {
    fn get_ext_table_name() -> &'static str {
        iam_tenant::Entity.table_name()
    }

    fn get_rbum_kind_id() -> String {
        constants::get_rbum_basic_info().kind_tenant_id.clone()
    }

    fn get_rbum_domain_id() -> String {
        constants::get_rbum_basic_info().domain_iam_id.clone()
    }

    async fn package_item_add(add_req: &IamTenantAddReq, _: &TardisRelDBlConnection<'a>, _: &TardisContext) -> TardisResult<RbumItemAddReq> {
        Ok(RbumItemAddReq {
            id: Some(TrimString(IamTenantServ::get_new_id())),
            uri_path: None,
            name: add_req.name.clone(),
            icon: add_req.icon.clone(),
            sort: add_req.sort,
            scope_level: add_req.scope_level,
            disabled: add_req.disabled,
            rel_rbum_kind_id: "".to_string(),
            rel_rbum_domain_id: "".to_string(),
        })
    }

    async fn package_ext_add(id: &str, add_req: &IamTenantAddReq, _: &TardisRelDBlConnection<'a>, _: &TardisContext) -> TardisResult<iam_tenant::ActiveModel> {
        Ok(iam_tenant::ActiveModel {
            id: Set(id.to_string()),
            contact_phone: Set(add_req.contact_phone.as_ref().unwrap_or(&"".to_string()).to_string()),
        })
    }

    async fn package_item_modify(_: &str, modify_req: &IamTenantModifyReq, _: &TardisRelDBlConnection<'a>, _: &TardisContext) -> TardisResult<Option<RbumItemModifyReq>> {
        if modify_req.name.is_none() && modify_req.icon.is_none() && modify_req.sort.is_none() && modify_req.scope_level.is_none() && modify_req.disabled.is_none() {
            return Ok(None);
        }
        Ok(Some(RbumItemModifyReq {
            uri_path: None,
            name: modify_req.name.clone(),
            icon: modify_req.icon.clone(),
            sort: modify_req.sort,
            scope_level: modify_req.scope_level.clone(),
            disabled: modify_req.disabled,
        }))
    }

    async fn package_ext_modify(id: &str, modify_req: &IamTenantModifyReq, _: &TardisRelDBlConnection<'a>, _: &TardisContext) -> TardisResult<Option<iam_tenant::ActiveModel>> {
        if modify_req.contact_phone.is_none() {
            return Ok(None);
        }
        let mut iam_tenant = iam_tenant::ActiveModel {
            id: Set(id.to_string()),
            ..Default::default()
        };
        if let Some(contact_phone) = &modify_req.contact_phone {
            iam_tenant.contact_phone = Set(contact_phone.to_string());
        }
        Ok(Some(iam_tenant))
    }

    async fn package_item_query(query: &mut SelectStatement, _: bool, _: &RbumItemFilterReq, _: &TardisRelDBlConnection<'a>, _: &TardisContext) -> TardisResult<()> {
        query.column((iam_tenant::Entity, iam_tenant::Column::ContactPhone));
        Ok(())
    }
}

impl IamTenantServ {
    pub fn get_new_id() -> String {
        TardisFuns::field.nanoid_len(RBUM_ITEM_ID_TENANT_LEN)
    }

    pub fn get_id_by_cxt(cxt: &TardisContext) -> TardisResult<String> {
        if let Some(id) = rbum_scope_helper::get_path_item(RBUM_SCOPE_LEVEL_TENANT, &cxt.scope_paths) {
            Ok(id)
        } else {
            Err(TardisError::Unauthorized(format!("tenant id not found in tardis content {}", cxt
                .scope_paths)))
        }
    }
}
