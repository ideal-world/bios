use tardis::basic::dto::{TardisContext, TardisFunsInst};
use tardis::basic::result::TardisResult;
use tardis::web::web_resp::TardisPage;

use bios_basic::rbum::dto::rbum_rel_agg_dto::RbumRelAggResp;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;

use crate::basic::dto::iam_account_dto::{IamAccountDetailResp, IamAccountModifyReq};
use crate::basic::dto::iam_filer_dto::IamAccountFilterReq;
use crate::iam_enumeration::IAMRelKind;
use crate::basic::serv::iam_account_serv::IamAccountServ;
use crate::basic::serv::iam_rel_serv::IamRelServ;
use crate::console_passport::dto::iam_cp_account_dto::IamCpAccountModifyReq;

pub struct IamCpAccountServ;

impl<'a> IamCpAccountServ {
    pub async fn modify_account(modify_req: &mut IamCpAccountModifyReq, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<()> {
        IamAccountServ::modify_item(
            &cxt.owner,
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

    pub async fn get_account(funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<IamAccountDetailResp> {
        IamAccountServ::get_item(&cxt.owner, &IamAccountFilterReq::default(), funs, cxt).await
    }

    pub async fn delete_account(funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<u64> {
        IamAccountServ::delete_item(&cxt.owner, funs, cxt).await
    }

    pub async fn paginate_rel_roles(
        page_number: u64,
        page_size: u64,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<TardisPage<RbumRelAggResp>> {
        IamRelServ::paginate_from_rels(
            IAMRelKind::IamAccountRole,
            &cxt.owner,
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
