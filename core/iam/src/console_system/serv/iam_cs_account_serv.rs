use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::db::reldb_client::TardisRelDBlConnection;
use tardis::web::web_resp::TardisPage;

use bios_basic::rbum::dto::filer_dto::RbumItemFilterReq;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;

use crate::basic::dto::iam_account_dto::{IamAccountDetailResp, IamAccountModifyReq, IamAccountSummaryResp};
use crate::basic::serv::iam_account_serv::IamAccountServ;
use crate::basic::serv::iam_role_serv::IamRoleServ;
use crate::console_system::dto::iam_cs_account_dto::IamCsAccountModifyReq;

pub struct IamCsAccountServ;

impl<'a> IamCsAccountServ {
    pub async fn modify_account(id: &str, modify_req: &mut IamCsAccountModifyReq, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<()> {
        IamRoleServ::need_sys_admin(db, cxt).await?;
        IamAccountServ::modify_item(
            id,
            &mut IamAccountModifyReq {
                name: None,
                icon: None,
                disabled: modify_req.disabled,
                scope_level: None,
            },
            db,
            cxt,
        )
        .await
    }

    pub async fn get_account(id: &str, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<IamAccountDetailResp> {
        IamRoleServ::need_sys_admin(db, cxt).await?;
        IamAccountServ::get_item(
            id,
            &RbumItemFilterReq {
                ignore_scope_check: true,
                ..Default::default()
            },
            db,
            cxt,
        )
        .await
    }

    pub async fn paginate_accounts(
        tenant_id: String,
        q_name: Option<String>,
        page_number: u64,
        page_size: u64,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        db: &TardisRelDBlConnection<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<TardisPage<IamAccountSummaryResp>> {
        IamRoleServ::need_sys_admin(db, cxt).await?;
        IamAccountServ::paginate_items(
            &RbumItemFilterReq {
                name: q_name,
                rel_scope_paths: Some(tenant_id),
                ignore_scope_check: true,
                ..Default::default()
            },
            page_number,
            page_size,
            desc_sort_by_create,
            desc_sort_by_update,
            db,
            cxt,
        )
        .await
    }
}
