use tardis::basic::dto::{TardisContext, TardisFunsInst};
use tardis::basic::result::TardisResult;
use tardis::web::web_resp::TardisPage;

use bios_basic::rbum::dto::rbum_filer_dto::RbumBasicFilterReq;
use bios_basic::rbum::dto::rbum_rel_agg_dto::RbumRelAggResp;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;

use crate::iam_constants;
use crate::basic::dto::iam_account_dto::{IamAccountAddReq, IamAccountDetailResp, IamAccountModifyReq, IamAccountSummaryResp};
use crate::basic::dto::iam_filer_dto::IamAccountFilterReq;
use crate::iam_enumeration::IAMRelKind;
use crate::basic::serv::iam_account_serv::IamAccountServ;
use crate::basic::serv::iam_rel_serv::IamRelServ;
use crate::basic::serv::iam_role_serv::IamRoleServ;
use crate::basic::serv::iam_tenant_serv::IamTenantServ;
use crate::console_tenant::dto::iam_ct_account_dto::{IamCtAccountAddReq, IamCtAccountModifyReq};

pub struct IamCtAccountServ;

impl<'a> IamCtAccountServ {
    pub async fn add_account(add_req: &mut IamCtAccountAddReq, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<String> {
        IamRoleServ::need_tenant_admin(funs, cxt).await?;
        IamAccountServ::add_item_with_simple_rel_by_to(
            &mut IamAccountAddReq {
                id: None,
                name: add_req.name.clone(),
                icon: add_req.icon.clone(),
                disabled: add_req.disabled,
                scope_level: iam_constants::RBUM_SCOPE_LEVEL_TENANT,
            },
            &IAMRelKind::IamTenantAccount.to_string(),
            &IamTenantServ::get_id_by_cxt(cxt)?,
            funs,
            cxt,
        )
        .await
    }

    pub async fn modify_account(id: &str, modify_req: &mut IamCtAccountModifyReq, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<()> {
        IamRoleServ::need_tenant_admin(funs, cxt).await?;
        IamAccountServ::modify_item(
            id,
            &mut IamAccountModifyReq {
                name: modify_req.name.clone(),
                icon: modify_req.icon.clone(),
                disabled: modify_req.disabled,
                scope_level: None,
            },
            funs,
            cxt,
        )
        .await
    }

    pub async fn get_account(id: &str, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<IamAccountDetailResp> {
        IamRoleServ::need_tenant_admin(funs, cxt).await?;
        IamAccountServ::get_item(id, &IamAccountFilterReq::default(), funs, cxt).await
    }

    pub async fn paginate_accounts(
        q_id: Option<String>,
        q_name: Option<String>,
        page_number: u64,
        page_size: u64,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<TardisPage<IamAccountSummaryResp>> {
        IamRoleServ::need_tenant_admin(funs, cxt).await?;
        IamAccountServ::paginate_items(
            &IamAccountFilterReq {
                basic: RbumBasicFilterReq {
                    ids:q_id.map(|id| vec![id]),
                    name: q_name,
                    own_paths: Some(IamTenantServ::get_id_by_cxt(cxt)?),
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

    pub async fn delete_account(id: &str, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<u64> {
        IamRoleServ::need_tenant_admin(funs, cxt).await?;
        IamAccountServ::delete_item(id, funs, cxt).await
    }

    pub async fn paginate_rel_roles(
        iam_account_id: &str,
        page_number: u64,
        page_size: u64,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<TardisPage<RbumRelAggResp>> {
        IamRoleServ::need_tenant_admin(funs, cxt).await?;
        IamRelServ::paginate_from_rels(
            IAMRelKind::IamAccountRole,
            iam_account_id,
            page_number,
            page_size,
            desc_sort_by_create,
            desc_sort_by_update,
            funs,
            cxt,
        )
        .await
    }
}
