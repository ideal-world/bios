use tardis::basic::dto::{TardisContext, TardisFunsInst};
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::web::web_resp::TardisPage;

use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;

use crate::iam_constants;
use crate::basic::dto::iam_account_dto::IamAccountAddReq;
use crate::basic::dto::iam_cert_conf_dto::{IamMailVCodeCertConfAddOrModifyReq, IamPhoneVCodeCertConfAddOrModifyReq, IamTokenCertConfAddReq, IamUserPwdCertConfAddOrModifyReq};
use crate::basic::dto::iam_cert_dto::IamUserPwdCertAddReq;
use crate::basic::dto::iam_filer_dto::IamTenantFilterReq;
use crate::basic::dto::iam_tenant_dto::{IamTenantAddReq, IamTenantDetailResp, IamTenantModifyReq, IamTenantSummaryResp};
use crate::iam_enumeration::{IAMRelKind, IamCertTokenKind};
use crate::basic::serv::iam_account_serv::IamAccountServ;
use crate::basic::serv::iam_cert_mail_vcode_serv::IamCertMailVCodeServ;
use crate::basic::serv::iam_cert_phone_vcode_serv::IamCertPhoneVCodeServ;
use crate::basic::serv::iam_cert_serv::IamCertServ;
use crate::basic::serv::iam_cert_token_serv::IamCertTokenServ;
use crate::basic::serv::iam_cert_user_pwd_serv::IamCertUserPwdServ;
use crate::basic::serv::iam_rel_serv::IamRelServ;
use crate::basic::serv::iam_role_serv::IamRoleServ;
use crate::basic::serv::iam_tenant_serv::IamTenantServ;
use crate::console_system::dto::iam_cs_tenant_dto::{IamCsTenantAddReq, IamCsTenantModifyReq};

pub struct IamCsTenantServ;

impl<'a> IamCsTenantServ {
    pub async fn add_tenant(add_req: &mut IamCsTenantAddReq, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<(String, String)> {
        IamRoleServ::need_sys_admin(funs, cxt).await?;
        let tenant_id = IamTenantServ::add_item(
            &mut IamTenantAddReq {
                name: add_req.tenant_name.clone(),
                icon: add_req.tenant_icon.clone(),
                sort: None,
                contact_phone: add_req.tenant_contact_phone.clone(),
                disabled: add_req.disabled,
                scope_level: iam_constants::RBUM_SCOPE_LEVEL_GLOBAL,
            },
            funs,
            cxt,
        )
        .await?;
        let account_id = IamAccountServ::add_item_with_simple_rel(
            &mut IamAccountAddReq {
                id: None,
                name: add_req.admin_name.clone(),
                icon: None,
                disabled: add_req.disabled,
                scope_level: iam_constants::RBUM_SCOPE_LEVEL_TENANT,
            },
            &IAMRelKind::IamAccountTenant.to_string(),
            &tenant_id,
            funs,
            cxt,
        )
        .await?;
        let pwd = IamCertServ::get_new_pwd();
        IamRelServ::add_rel(
            IAMRelKind::IamAccountTenant,
            &account_id,
            &iam_constants::get_rbum_basic_info().role_tenant_admin_id,
            None,
            None,
            funs,
            cxt,
        )
        .await?;
        IamCertUserPwdServ::add_cert_conf(
            &mut IamUserPwdCertConfAddOrModifyReq {
                ak_note: None,
                ak_rule: None,
                sk_note: None,
                sk_rule: None,
                repeatable: Some(true),
                expire_sec: None,
            },
            Some(tenant_id.to_string()),
            funs,
            cxt,
        )
        .await?;

        IamCertMailVCodeServ::add_cert_conf(
            &mut IamMailVCodeCertConfAddOrModifyReq { ak_note: None, ak_rule: None },
            Some(tenant_id.to_string()),
            funs,
            cxt,
        )
        .await?;

        IamCertPhoneVCodeServ::add_cert_conf(
            &mut IamPhoneVCodeCertConfAddOrModifyReq { ak_note: None, ak_rule: None },
            Some(tenant_id.to_string()),
            funs,
            cxt,
        )
        .await?;

        IamCertTokenServ::add_cert_conf(
            &mut IamTokenCertConfAddReq {
                name: TrimString(IamCertTokenKind::TokenDefault.to_string()),
                coexist_num: iam_constants::RBUM_CERT_CONF_TOKEN_DEFAULT_COEXIST_NUM,
                expire_sec: Some(iam_constants::RBUM_CERT_CONF_TOKEN_EXPIRE_SEC),
            },
            IamCertTokenKind::TokenDefault,
            Some(tenant_id.to_string()),
            funs,
            cxt,
        )
        .await?;

        IamCertTokenServ::add_cert_conf(
            &mut IamTokenCertConfAddReq {
                name: TrimString(IamCertTokenKind::TokenPc.to_string()),
                coexist_num: 1,
                expire_sec: Some(iam_constants::RBUM_CERT_CONF_TOKEN_EXPIRE_SEC),
            },
            IamCertTokenKind::TokenPc,
            Some(tenant_id.to_string()),
            funs,
            cxt,
        )
        .await?;

        IamCertTokenServ::add_cert_conf(
            &mut IamTokenCertConfAddReq {
                name: TrimString(IamCertTokenKind::TokenPhone.to_string()),
                coexist_num: 1,
                expire_sec: Some(iam_constants::RBUM_CERT_CONF_TOKEN_EXPIRE_SEC),
            },
            IamCertTokenKind::TokenPhone,
            Some(tenant_id.to_string()),
            funs,
            cxt,
        )
        .await?;

        IamCertTokenServ::add_cert_conf(
            &mut IamTokenCertConfAddReq {
                name: TrimString(IamCertTokenKind::TokenPad.to_string()),
                coexist_num: 1,
                expire_sec: Some(iam_constants::RBUM_CERT_CONF_TOKEN_EXPIRE_SEC),
            },
            IamCertTokenKind::TokenPad,
            Some(tenant_id.to_string()),
            funs,
            cxt,
        )
        .await?;

        IamCertUserPwdServ::add_cert(
            &mut IamUserPwdCertAddReq {
                ak: TrimString(add_req.admin_username.0.to_string()),
                sk: TrimString(pwd.to_string()),
            },
            &account_id,
            Some(&tenant_id),
            funs,
            cxt,
        )
        .await?;
        Ok((tenant_id, pwd))
    }

    pub async fn modify_tenant(id: &str, modify_req: &mut IamCsTenantModifyReq, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<()> {
        IamRoleServ::need_sys_admin(funs, cxt).await?;
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
            funs,
            cxt,
        )
        .await
    }

    pub async fn get_tenant(id: &str, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<IamTenantDetailResp> {
        IamRoleServ::need_sys_admin(funs, cxt).await?;
        IamTenantServ::get_item(id, &IamTenantFilterReq::default(), funs, cxt).await
    }

    pub async fn paginate_tenants(
        filter: &IamTenantFilterReq,
        page_number: u64,
        page_size: u64,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<TardisPage<IamTenantSummaryResp>> {
        IamRoleServ::need_sys_admin(funs, cxt).await?;
        IamTenantServ::paginate_items(filter, page_number, page_size, desc_sort_by_create, desc_sort_by_update, funs, cxt).await
    }

    pub async fn delete_tenant(id: &str, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<u64> {
        IamRoleServ::need_sys_admin(funs, cxt).await?;
        IamTenantServ::delete_item(id, funs, cxt).await
    }
}
