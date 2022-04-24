use tardis::basic::dto::{TardisContext, TardisFunsInst};
use tardis::basic::error::TardisError;
use tardis::basic::result::TardisResult;
use tardis::web::web_resp::TardisPage;

use bios_basic::rbum::dto::rbum_filer_dto::RbumBasicFilterReq;
use bios_basic::rbum::dto::rbum_rel_agg_dto::RbumRelAggResp;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;

use crate::basic::dto::iam_filer_dto::IamRoleFilterReq;
use crate::basic::dto::iam_role_dto::{IamRoleAddReq, IamRoleDetailResp, IamRoleModifyReq, IamRoleSummaryResp};
use crate::basic::serv::iam_rel_serv::IamRelServ;
use crate::basic::serv::iam_role_serv::IamRoleServ;
use crate::console_app::dto::iam_ca_role_dto::{IamCaRoleAddReq, IamCaRoleModifyReq};
use crate::iam_config::IamBasicInfoManager;
use crate::iam_constants;
use crate::iam_enumeration::IAMRelKind;

pub struct IamCaRoleServ;

impl<'a> IamCaRoleServ {
    pub async fn add_role(add_req: &mut IamCaRoleAddReq, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<String> {
        IamRoleServ::need_app_admin(funs, cxt).await?;
        IamRoleServ::add_item(
            &mut IamRoleAddReq {
                name: add_req.name.clone(),
                icon: add_req.icon.clone(),
                disabled: add_req.disabled,
                scope_level: iam_constants::RBUM_SCOPE_LEVEL_APP,
                sort: add_req.sort,
            },
            funs,
            cxt,
        )
        .await
    }

    pub async fn modify_role(id: &str, modify_req: &mut IamCaRoleModifyReq, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<()> {
        IamRoleServ::need_app_admin(funs, cxt).await?;
        IamRoleServ::modify_item(
            id,
            &mut IamRoleModifyReq {
                name: modify_req.name.clone(),
                icon: modify_req.icon.clone(),
                disabled: modify_req.disabled,
                scope_level: None,
                sort: modify_req.sort,
            },
            funs,
            cxt,
        )
        .await
    }

    pub async fn get_role(id: &str, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<IamRoleDetailResp> {
        IamRoleServ::need_app_admin(funs, cxt).await?;
        IamRoleServ::get_item(id, &IamRoleFilterReq::default(), funs, cxt).await
    }

    pub async fn paginate_roles(
        q_id: Option<String>,
        q_name: Option<String>,
        page_number: u64,
        page_size: u64,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<TardisPage<IamRoleSummaryResp>> {
        IamRoleServ::need_app_admin(funs, cxt).await?;
        IamRoleServ::paginate_items(
            &IamRoleFilterReq {
                basic: RbumBasicFilterReq {
                    ids: q_id.map(|id| vec![id]),
                    name: q_name,
                    own_paths: Some(cxt.own_paths.clone()),
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

    pub async fn delete_role(id: &str, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<u64> {
        IamRoleServ::need_app_admin(funs, cxt).await?;
        IamRoleServ::delete_item_with_all_rels(id, funs, cxt).await
    }

    pub async fn add_rel_account(
        role_id: &str,
        account_id: &str,
        start_timestamp: Option<i64>,
        end_timestamp: Option<i64>,
        funs: &TardisFunsInst<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<()> {
        IamRoleServ::need_app_admin(funs, cxt).await?;
        if IamBasicInfoManager::get().role_sys_admin_id == role_id || IamBasicInfoManager::get().role_tenant_admin_id == role_id {
            return Err(TardisError::BadRequest("The associated role is invalid.".to_string()));
        }
        IamRelServ::add_rel(IAMRelKind::IamAccountRole, account_id, role_id, start_timestamp, end_timestamp, funs, cxt).await
    }

    pub async fn paginate_rel_accounts(
        role_id: &str,
        page_number: u64,
        page_size: u64,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<TardisPage<RbumRelAggResp>> {
        IamRoleServ::need_app_admin(funs, cxt).await?;
        IamRelServ::paginate_to_rels(
            IAMRelKind::IamAccountRole,
            role_id,
            page_number,
            page_size,
            desc_sort_by_create,
            desc_sort_by_update,
            funs,
            cxt,
        )
        .await
    }

    pub async fn add_rel_http_res(
        role_id: &str,
        http_res_id: &str,
        start_timestamp: Option<i64>,
        end_timestamp: Option<i64>,
        funs: &TardisFunsInst<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<()> {
        IamRoleServ::need_app_admin(funs, cxt).await?;
        IamRelServ::add_rel(IAMRelKind::IamHttpResRole, http_res_id, role_id, start_timestamp, end_timestamp, funs, cxt).await
    }

    pub async fn paginate_rel_http_res(
        role_id: &str,
        page_number: u64,
        page_size: u64,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<TardisPage<RbumRelAggResp>> {
        IamRoleServ::need_app_admin(funs, cxt).await?;
        IamRelServ::paginate_to_rels(
            IAMRelKind::IamHttpResRole,
            role_id,
            page_number,
            page_size,
            desc_sort_by_create,
            desc_sort_by_update,
            funs,
            cxt,
        )
        .await
    }

    pub async fn delete_rel(id: &str, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<()> {
        IamRoleServ::need_app_admin(funs, cxt).await?;
        IamRelServ::delete_rel(id, funs, cxt).await
    }
}
