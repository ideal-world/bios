use tardis::basic::dto::{TardisContext, TardisFunsInst};
use tardis::basic::result::TardisResult;
use tardis::web::web_resp::TardisPage;

use bios_basic::rbum::dto::rbum_filer_dto::RbumBasicFilterReq;
use bios_basic::rbum::dto::rbum_rel_agg_dto::RbumRelAggResp;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;

use crate::basic::dto::iam_filer_dto::IamResFilterReq;
use crate::basic::dto::iam_res_dto::{IamResAddReq, IamResDetailResp, IamResModifyReq, IamResSummaryResp};
use crate::basic::serv::iam_rel_serv::IamRelServ;
use crate::basic::serv::iam_res_serv::IamResServ;
use crate::basic::serv::iam_role_serv::IamRoleServ;
use crate::console_app::dto::iam_ca_res_dto::{IamCaResAddReq, IamCaResModifyReq};
use crate::iam_constants;
use crate::iam_enumeration::{IAMRelKind, IamResKind};

pub struct IamCaResServ;

impl<'a> IamCaResServ {
    pub async fn add_res(add_req: &mut IamCaResAddReq, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<String> {
        IamRoleServ::need_app_admin(funs, cxt).await?;
        IamResServ::add_item(
            &mut IamResAddReq {
                name: add_req.name.clone(),
                code: add_req.code.clone(),
                icon: add_req.icon.clone(),
                disabled: add_req.disabled,
                scope_level: iam_constants::RBUM_SCOPE_LEVEL_APP,
                sort: add_req.sort,
                method: add_req.method.clone(),
                hide: add_req.hide,
                kind: add_req.kind.clone(),
                action: add_req.action.clone(),
            },
            funs,
            cxt,
        )
        .await
    }

    pub async fn modify_res(id: &str, modify_req: &mut IamCaResModifyReq, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<()> {
        IamRoleServ::need_app_admin(funs, cxt).await?;
        IamResServ::modify_item(
            id,
            &mut IamResModifyReq {
                name: modify_req.name.clone(),
                code: modify_req.code.clone(),
                icon: modify_req.icon.clone(),
                disabled: modify_req.disabled,
                scope_level: None,
                sort: modify_req.sort,
                method: modify_req.method.clone(),
                hide: modify_req.hide,
                action: modify_req.action.clone(),
            },
            funs,
            cxt,
        )
        .await
    }

    pub async fn get_res(id: &str, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<IamResDetailResp> {
        IamRoleServ::need_app_admin(funs, cxt).await?;
        IamResServ::get_item(id, &IamResFilterReq::default(), funs, cxt).await
    }

    pub async fn paginate_res(
        q_kind: IamResKind,
        q_id: Option<String>,
        q_name: Option<String>,
        page_number: u64,
        page_size: u64,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<TardisPage<IamResSummaryResp>> {
        IamRoleServ::need_app_admin(funs, cxt).await?;
        IamResServ::paginate_items(
            &IamResFilterReq {
                basic: RbumBasicFilterReq {
                    ids: q_id.map(|id| vec![id]),
                    name: q_name,
                    own_paths: Some(cxt.own_paths.clone()),
                    ..Default::default()
                },
                kind: Some(q_kind),
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

    pub async fn paginate_rel_roles(
        res_id: &str,
        page_number: u64,
        page_size: u64,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<TardisPage<RbumRelAggResp>> {
        IamRoleServ::need_app_admin(funs, cxt).await?;
        IamRelServ::paginate_from_rels(
            IAMRelKind::IamResRole,
            false,
            res_id,
            page_number,
            page_size,
            desc_sort_by_create,
            desc_sort_by_update,
            funs,
            cxt,
        )
        .await
    }

    pub async fn delete_res(id: &str, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<u64> {
        IamRoleServ::need_app_admin(funs, cxt).await?;
        IamResServ::delete_item_with_all_rels(id, funs, cxt).await
    }
}
