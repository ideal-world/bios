use tardis::basic::dto::{TardisContext, TardisFunsInst};
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::web::web_resp::TardisPage;

use bios_basic::rbum::dto::rbum_filer_dto::RbumBasicFilterReq;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;

use crate::basic::dto::iam_app_dto::{IamAppAddReq, IamAppDetailResp, IamAppModifyReq, IamAppSummaryResp};
use crate::basic::dto::iam_filer_dto::IamAppFilterReq;
use crate::basic::serv::iam_app_serv::IamAppServ;
use crate::basic::serv::iam_rel_serv::IamRelServ;
use crate::basic::serv::iam_role_serv::IamRoleServ;
use crate::basic::serv::iam_set_serv::IamSetServ;
use crate::console_tenant::dto::iam_ct_app_dto::{IamCtAppAddReq, IamCtAppModifyReq};
use crate::iam_config::IamBasicInfoManager;
use crate::iam_constants;
use crate::iam_constants::RBUM_SCOPE_LEVEL_APP;
use crate::iam_enumeration::IamRelKind;

pub struct IamCtAppServ;

impl<'a> IamCtAppServ {
    pub async fn add_app(add_req: &mut IamCtAppAddReq, funs: &TardisFunsInst<'a>, tenant_cxt: &TardisContext) -> TardisResult<String> {
        IamRoleServ::need_tenant_admin(funs, tenant_cxt).await?;

        let app_id = IamAppServ::get_new_id();
        let app_cxt = TardisContext {
            own_paths: format!("{}/{}", tenant_cxt.own_paths, app_id),
            ak: "".to_string(),
            token: "".to_string(),
            token_kind: "".to_string(),
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

        IamRelServ::add_rel(
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

    pub async fn modify_app(id: &str, modify_req: &mut IamCtAppModifyReq, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<()> {
        IamRoleServ::need_tenant_admin(funs, cxt).await?;
        IamAppServ::modify_item(
            id,
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

    pub async fn get_app(id: &str, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<IamAppDetailResp> {
        IamRoleServ::need_tenant_admin(funs, cxt).await?;
        IamAppServ::get_item(id, &IamAppFilterReq::default(), funs, cxt).await
    }

    pub async fn paginate_apps(
        q_id: Option<String>,
        q_name: Option<String>,
        page_number: u64,
        page_size: u64,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<TardisPage<IamAppSummaryResp>> {
        IamRoleServ::need_tenant_admin(funs, cxt).await?;
        IamAppServ::paginate_items(
            &IamAppFilterReq {
                basic: RbumBasicFilterReq {
                    ids: q_id.map(|id| vec![id]),
                    name: q_name,
                    own_paths: Some(cxt.own_paths.clone()),
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                ..Default::default()
            },
            page_number,
            page_size,
            desc_sort_by_create,
            desc_sort_by_update,
            funs,
            cxt,
        )
        .await
    }

    pub async fn delete_app(id: &str, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<u64> {
        IamRoleServ::need_tenant_admin(funs, cxt).await?;
        IamAppServ::delete_item_with_all_rels(id, funs, cxt).await
    }
}
