use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::db::reldb_client::TardisRelDBlConnection;

use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;

use crate::basic::dto::iam_tenant_dto::IamTenantModifyReq;
use crate::basic::serv::iam_role_serv::IamRoleServ;
use crate::basic::serv::iam_tenant_serv::IamTenantServ;
use crate::console_tenant::dto::iam_ct_tenant_dto::IamCtTenantModifyReq;

pub struct IamCtTenantServ;

impl IamCtTenantServ {
    pub async fn modify_tenant<'a>(modify_req: &mut IamCtTenantModifyReq, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<()> {
        IamRoleServ::need_tenant_admin(db, cxt).await?;
        IamTenantServ::modify_item(
            &IamTenantServ::get_id_by_cxt(cxt)?,
            &mut IamTenantModifyReq {
                name: modify_req.name.clone(),
                icon: modify_req.icon.clone(),
                sort: modify_req.sort,
                contact_phone: modify_req.contact_phone.clone(),
                disabled: modify_req.disabled,
                scope_level: modify_req.scope_level,
            },
            db,
            cxt,
        )
        .await
    }
}
