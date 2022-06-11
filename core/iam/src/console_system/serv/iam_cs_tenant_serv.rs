use tardis::basic::dto::TardisContext;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::{TardisFuns, TardisFunsInst};

use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;

use crate::basic::dto::iam_account_dto::IamAccountAggAddReq;
use crate::basic::dto::iam_tenant_dto::IamTenantAddReq;
use crate::basic::serv::iam_account_serv::IamAccountServ;
use crate::basic::serv::iam_cert_serv::IamCertServ;
use crate::basic::serv::iam_set_serv::IamSetServ;
use crate::basic::serv::iam_tenant_serv::IamTenantServ;
use crate::console_system::dto::iam_cs_tenant_dto::IamCsTenantAddReq;
use crate::iam_config::IamBasicConfigApi;
use crate::iam_constants;
use crate::iam_constants::RBUM_SCOPE_LEVEL_TENANT;

pub struct IamCsTenantServ;

impl<'a> IamCsTenantServ {
    pub async fn add_tenant(add_req: &mut IamCsTenantAddReq, funs: &TardisFunsInst<'a>) -> TardisResult<(String, String)> {
        let tenant_admin_id = TardisFuns::field.nanoid();
        let tenant_id = IamTenantServ::get_new_id();
        let tenant_ctx = TardisContext {
            own_paths: tenant_id.clone(),
            ak: "".to_string(),
            roles: vec![],
            groups: vec![],
            owner: tenant_admin_id.to_string(),
        };
        IamTenantServ::add_item(
            &mut IamTenantAddReq {
                id: Some(TrimString(tenant_id.clone())),
                name: add_req.tenant_name.clone(),
                icon: add_req.tenant_icon.clone(),
                sort: None,
                contact_phone: add_req.tenant_contact_phone.clone(),
                disabled: add_req.disabled,
                scope_level: Some(iam_constants::RBUM_SCOPE_LEVEL_GLOBAL),
                note: add_req.tenant_note.clone(),
            },
            funs,
            &tenant_ctx,
        )
        .await?;

        IamSetServ::init_set(true, RBUM_SCOPE_LEVEL_TENANT, funs, &tenant_ctx).await?;
        IamSetServ::init_set(false, RBUM_SCOPE_LEVEL_TENANT, funs, &tenant_ctx).await?;

        IamCertServ::init_default_ident_conf(
            add_req.cert_conf_by_user_pwd.clone(),
            add_req.cert_conf_by_phone_vcode.clone(),
            add_req.cert_conf_by_mail_vcode.clone(),
            funs,
            &tenant_ctx,
        )
        .await?;

        let pwd = if let Some(admin_password) = &add_req.admin_password {
            admin_password.to_string()
        } else {
            IamCertServ::get_new_pwd()
        };

        IamAccountServ::add_account_agg(
            &IamAccountAggAddReq {
                id: Some(TrimString(tenant_admin_id.clone())),
                name: add_req.admin_name.clone(),
                cert_user_name: TrimString(add_req.admin_username.0.to_string()),
                cert_password: TrimString(pwd.to_string()),
                cert_phone: None,
                cert_mail: None,
                icon: None,
                disabled: add_req.disabled,
                scope_level: Some(iam_constants::RBUM_SCOPE_LEVEL_TENANT),
                role_ids: Some(vec![funs.iam_basic_role_tenant_admin_id()]),
                exts: Default::default(),
            },
            funs,
            &tenant_ctx,
        )
        .await?;

        Ok((tenant_id, pwd))
    }
}
