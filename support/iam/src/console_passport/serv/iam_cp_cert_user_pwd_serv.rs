use bios_basic::rbum::dto::rbum_filer_dto::RbumCertFilterReq;
use bios_basic::rbum::rbum_enumeration::RbumCertRelKind;
use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::TardisFunsInst;

use bios_basic::rbum::helper::rbum_scope_helper::get_max_level_id_by_context;
use bios_basic::rbum::serv::rbum_cert_serv::RbumCertServ;
use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;

use crate::basic::dto::iam_account_dto::{IamAccountInfoResp, IamAccountModifyReq};
use crate::basic::dto::iam_cert_dto::{IamCertPwdNewReq, IamCertUserNameNewReq, IamCertUserPwdModifyReq};
use crate::basic::serv::clients::spi_log_client::{LogParamTag, SpiLogClient};
use crate::basic::serv::iam_account_serv::IamAccountServ;
use crate::basic::serv::iam_cert_ldap_serv::IamCertLdapServ;
use crate::basic::serv::iam_cert_serv::IamCertServ;
use crate::basic::serv::iam_cert_user_pwd_serv::IamCertUserPwdServ;
use crate::basic::serv::iam_key_cache_serv::IamIdentCacheServ;
use crate::basic::serv::iam_tenant_serv::IamTenantServ;
use crate::console_passport::dto::iam_cp_cert_dto::IamCpUserPwdLoginReq;
use crate::iam_enumeration::{IamAccountStatusKind, IamCertKernelKind};

pub struct IamCpCertUserPwdServ;

impl IamCpCertUserPwdServ {
    pub async fn new_pwd_without_login(pwd_new_req: &IamCertPwdNewReq, funs: &TardisFunsInst) -> TardisResult<()> {
        let mut tenant_id = Self::get_tenant_id(pwd_new_req.tenant_id.clone(), funs).await?;
        let mut rbum_cert_conf_id = IamCertServ::get_cert_conf_id_by_kind(&IamCertKernelKind::UserPwd.to_string(), Some(tenant_id.clone()), funs).await?;
        let validate_resp = IamCertServ::validate_by_ak_and_sk(
            &pwd_new_req.ak.0,
            &pwd_new_req.original_sk.0,
            None,
            Some(&RbumCertRelKind::Item),
            true,
            Some(tenant_id.clone()),
            Some(vec![
                &IamCertKernelKind::UserPwd.to_string(),
                &IamCertKernelKind::MailVCode.to_string(),
                &IamCertKernelKind::PhoneVCode.to_string(),
            ]),
            funs,
        )
        .await;
        let (_, _, rbum_item_id) = if let Ok(validate_resp) = validate_resp {
            validate_resp
        } else {
            if let Some(e) = validate_resp.clone().err() {
                // throw out Err when sk is expired and cert is locked
                if e.code == "409-iam-cert-valid" || e.code == "401-iam-cert-valid_lock" {
                    validate_resp?;
                }
            };
            tenant_id = "".to_string();
            rbum_cert_conf_id = IamCertServ::get_cert_conf_id_by_kind(&IamCertKernelKind::UserPwd.to_string(), Some(tenant_id.clone()), funs).await?;
            IamCertServ::validate_by_ak_and_sk(
                &pwd_new_req.ak.0,
                &pwd_new_req.original_sk.0,
                None,
                Some(&RbumCertRelKind::Item),
                true,
                Some("".to_string()),
                Some(vec![
                    &IamCertKernelKind::UserPwd.to_string(),
                    &IamCertKernelKind::MailVCode.to_string(),
                    &IamCertKernelKind::PhoneVCode.to_string(),
                ]),
                funs,
            )
            .await?
        };
        let ctx = TardisContext {
            own_paths: tenant_id.clone(),
            ak: pwd_new_req.ak.to_string(),
            owner: rbum_item_id.to_string(),
            roles: vec![],
            groups: vec![],
            ..Default::default()
        };
        IamAccountServ::modify_item(
            &rbum_item_id,
            &mut IamAccountModifyReq {
                name: None,
                scope_level: None,
                disabled: None,
                status: Some(IamAccountStatusKind::Active),
                is_auto: Some(false),
                icon: None,
                lock_status: None,
            },
            funs,
            &ctx,
        )
        .await?;
        IamCertUserPwdServ::modify_cert(
            &IamCertUserPwdModifyReq {
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

    pub async fn new_user_name(req: &IamCertUserNameNewReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let tenant_id = if IamAccountServ::is_global_account(ctx.owner.as_ref(), funs, ctx).await? {
            Some("".to_string())
        } else {
            Some(ctx.own_paths.clone())
        };
        let ctx = IamAccountServ::new_context_if_account_is_global(ctx, funs).await?;
        let rbum_cert_conf_id = IamCertServ::get_cert_conf_id_by_kind(&IamCertKernelKind::UserPwd.to_string(), tenant_id.clone(), funs).await?;
        let _ = IamCertServ::validate_by_ak_and_sk(
            &req.original_ak.0,
            &req.sk.0,
            None,
            Some(&RbumCertRelKind::Item),
            false,
            tenant_id,
            Some(vec![
                &IamCertKernelKind::UserPwd.to_string(),
                &IamCertKernelKind::MailVCode.to_string(),
                &IamCertKernelKind::PhoneVCode.to_string(),
            ]),
            funs,
        )
        .await?;
        IamCertUserPwdServ::modify_ak_cert(req, &rbum_cert_conf_id, funs, &ctx).await?;

        let id = ctx.owner.to_string();
        let op_describe = format!("修改用户名为{}", req.new_ak.as_ref());
        let _ = SpiLogClient::add_ctx_task(LogParamTag::IamAccount, Some(id), op_describe, Some("ModifyUserName".to_string()), &ctx).await;

        Ok(())
    }

    pub async fn modify_cert_user_pwd(id: &str, modify_req: &IamCertUserPwdModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let actual_ctx = if IamAccountServ::is_global_account(id, funs, ctx).await? {
            TardisContext {
                own_paths: "".to_string(),
                ..ctx.clone()
            }
        } else {
            ctx.clone()
        };
        let rbum_cert_conf_id = IamCertServ::get_cert_conf_id_by_kind(IamCertKernelKind::UserPwd.to_string().as_str(), get_max_level_id_by_context(&actual_ctx), funs).await?;
        IamCertUserPwdServ::modify_cert(modify_req, id, &rbum_cert_conf_id, funs, &actual_ctx).await
    }

    pub async fn generic_sk_validate(sk: &str, supplier: Option<String>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        if let Some(supplier) = supplier {
            IamCertLdapServ::validate_by_ldap(sk, &supplier, funs, ctx).await?;
        } else {
            Self::validate_by_user_pwd(sk, false, funs, ctx).await?;
        }
        IamIdentCacheServ::add_double_auth(&ctx.owner, funs).await?;
        Ok(())
    }

    pub async fn validate_by_user_pwd(sk: &str, ignore_end_time: bool, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let rbum_cert_conf_id = IamCertServ::get_cert_conf_id_by_kind(IamCertKernelKind::UserPwd.to_string().as_str(), get_max_level_id_by_context(ctx), funs).await?;
        let user_pwd_cert = IamCertServ::get_kernel_cert(&ctx.owner, &IamCertKernelKind::UserPwd, funs, ctx).await?;

        let (_, _, _) = IamCertServ::validate_by_ak_and_sk(
            &user_pwd_cert.ak,
            sk,
            Some(&rbum_cert_conf_id),
            None,
            ignore_end_time,
            Some(ctx.own_paths.clone()),
            None,
            funs,
        )
        .await?;
        Ok(())
    }

    pub async fn login_by_user_pwd(login_req: &IamCpUserPwdLoginReq, funs: &TardisFunsInst) -> TardisResult<IamAccountInfoResp> {
        let tenant_id = Self::get_tenant_id(login_req.tenant_id.clone(), funs).await?;
        let validate_resp = IamCertServ::validate_by_ak_and_sk(
            &login_req.ak.0,
            &login_req.sk.0,
            None,
            Some(&RbumCertRelKind::Item),
            false,
            Some(tenant_id),
            Some(vec![
                &IamCertKernelKind::UserPwd.to_string(),
                &IamCertKernelKind::MailVCode.to_string(),
                &IamCertKernelKind::PhoneVCode.to_string(),
            ]),
            funs,
        )
        .await;
        let (_, _, rbum_item_id) = if let Ok(validate_resp) = validate_resp {
            validate_resp
        } else {
            if let Some(e) = validate_resp.clone().err() {
                // throw out Err when sk is expired and cert is locked
                if e.code == "409-iam-cert-valid" || e.code == "401-iam-cert-valid_lock" {
                    validate_resp?;
                }
            };
            IamCertServ::validate_by_ak_and_sk(
                &login_req.ak.0,
                &login_req.sk.0,
                None,
                Some(&RbumCertRelKind::Item),
                false,
                Some("".to_string()),
                Some(vec![
                    &IamCertKernelKind::UserPwd.to_string(),
                    &IamCertKernelKind::MailVCode.to_string(),
                    &IamCertKernelKind::PhoneVCode.to_string(),
                ]),
                funs,
            )
            .await?
        };
        let resp = IamCertServ::package_tardis_context_and_resp(login_req.tenant_id.clone(), &rbum_item_id, login_req.flag.clone(), None, funs).await?;
        Ok(resp)
    }

    pub async fn get_tenant_id(tenant_id: Option<String>, funs: &TardisFunsInst) -> TardisResult<String> {
        let tenant_id = if let Some(tenant_id) = &tenant_id {
            if IamTenantServ::is_disabled(tenant_id, funs).await? {
                return Err(funs.err().conflict("iam_cert_user_pwd", "login", &format!("tenant {tenant_id} is disabled"), "409-iam-tenant-is-disabled"));
            }
            tenant_id
        } else {
            ""
        };
        Ok(tenant_id.to_string())
    }

    pub async fn get_cert_rel_account_by_user_name(user_name: &str, rel_rbum_cert_conf_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Option<String>> {
        let result = RbumCertServ::find_rbums(
            &RbumCertFilterReq {
                rel_rbum_cert_conf_ids: Some(vec![rel_rbum_cert_conf_id.to_string()]),
                ak: Some(user_name.to_string()),
                ..Default::default()
            },
            None,
            None,
            funs,
            ctx,
        )
        .await?
        .first()
        .map(|r| r.rel_rbum_id.to_string());
        Ok(result)
    }
}
