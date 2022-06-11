use tardis::basic::dto::TardisContext;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::TardisFunsInst;

use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;

use crate::basic::dto::iam_app_dto::IamAppAddReq;
use crate::basic::serv::iam_app_serv::IamAppServ;
use crate::basic::serv::iam_role_serv::IamRoleServ;
use crate::basic::serv::iam_set_serv::IamSetServ;
use crate::console_tenant::dto::iam_ct_app_dto::IamCtAppAddReq;
use crate::iam_config::IamBasicConfigApi;
use crate::iam_constants;
use crate::iam_constants::RBUM_SCOPE_LEVEL_APP;

pub struct IamCtAppServ;

impl<'a> IamCtAppServ {
    pub async fn add_app(add_req: &mut IamCtAppAddReq, funs: &TardisFunsInst<'a>, tenant_ctx: &TardisContext) -> TardisResult<String> {
        let app_id = IamAppServ::get_new_id();
        let app_ctx = TardisContext {
            own_paths: format!("{}/{}", tenant_ctx.own_paths, app_id),
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
            &app_ctx,
        )
        .await?;

        IamAppServ::add_rel_account(&app_id, &add_req.admin_id, funs, &app_ctx).await?;
        IamRoleServ::add_rel_account(&funs.iam_basic_role_app_admin_id(), &add_req.admin_id, funs, &app_ctx).await?;

        IamSetServ::init_set(true, RBUM_SCOPE_LEVEL_APP, funs, &app_ctx).await?;
        IamSetServ::init_set(false, RBUM_SCOPE_LEVEL_APP, funs, &app_ctx).await?;

        Ok(app_id)
    }
}
