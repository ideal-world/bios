use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::db::reldb_client::TardisRelDBlConnection;
use tardis::web::web_resp::TardisPage;

use bios_basic::rbum::dto::filer_dto::RbumItemFilterReq;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;

use crate::basic::constants;
use crate::basic::dto::iam_account_dto::IamAccountAddReq;
use crate::basic::dto::iam_tenant_dto::{IamTenantAddReq, IamTenantDetailResp, IamTenantModifyReq, IamTenantSummaryResp};
use crate::basic::enumeration::{IAMRelKind, IamIdentKind};
use crate::basic::serv::iam_account_serv::IamAccountServ;
use crate::basic::serv::iam_cert_serv::IamCertServ;
use crate::basic::serv::iam_rel_serv::IamRelServ;
use crate::basic::serv::iam_role_serv::IamRoleServ;
use crate::basic::serv::iam_tenant_serv::IamTenantServ;
use crate::console_system::dto::iam_cs_tenant_dto::{IamCsTenantAddReq, IamCsTenantModifyReq};

pub struct IamCsTenantServ;

impl<'a> IamCsTenantServ {
    pub async fn add_tenant(add_req: &mut IamCsTenantAddReq, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<(String, String)> {
        IamRoleServ::need_sys_admin(db, cxt).await?;
        let tenant_id = IamTenantServ::add_item(
            &mut IamTenantAddReq {
                name: add_req.tenant_name.clone(),
                icon: add_req.tenant_icon.clone(),
                sort: None,
                contact_phone: add_req.tenant_contact_phone.clone(),
                disabled: add_req.disabled,
                scope_level: constants::RBUM_SCOPE_LEVEL_GLOBAL,
            },
            db,
            cxt,
        )
        .await?;
        let account_id = IamAccountServ::add_item_with_simple_rel(
            &mut IamAccountAddReq {
                id: None,
                name: add_req.admin_name.clone(),
                icon: None,
                disabled: add_req.disabled,
                scope_level: constants::RBUM_SCOPE_LEVEL_TENANT,
            },
            &IAMRelKind::IamAccountTenant.to_string(),
            &tenant_id,
            db,
            cxt,
        )
        .await?;
        let pwd = IamCertServ::get_new_pwd();
        IamRelServ::add_rel(
            IAMRelKind::IamAccountTenant,
            &account_id,
            &constants::get_rbum_basic_info().role_tenant_admin_id,
            None,
            None,
            db,
            cxt,
        )
        .await?;
        IamCertServ::add_ident(add_req.admin_username.0.as_str(), Some(&pwd), IamIdentKind::UserPwd, None, &account_id, db, cxt).await?;
        Ok((tenant_id, pwd))
    }

    pub async fn modify_tenant(id: &str, modify_req: &mut IamCsTenantModifyReq, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<()> {
        IamRoleServ::need_sys_admin(db, cxt).await?;
        IamTenantServ::modify_item(
            id,
            &mut IamTenantModifyReq {
                name: None,
                icon: None,
                sort: None,
                contact_phone: None,
                disabled: modify_req.disabled,
                scope_level: None,
            },
            db,
            cxt,
        )
        .await
    }

    pub async fn get_tenant(id: &str, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<IamTenantDetailResp> {
        IamRoleServ::need_sys_admin(db, cxt).await?;
        IamTenantServ::get_item(id, &RbumItemFilterReq::default(), db, cxt).await
    }

    pub async fn paginate_tenants(
        filter: &RbumItemFilterReq,
        page_number: u64,
        page_size: u64,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        db: &TardisRelDBlConnection<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<TardisPage<IamTenantSummaryResp>> {
        IamRoleServ::need_sys_admin(db, cxt).await?;
        IamTenantServ::paginate_items(filter, page_number, page_size, desc_sort_by_create, desc_sort_by_update, db, cxt).await
    }

    pub async fn delete_tenant(id: &str, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<u64> {
        IamRoleServ::need_sys_admin(db, cxt).await?;
        IamTenantServ::delete_item(id, db, cxt).await
    }
}
