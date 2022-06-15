use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::TardisFunsInst;

use bios_basic::rbum::helper::rbum_scope_helper::get_max_level_id_by_context;
use bios_basic::rbum::serv::rbum_cert_serv::RbumCertServ;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;

use crate::basic::dto::iam_account_dto::IamAccountInfoResp;
use crate::basic::dto::iam_cert_dto::{IamPwdNewReq, IamUserPwdCertModifyReq};
use crate::basic::serv::iam_cert_serv::IamCertServ;
use crate::basic::serv::iam_cert_user_pwd_serv::IamCertUserPwdServ;
use crate::basic::serv::iam_tenant_serv::IamTenantServ;
use crate::console_passport::dto::iam_cp_cert_dto::IamCpUserPwdLoginReq;
use crate::iam_enumeration::IamCertKind;

pub struct IamCpCertUserPwdServ;

impl<'a> IamCpCertUserPwdServ {
    pub async fn new_pwd_without_login(pwd_new_req: &IamPwdNewReq, funs: &TardisFunsInst<'a>) -> TardisResult<()> {
        let tenant_id = Self::get_tenant_id(pwd_new_req.tenant_id.clone(), funs).await?;
        let rbum_cert_conf_id = IamCertServ::get_cert_conf_id_by_code(&IamCertKind::UserPwd.to_string(), Some(tenant_id.clone()), funs).await?;
        let (_, _, rbum_item_id) = RbumCertServ::validate(&pwd_new_req.ak.0, &pwd_new_req.original_sk.0, &rbum_cert_conf_id, true, &tenant_id, funs).await?;
        let ctx = TardisContext {
            own_paths: tenant_id.clone(),
            ak: pwd_new_req.ak.to_string(),
            owner: rbum_item_id.to_string(),
            roles: vec![],
            groups: vec![],
        };
        IamCertUserPwdServ::modify_cert(
            &IamUserPwdCertModifyReq {
                original_sk: pwd_new_req.original_sk.clone(),
                new_sk: pwd_new_req.new_sk.clone(),
            },
            &rbum_item_id,
            &rbum_cert_conf_id,
            funs,
            &ctx,
        )
        .await
    }

    pub async fn modify_cert_user_pwd(id: &str, modify_req: &IamUserPwdCertModifyReq, funs: &TardisFunsInst<'a>, ctx: &TardisContext) -> TardisResult<()> {
        let rbum_cert_conf_id = IamCertServ::get_cert_conf_id_by_code(IamCertKind::UserPwd.to_string().as_str(), get_max_level_id_by_context(ctx), funs).await?;
        IamCertUserPwdServ::modify_cert(modify_req, id, &rbum_cert_conf_id, funs, ctx).await
    }

    pub async fn login_by_user_pwd(login_req: &IamCpUserPwdLoginReq, funs: &TardisFunsInst<'a>) -> TardisResult<IamAccountInfoResp> {
        let tenant_id = Self::get_tenant_id(login_req.tenant_id.clone(), funs).await?;
        let rbum_cert_conf_id = IamCertServ::get_cert_conf_id_by_code(&IamCertKind::UserPwd.to_string(), Some(tenant_id.clone()), funs).await?;
        let (_, _, rbum_item_id) = RbumCertServ::validate(&login_req.ak.0, &login_req.sk.0, &rbum_cert_conf_id, false, &tenant_id, funs).await?;
        let resp = IamCertServ::package_tardis_context_and_resp(login_req.tenant_id.clone(), &login_req.ak.0, &rbum_item_id, login_req.flag.clone(), funs).await?;
        Ok(resp)
    }

    async fn get_tenant_id(tenant_id: Option<String>, funs: &TardisFunsInst<'a>) -> TardisResult<String> {
        let tenant_id = if let Some(tenant_id) = &tenant_id {
            if IamTenantServ::is_disabled(tenant_id, funs).await? {
                return Err(funs.err().conflict("cert_user_pwd", "login", &format!("tenant {} is disabled", tenant_id)));
            }
            tenant_id
        } else {
            ""
        };
        Ok(tenant_id.to_string())
    }
}
