use tardis::basic::dto::{TardisContext, TardisFunsInst};
use tardis::basic::result::TardisResult;
use tardis::web::web_resp::TardisPage;

use bios_basic::rbum::dto::rbum_filer_dto::RbumBasicFilterReq;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;

use crate::basic::dto::iam_account_dto::{IamAccountDetailResp, IamAccountModifyReq, IamAccountSummaryResp};
use crate::basic::dto::iam_filer_dto::IamAccountFilterReq;
use crate::basic::serv::iam_account_serv::IamAccountServ;
use crate::basic::serv::iam_role_serv::IamRoleServ;
use crate::console_system::dto::iam_cs_account_dto::IamCsAccountModifyReq;

pub struct IamCsAccountServ;

impl<'a> IamCsAccountServ {
    pub async fn modify_account(id: &str, modify_req: &mut IamCsAccountModifyReq, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<()> {
        IamRoleServ::need_sys_admin(funs, cxt).await?;
        IamAccountServ::modify_item(
            id,
            &mut IamAccountModifyReq {
                name: None,
                icon: None,
                disabled: modify_req.disabled,
                scope_level: None,
            },
            funs,
            cxt,
        )
        .await
    }

    pub async fn get_account(id: &str, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<IamAccountDetailResp> {
        IamRoleServ::need_sys_admin(funs, cxt).await?;
        IamAccountServ::get_item(
            id,
            &IamAccountFilterReq {
                basic: RbumBasicFilterReq {
                    ignore_scope: true,
                    ..Default::default()
                },
                ..Default::default()
            },
            funs,
            cxt,
        )
        .await
    }

    pub async fn paginate_accounts(
        tenant_id: String,
        q_id: Option<String>,
        q_name: Option<String>,
        page_number: u64,
        page_size: u64,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<TardisPage<IamAccountSummaryResp>> {
        IamRoleServ::need_sys_admin(funs, cxt).await?;
        IamAccountServ::paginate_items(
            &IamAccountFilterReq {
                basic: RbumBasicFilterReq {
                    ignore_scope: true,
                    ids: q_id.map(|id| vec![id]),
                    name: q_name,
                    own_paths_with_sub: Some(tenant_id),
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
}
