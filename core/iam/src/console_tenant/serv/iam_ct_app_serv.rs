use tardis::basic::dto::{TardisContext, TardisFunsInst};
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;

use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;

use crate::basic::dto::iam_app_dto::IamAppAddReq;
use crate::basic::serv::iam_app_serv::IamAppServ;
use crate::basic::serv::iam_rel_serv::IamRelServ;
use crate::basic::serv::iam_set_serv::IamSetServ;
use crate::console_tenant::dto::iam_ct_app_dto::IamCtAppAddReq;
use crate::iam_config::IamBasicInfoManager;
use crate::iam_constants;
use crate::iam_constants::RBUM_SCOPE_LEVEL_APP;
use crate::iam_enumeration::IamRelKind;

pub struct IamCtAppServ;

impl<'a> IamCtAppServ {
    pub async fn add_app(add_req: &mut IamCtAppAddReq, funs: &TardisFunsInst<'a>, tenant_cxt: &TardisContext) -> TardisResult<String> {
        let app_id = IamAppServ::get_new_id();
        let app_cxt = TardisContext {
            own_paths: format!("{}/{}", tenant_cxt.own_paths, app_id),
            ak: "".to_string(),
            roles: vec![],
            groups: vec![],
            owner: add_req.admin_id.clone(),
        };
        IamAppServ::add_item(
            &mut IamAppAddReq {
                id: Some(TrimString(app_id.clone())),
                name: add_req.app_name.clone(),
                icon: add_req.app_icon.clone(),
                sort: add_req.app_sort,
                contact_phone: add_req.app_contact_phone.clone(),
                disabled: add_req.disabled,
                scope_level: Some(iam_constants::RBUM_SCOPE_LEVEL_TENANT),
            },
            funs,
            &app_cxt,
        )
        .await?;

        IamRelServ::add_simple_rel(
            IamRelKind::IamAccountRole,
            &add_req.admin_id,
            &IamBasicInfoManager::get().role_app_admin_id,
            None,
            None,
            funs,
            &app_cxt,
        )
        .await?;

        IamSetServ::init_set(true, RBUM_SCOPE_LEVEL_APP, funs, &app_cxt).await?;
        IamSetServ::init_set(false, RBUM_SCOPE_LEVEL_APP, funs, &app_cxt).await?;

        Ok(app_id)
    }
}
