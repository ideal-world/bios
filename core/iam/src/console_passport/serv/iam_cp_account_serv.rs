use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::db::reldb_client::TardisRelDBlConnection;
use tardis::web::web_resp::TardisPage;

use bios_basic::rbum::dto::filer_dto::RbumItemFilterReq;
use bios_basic::rbum::dto::rbum_rel_agg_dto::RbumRelAggResp;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;

use crate::basic::dto::iam_account_dto::{IamAccountDetailResp, IamAccountModifyReq};
use crate::basic::enumeration::IAMRelKind;
use crate::basic::serv::iam_account_serv::IamAccountServ;
use crate::basic::serv::iam_rel_serv::IamRelServ;
use crate::basic::serv::iam_role_serv::IamRoleServ;
use crate::console_passport::dto::iam_cp_account_dto::IamCpAccountModifyReq;

pub struct IamCpAccountServ;

impl<'a> IamCpAccountServ {
    pub async fn modify_account(modify_req: &mut IamCpAccountModifyReq, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<()> {
        IamAccountServ::modify_item(
            &cxt.account_id,
            &mut IamAccountModifyReq {
                name: modify_req.name.clone(),
                icon: modify_req.icon.clone(),
                disabled: modify_req.disabled,
                scope_level: None,
            },
            db,
            cxt,
        )
        .await
    }

    pub async fn get_account(funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<IamAccountDetailResp> {
        IamAccountServ::get_item(&cxt.account_id, &RbumItemFilterReq::default(), db, cxt).await
    }

    pub async fn delete_account(funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<u64> {
        IamAccountServ::delete_item(&cxt.account_id, db, cxt).await
    }

    pub async fn paginate_rel_roles(
        page_number: u64,
        page_size: u64,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<TardisPage<RbumRelAggResp>> {
        IamRoleServ::need_tenant_admin(db, cxt).await?;
        IamRelServ::paginate_to_rels(
            IAMRelKind::IamRoleAccount,
            &cxt.account_id,
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
