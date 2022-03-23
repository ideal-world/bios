use tardis::basic::dto::TardisContext;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::db::reldb_client::TardisRelDBlConnection;
use tardis::web::web_resp::TardisPage;

use bios_basic::rbum::dto::filer_dto::RbumItemFilterReq;
use bios_basic::rbum::enumeration::RbumScopeKind;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;

use crate::basic::constants;
use crate::basic::dto::iam_account_dto::{IamAccountDetailResp, IamAccountSummaryResp};
use crate::basic::dto::iam_app_dto::{IamAppAddReq, IamAppDetailResp, IamAppModifyReq, IamAppSummaryResp};
use crate::basic::serv::iam_account_serv::IamAccountServ;
use crate::basic::serv::iam_app_serv::IamAppServ;
use crate::basic::serv::iam_role_serv::IamRoleServ;
use crate::basic::serv::iam_tenant_serv::IamTenantServ;
use crate::console_tenant::dto::iam_ct_app_dto::{IamCtAppAddReq, IamCtAppModifyReq};

pub struct IamCtAppServ;

impl<'a> IamCtAppServ {
    pub async fn add_app(add_req: &mut IamCtAppAddReq, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<String> {
        IamRoleServ::need_tenant_admin(db, cxt).await?;
        IamAppServ::add_item_with_simple_rel(
            &mut IamAppAddReq {
                code: Some(TrimString(IamAppServ::get_new_code(&cxt.tenant_code))),
                name: add_req.name.clone(),
                icon: add_req.icon.clone(),
                sort: None,
                contact_phone: add_req.contact_phone.clone(),
                scope_kind: Some(RbumScopeKind::Tenant),
                disabled: add_req.disabled,
            },
            constants::RBUM_REL_BIND,
            &IamTenantServ::get_id_by_cxt(db, cxt).await?,
            db,
            cxt,
        )
        .await
    }

    pub async fn modify_app(id: &str, modify_req: &mut IamCtAppModifyReq, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<()> {
        IamRoleServ::need_tenant_admin(db, cxt).await?;
        IamAppServ::modify_item(
            id,
            &mut IamAppModifyReq {
                name: modify_req.name.clone(),
                icon: modify_req.icon.clone(),
                sort: modify_req.sort,
                contact_phone: modify_req.contact_phone.clone(),
                scope_kind: None,
                disabled: modify_req.disabled,
            },
            db,
            cxt,
        )
        .await
    }

    pub async fn get_app(id: &str, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<IamAppDetailResp> {
        IamRoleServ::need_tenant_admin(db, cxt).await?;
        IamAppServ::get_item(id, &RbumItemFilterReq::default(), db, cxt).await
    }

    pub async fn paginate_apps(
        q_name: Option<String>,
        page_number: u64,
        page_size: u64,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        db: &TardisRelDBlConnection<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<TardisPage<IamAppSummaryResp>> {
        IamRoleServ::need_tenant_admin(db, cxt).await?;
        IamAppServ::paginate_items(
            &RbumItemFilterReq {
                name: q_name,
                iam_tenant_id: Some(IamTenantServ::get_id_by_cxt(db, cxt).await?),
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

    pub async fn delete_app(id: &str, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<u64> {
        IamRoleServ::need_tenant_admin(db, cxt).await?;
        IamAppServ::delete_item(id, db, cxt).await
    }
}
