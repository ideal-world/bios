use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::db::reldb_client::TardisRelDBlConnection;

use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;

use crate::basic::dto::iam_account_dto::IamAccountModifyReq;
use crate::basic::serv::iam_account_serv::IamAccountCrudServ;
use crate::console_system::dto::iam_cs_account_dto::IamCsAccountModifyReq;

pub struct IamCsAccountServ;

impl IamCsAccountServ {
    pub async fn modify_account<'a>(id: &str, modify_req: &mut IamCsAccountModifyReq, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<()> {
        IamAccountCrudServ::modify_item(
            id,
            &mut IamAccountModifyReq {
                name: None,
                icon: None,
                scope_kind: None,
                disabled: modify_req.disabled,
            },
            db,
            cxt,
        )
        .await
    }
}
