use tardis::basic::dto::{TardisContext, TardisFunsInst};
use tardis::basic::result::TardisResult;

use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;

use crate::basic::dto::iam_app_dto::{IamAppDetailResp, IamAppModifyReq};
use crate::basic::dto::iam_filer_dto::IamAppFilterReq;
use crate::basic::serv::iam_app_serv::IamAppServ;
use crate::basic::serv::iam_role_serv::IamRoleServ;
use crate::console_app::dto::iam_ca_app_dto::IamCaAppModifyReq;

pub struct IamCaAppServ;

impl<'a> IamCaAppServ {
    pub async fn modify_app(modify_req: &mut IamCaAppModifyReq, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<()> {
        IamRoleServ::need_app_admin(funs, cxt).await?;
        IamAppServ::modify_item(
            &IamAppServ::get_id_by_cxt(cxt)?,
            &mut IamAppModifyReq {
                name: modify_req.name.clone(),
                icon: modify_req.icon.clone(),
                sort: modify_req.sort,
                contact_phone: modify_req.contact_phone.clone(),
                disabled: modify_req.disabled,
                scope_level: None,
            },
            funs,
            cxt,
        )
        .await
    }

    pub async fn get_app(funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<IamAppDetailResp> {
        IamRoleServ::need_app_admin(funs, cxt).await?;
        IamAppServ::get_item(&IamAppServ::get_id_by_cxt(cxt)?, &IamAppFilterReq::default(), funs, cxt).await
    }
}
