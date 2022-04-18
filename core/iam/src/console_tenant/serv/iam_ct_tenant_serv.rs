use tardis::basic::dto::{TardisContext, TardisFunsInst};
use tardis::basic::result::TardisResult;

use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;

use crate::basic::dto::iam_filer_dto::IamTenantFilterReq;
use crate::basic::dto::iam_tenant_dto::{IamTenantDetailResp, IamTenantModifyReq};
use crate::basic::serv::iam_role_serv::IamRoleServ;
use crate::basic::serv::iam_tenant_serv::IamTenantServ;
use crate::console_tenant::dto::iam_ct_tenant_dto::IamCtTenantModifyReq;

pub struct IamCtTenantServ;

impl IamCtTenantServ {
    pub async fn get_tenant<'a>(funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<IamTenantDetailResp> {
        IamRoleServ::need_tenant_admin(funs, cxt).await?;
        IamTenantServ::get_item(&IamTenantServ::get_id_by_cxt(cxt)?, &IamTenantFilterReq::default
            (), funs, cxt).await
    }

    pub async fn modify_tenant<'a>(modify_req: &mut IamCtTenantModifyReq, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<()> {
        IamRoleServ::need_tenant_admin(funs, cxt).await?;
        IamTenantServ::modify_item(
            &IamTenantServ::get_id_by_cxt(cxt)?,
            &mut IamTenantModifyReq {
                name: modify_req.name.clone(),
                icon: modify_req.icon.clone(),
                sort: modify_req.sort,
                contact_phone: modify_req.contact_phone.clone(),
                disabled: modify_req.disabled,
                scope_level: modify_req.scope_level.clone(),
            },
            funs,
            cxt,
        )
        .await
    }
}
