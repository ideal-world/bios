use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::db::reldb_client::TardisRelDBlConnection;

use bios_basic::rbum::enumeration::RbumScopeKind;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;

use crate::basic::dto::iam_tenant_dto::{IamTenantAddReq, IamTenantModifyReq};
use crate::basic::serv::iam_tenant_serv::IamTenantCrudServ;
use crate::console_system::dto::iam_cs_tenant_dto::{IamCsTenantAddReq, IamCsTenantModifyReq};

pub struct IamCsTenantServ;

impl IamCsTenantServ {
    pub async fn add_tenant<'a>(add_req: &mut IamCsTenantAddReq, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<String> {
        IamTenantCrudServ::add_item(
            &mut IamTenantAddReq {
                code: None,
                name: add_req.name.clone(),
                icon: add_req.icon.clone(),
                sort: None,
                contact_phone: add_req.contact_phone.clone(),
                scope_kind: Some(RbumScopeKind::Global),
                disabled: add_req.disabled,
            },
            db,
            cxt,
        )
        .await
    }

    pub async fn modify_tenant<'a>(id: &str, modify_req: &mut IamCsTenantModifyReq, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<()> {
        IamTenantCrudServ::modify_item(
            id,
            &mut IamTenantModifyReq {
                name: None,
                icon: None,
                sort: None,
                contact_phone: None,
                scope_kind: None,
                disabled: modify_req.disabled,
            },
            db,
            cxt,
        )
        .await
    }
}
