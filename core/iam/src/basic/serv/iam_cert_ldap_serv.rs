use ldap3::log::warn;
use std::collections::HashMap;

use self::ldap::LdapClient;
use crate::basic::dto::iam_account_dto::{IamAccountAddByLdapResp, IamAccountExtSysAddReq, IamAccountExtSysBatchAddReq};
use crate::basic::dto::iam_filer_dto::IamAccountFilterReq;
use crate::console_passport::dto::iam_cp_cert_dto::IamCpUserPwdBindWithLdapReq;
use crate::console_passport::serv::iam_cp_cert_user_pwd_serv::IamCpCertUserPwdServ;
use crate::iam_enumeration::IamCertKernelKind;
use crate::{
    basic::dto::{
        iam_account_dto::{IamAccountAggAddReq, IamAccountExtSysResp},
        iam_cert_conf_dto::{IamCertConfLdapAddOrModifyReq, IamCertConfLdapResp},
        iam_cert_dto::IamCertLdapAddOrModifyReq,
        iam_filer_dto::IamTenantFilterReq,
    },
    iam_config::IamBasicConfigApi,
    iam_enumeration::IamCertExtKind,
};
use bios_basic::rbum::rbum_enumeration::RbumScopeLevelKind;
use bios_basic::rbum::{
    dto::{
        rbum_cert_conf_dto::{RbumCertConfAddReq, RbumCertConfModifyReq},
        rbum_cert_dto::{RbumCertAddReq, RbumCertModifyReq},
        rbum_filer_dto::{RbumCertConfFilterReq, RbumCertFilterReq},
    },
    rbum_enumeration::{RbumCertRelKind, RbumCertStatusKind},
    serv::{
        rbum_cert_serv::{RbumCertConfServ, RbumCertServ},
        rbum_crud_serv::RbumCrudOperation,
        rbum_item_serv::RbumItemCrudOperation,
    },
};
use serde::{Deserialize, Serialize};
use tardis::{
    basic::{dto::TardisContext, field::TrimString, result::TardisResult},
    TardisFuns, TardisFunsInst,
};

use super::{iam_account_serv::IamAccountServ, iam_cert_serv::IamCertServ, iam_tenant_serv::IamTenantServ};

pub struct IamCertLdapServ;

impl IamCertLdapServ {
    pub async fn add_cert_conf(add_req: &IamCertConfLdapAddOrModifyReq, rel_iam_item_id: Option<String>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<String> {
        RbumCertConfServ::add_rbum(
            &mut RbumCertConfAddReq {
                code: TrimString(format!("{}{}", IamCertExtKind::Ldap, add_req.code.clone())),
                name: TrimString(add_req.name.clone()),
                note: None,
                ak_note: None,
                ak_rule: None,
                sk_note: None,
                sk_rule: None,
                ext: Some(TardisFuns::json.obj_to_string(&IamCertLdapServerAuthInfo {
                    is_tls: add_req.is_tls,
                    principal: add_req.principal.to_string(),
                    credentials: add_req.credentials.to_string(),
                    base_dn: add_req.base_dn.to_string(),
                    search_base_filter: add_req.search_base_filter.to_string(),
                    field_display_name: add_req.field_display_name.to_string(),
                })?),
                sk_need: Some(false),
                sk_dynamic: Some(false),
                sk_encrypted: Some(false),
                repeatable: None,
                is_basic: Some(false),
                is_ak_repeatable: None,
                rest_by_kinds: None,
                expire_sec: None,
                sk_lock_cycle_sec: None,
                sk_lock_err_times: None,
                sk_lock_duration_sec: None,
                coexist_num: Some(1),
                conn_uri: Some(add_req.conn_uri.clone()),
                rel_rbum_domain_id: funs.iam_basic_domain_iam_id(),
                rel_rbum_item_id: rel_iam_item_id.clone(),
            },
            funs,
            ctx,
        )
        .await
    }

    pub async fn modify_cert_conf(id: &str, modify_req: &IamCertConfLdapAddOrModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        RbumCertConfServ::modify_rbum(
            id,
            &mut RbumCertConfModifyReq {
                name: None,
                note: None,
                ak_note: None,
                ak_rule: None,
                sk_note: None,
                sk_rule: None,
                ext: Some(TardisFuns::json.obj_to_string(&IamCertLdapServerAuthInfo {
                    is_tls: modify_req.is_tls,
                    principal: modify_req.principal.to_string(),
                    credentials: modify_req.credentials.to_string(),
                    base_dn: modify_req.base_dn.to_string(),
                    search_base_filter: modify_req.search_base_filter.to_string(),
                    field_display_name: modify_req.field_display_name.to_string(),
                })?),
                sk_need: None,
                sk_encrypted: None,
                repeatable: None,
                is_basic: None,
                rest_by_kinds: None,
                expire_sec: None,
                sk_lock_cycle_sec: None,
                sk_lock_err_times: None,
                sk_lock_duration_sec: None,
                coexist_num: None,
                conn_uri: Some(modify_req.conn_uri.clone()),
            },
            funs,
            ctx,
        )
        .await
    }

    pub async fn get_cert_conf(id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<IamCertConfLdapResp> {
        RbumCertConfServ::get_rbum(id, &RbumCertConfFilterReq::default(), funs, ctx).await.map(|resp| {
            TardisFuns::json
                .str_to_obj::<IamCertLdapServerAuthInfo>(&resp.ext)
                .map(|info| IamCertConfLdapResp {
                    conn_uri: resp.conn_uri,
                    is_tls: info.is_tls,
                    principal: info.principal,
                    credentials: info.credentials,
                    base_dn: info.base_dn,
                    search_base_filter: info.search_base_filter,
                    field_display_name: info.field_display_name,
                })
                .unwrap()
        })
    }

    pub async fn add_or_modify_cert(
        add_or_modify_req: &IamCertLdapAddOrModifyReq,
        account_id: &str,
        rel_rbum_cert_conf_id: &str,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<()> {
        let cert_id = RbumCertServ::find_id_rbums(
            &RbumCertFilterReq {
                rel_rbum_cert_conf_ids: Some(vec![rel_rbum_cert_conf_id.to_string()]),
                rel_rbum_id: Some(account_id.to_string()),
                ..Default::default()
            },
            None,
            None,
            funs,
            ctx,
        )
        .await?;
        if let Some(cert_id) = cert_id.first() {
            RbumCertServ::modify_rbum(
                cert_id,
                &mut RbumCertModifyReq {
                    ak: Some(add_or_modify_req.dn.clone()),
                    sk: None,
                    ext: None,
                    start_time: None,
                    end_time: None,
                    conn_uri: None,
                    status: None,
                },
                funs,
                ctx,
            )
            .await?;
        } else {
            RbumCertServ::add_rbum(
                &mut RbumCertAddReq {
                    ak: add_or_modify_req.dn.clone(),
                    sk: None,
                    vcode: None,
                    ext: None,
                    start_time: None,
                    end_time: None,
                    conn_uri: None,
                    status: RbumCertStatusKind::Enabled,
                    rel_rbum_cert_conf_id: Some(rel_rbum_cert_conf_id.to_string()),
                    rel_rbum_kind: RbumCertRelKind::Item,
                    rel_rbum_id: account_id.to_string(),
                    is_outside: false,
                },
                funs,
                ctx,
            )
            .await?;
        };
        Ok(())
    }

    pub async fn get_cert_rel_account_by_dn(dn: &str, rel_rbum_cert_conf_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Option<String>> {
        let result = RbumCertServ::find_rbums(
            &RbumCertFilterReq {
                rel_rbum_cert_conf_ids: Some(vec![rel_rbum_cert_conf_id.to_string()]),
                ak: Some(dn.to_string()),
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

    pub async fn batch_get_or_add_account_without_verify(
        add_req: IamAccountExtSysBatchAddReq,
        tenant_id: Option<String>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<IamAccountAddByLdapResp> {
        if add_req.account_id.is_empty() {
            Ok(IamAccountAddByLdapResp {
                result: vec![],
                fail: HashMap::new(),
            })
        } else {
            let mut result: Vec<String> = Vec::new();
            let mut fail: HashMap<String, String> = HashMap::new();
            for account_id in add_req.account_id {
                let verify = Self::get_or_add_account_without_verify(
                    IamAccountExtSysAddReq {
                        account_id: account_id.clone(),
                        code: add_req.code.clone(),
                    },
                    tenant_id.clone(),
                    funs,
                    ctx,
                )
                .await;
                if verify.is_ok() {
                    result.push(verify.unwrap().0);
                } else {
                    let err_msg = if let Err(tardis_error) = verify { tardis_error.message } else { "".to_string() };
                    warn!("get_or_add_account_without_verify resp is err:{}", err_msg);
                    fail.insert(account_id, err_msg);
                }
            }
            Ok(IamAccountAddByLdapResp { result, fail })
        }
    }

    pub async fn get_or_add_account_without_verify(
        add_req: IamAccountExtSysAddReq,
        tenant_id: Option<String>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<(String, String)> {
        let dn = &add_req.account_id;
        let cert_conf_id = IamCertServ::get_cert_conf_id_by_code(&format!("{}{}", IamCertExtKind::Ldap, add_req.code.clone()), tenant_id, funs).await?;
        let cert_conf = Self::get_cert_conf(&cert_conf_id, funs, ctx).await?;
        if let Some(account_id) = Self::get_cert_rel_account_by_dn(dn, &cert_conf_id, funs, ctx).await? {
            return Ok((account_id, dn.to_string()));
        }
        let mut ldap_client = LdapClient::new(&cert_conf.conn_uri, cert_conf.is_tls, &cert_conf.base_dn).await?;
        if ldap_client.bind(&cert_conf.principal, &cert_conf.credentials).await?.is_none() {
            ldap_client.unbind().await?;
            return Err(funs.err().unauthorized("rbum_cert", "search_accounts", "ldap admin validation error", "401-rbum-cert-valid-error"));
        };
        let account = ldap_client.get_by_dn(dn, &vec!["dn", "cn", &cert_conf.field_display_name]).await?;
        ldap_client.unbind().await?;
        if let Some(account) = account {
            let mock_ctx = TardisContext {
                own_paths: ctx.own_paths.clone(),
                owner: TardisFuns::field.nanoid(),
                ..Default::default()
            };
            let account_id = Self::do_add_account(
                &account.dn,
                &account.get_simple_attr(&cert_conf.field_display_name).unwrap_or_else(|| "".to_string()),
                &cert_conf_id,
                funs,
                &mock_ctx,
            )
            .await?;
            Ok((account_id, dn.to_string()))
        } else {
            return Err(funs.err().not_found(
                "rbum_cert",
                "get_or_add_account_without_verify",
                &format!("not found ldap cert(openid): {}", &dn),
                "401-rbum-cert-valid-error",
            ));
        }
    }

    pub async fn get_account_with_verify(user_name: &str, password: &str, tenant_id: Option<String>, code: &str, funs: &TardisFunsInst) -> TardisResult<Option<(String, String)>> {
        let mock_ctx = Self::generate_default_mock_ctx(tenant_id.clone()).await;
        let (mut ldap_client, _, cert_conf_id) = Self::get_ldap_client(None, code, funs, &mock_ctx).await?;
        let dn = if let Some(dn) = ldap_client.bind(user_name, password).await? {
            dn
        } else {
            ldap_client.unbind().await?;
            return Err(funs.err().unauthorized("rbum_cert", "get_or_add_account", "validation error", "401-rbum-cert-valid-error"));
        };
        ldap_client.unbind().await?;
        if let Some(account_id) = Self::get_cert_rel_account_by_dn(&dn, &cert_conf_id, funs, &mock_ctx).await? {
            Ok(Some((account_id, dn)))
        } else {
            Ok(None)
        }
    }

    pub async fn search_accounts(
        user_or_display_name: &str,
        tenant_id: Option<String>,
        code: &str,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<Vec<IamAccountExtSysResp>> {
        let (mut ldap_client, cert_conf, _) = Self::get_ldap_client(tenant_id, code, funs, ctx).await?;
        if ldap_client.bind(&cert_conf.principal, &cert_conf.credentials).await?.is_none() {
            ldap_client.unbind().await?;
            return Err(funs.err().unauthorized("rbum_cert", "search_accounts", "ldap admin validation error", "401-rbum-cert-valid-error"));
        };
        let accounts = ldap_client
            .search(
                &cert_conf.package_fitler_by_search_account(user_or_display_name),
                &vec!["dn", "cn", &cert_conf.field_display_name],
            )
            .await?
            .into_iter()
            .map(|r| IamAccountExtSysResp {
                user_name: r.get_simple_attr("cn").unwrap_or_else(|| "".to_string()),
                display_name: r.get_simple_attr(&cert_conf.field_display_name).unwrap_or_else(|| "".to_string()),
                account_id: r.dn,
            })
            .collect();
        ldap_client.unbind().await?;
        Ok(accounts)
    }

    pub async fn check_user_pwd_is_bind(ak: &str, code: &str, tenant_id: Option<String>, funs: &TardisFunsInst) -> TardisResult<bool> {
        let mut tenant_id = tenant_id.clone();
        if tenant_id.is_some() && tenant_id.clone().unwrap().is_empty() {
            tenant_id == None;
        }
        if tenant_id.is_some() && IamTenantServ::is_disabled(&tenant_id.clone().unwrap(), funs).await? {
            return Err(funs.err().conflict(
                "user_pwd",
                "check_bind",
                &format!("tenant {} is disabled", tenant_id.unwrap()),
                "409-iam-tenant-is-disabled",
            ));
        }
        let userpwd_cert_conf_id = IamCertServ::get_cert_conf_id_by_code(&IamCertKernelKind::UserPwd.to_string(), tenant_id.clone(), funs).await?;
        let ldap_cert_conf_id_result = IamCertServ::get_cert_conf_id_by_code(&format!("{}{}", IamCertExtKind::Ldap, code), tenant_id.clone(), funs).await;
        if ldap_cert_conf_id_result.is_err() {
            return Ok(false);
        }
        let ldap_cert_conf_id = ldap_cert_conf_id_result?;
        let exist = RbumCertServ::check_exist(ak, &userpwd_cert_conf_id, None, funs).await?;
        if exist {
            let mock_ctx = Self::generate_default_mock_ctx(tenant_id.clone()).await;
            if let Some(account_id) = IamCpCertUserPwdServ::get_cert_rel_account_by_user_name(ak, &userpwd_cert_conf_id, funs, &mock_ctx).await? {
                let cert_id = Self::get_ldap_cert_account_by_account(&account_id, &ldap_cert_conf_id, funs, &mock_ctx).await?;
                if cert_id.is_some() {
                    Ok(true)
                } else {
                    Ok(false)
                }
            } else {
                // Unreachable code
                Ok(false)
            }
        } else {
            Err(funs.err().not_found("user_pwd", "check_bind", "not found cert record", "404-rbum-*-obj-not-exist"))
        }
    }

    pub async fn bind_or_create_user_pwd_by_ldap(login_req: &IamCpUserPwdBindWithLdapReq, funs: &TardisFunsInst) -> TardisResult<(String, String)> {
        let tenant_id = login_req.tenant_id.clone();
        let mut mock_ctx = Self::generate_default_mock_ctx(tenant_id.clone()).await;

        let (mut ldap_client, cert_conf, cert_conf_id) = Self::get_ldap_client(None, login_req.ldap_login.code.to_string().as_str(), funs, &mock_ctx).await?;
        let dn = if let Some(dn) = ldap_client.bind(login_req.ldap_login.name.to_string().as_str(), login_req.ldap_login.password.as_str()).await? {
            dn
        } else {
            ldap_client.unbind().await?;
            return Err(funs.err().unauthorized("rbum_cert", "get_or_add_account", "validation error", "401-rbum-cert-valid-error"));
        };

        let account = ldap_client.get_by_dn(&dn, &vec!["dn", "cn", &cert_conf.field_display_name]).await?;
        ldap_client.unbind().await?;
        if let Some(account) = account {
            mock_ctx.owner = TardisFuns::field.nanoid();
            let account_id = if let Some(ak) = login_req.bind_user_pwd.ak.clone() {
                // bind user_pwd with ldap cert
                Self::bind_user_pwd_by_ldap(
                    &dn,
                    ak.as_ref(),
                    login_req.bind_user_pwd.sk.as_ref(),
                    &cert_conf_id,
                    tenant_id.clone(),
                    &login_req.ldap_login.code,
                    funs,
                    &mock_ctx,
                )
                .await?
            } else {
                // create user_pwd and bind user_pwd with ldap cert
                Self::create_user_pwd_by_ldap(
                    &dn,
                    &account.get_simple_attr(&cert_conf.field_display_name).unwrap_or_else(|| "".to_string()),
                    login_req.bind_user_pwd.sk.as_ref(),
                    &cert_conf_id,
                    None,
                    funs,
                    &mock_ctx,
                )
                .await?
            };
            Ok((account_id, dn))
        } else {
            return Err(funs.err().not_found(
                "rbum_cert",
                "bind_or_create_user_pwd_by_ldap",
                &format!("not found ldap cert(openid): {}", &dn),
                "401-rbum-cert-valid-error",
            ));
        }
    }

    pub async fn create_user_pwd_by_ldap(
        dn: &str,
        account_name: &str,
        password: &str,
        cert_conf_id: &str,
        tenant_id: Option<String>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<String> {
        if tenant_id.is_some() && !IamTenantServ::get_item(&tenant_id.unwrap(), &IamTenantFilterReq::default(), funs, ctx).await?.account_self_reg {
            return Err(funs.err().not_found(
                "rbum_cert",
                "create_user_pwd_by_ldap",
                &format!("not found ldap cert(openid): {} and self-registration disabled", &dn),
                "401-rbum-cert-valid-error",
            ));
        }

        let account_id = IamAccountServ::add_account_agg(
            &IamAccountAggAddReq {
                id: Some(TrimString(ctx.owner.clone())),
                name: TrimString(account_name.to_string()),
                cert_user_name: TrimString(TardisFuns::field.nanoid_len(8).to_lowercase()),
                cert_password: password.into(),
                cert_phone: None,
                cert_mail: None,
                role_ids: None,
                org_node_ids: None,
                scope_level: Some(RbumScopeLevelKind::L1),
                disabled: None,
                icon: None,
                exts: HashMap::new(),
                status: Some(RbumCertStatusKind::Pending),
            },
            funs,
            ctx,
        )
        .await?;
        Self::add_or_modify_cert(&IamCertLdapAddOrModifyReq { dn: TrimString(dn.to_string()) }, &account_id, cert_conf_id, funs, ctx).await?;
        Ok(account_id)
    }

    pub async fn bind_user_pwd_by_ldap(
        dn: &str,
        user_name: &str,
        password: &str,
        cert_conf_id: &str,
        tenant_id: Option<String>,
        code: &str,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<String> {
        //验证用户名密码登录
        let (_, _, rbum_item_id) = if let Some(tenant_id) = tenant_id.clone() {
            let global_check = RbumCertServ::validate_by_ak_and_basic_sk(user_name, password, &RbumCertRelKind::Item, false, "", funs).await;
            if global_check.is_err() {
                let tenant_check = RbumCertServ::validate_by_ak_and_basic_sk(user_name, password, &RbumCertRelKind::Item, false, &tenant_id, funs).await;
                if tenant_check.is_ok() {
                    return Err(funs.err().conflict("rbum_cert", "bind_user_pwd_by_ldap", "user is private", "409-user-is-private"));
                } else {
                    return Err(funs.err().unauthorized("rbum_cert", "valid", "validation error", "401-rbum-cert-valid-error"));
                }
            } else {
                global_check?
            }
        } else {
            RbumCertServ::validate_by_ak_and_basic_sk(user_name, password, &RbumCertRelKind::Item, false, "", funs).await?
        };
        if let true = Self::check_user_pwd_is_bind(user_name, code, tenant_id.clone(), funs).await? {
            return Err(funs.err().not_found("rbum_cert", "bind_user_pwd_by_ldap", "user is bound by ldap", "409-iam-user-is-bound"));
        }
        //查出用户名密码的account_id
        Self::add_or_modify_cert(&IamCertLdapAddOrModifyReq { dn: TrimString(dn.to_string()) }, &rbum_item_id, cert_conf_id, funs, ctx).await?;
        Ok(rbum_item_id)
    }

    pub async fn generate_default_mock_ctx(_tenant_id: Option<String>) -> TardisContext {
        TardisContext { ..Default::default() }
    }

    async fn do_add_account(dn: &str, name: &str, cert_conf_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<String> {
        let account_id = IamAccountServ::add_account_agg(
            &IamAccountAggAddReq {
                id: Some(TrimString(ctx.owner.clone())),
                name: TrimString(name.to_string()),
                // TODO Auto match rule
                cert_user_name: TrimString(TardisFuns::field.nanoid_len(8).to_lowercase()),
                cert_password: TrimString(format!("{}0Pw$", TardisFuns::field.nanoid_len(6))),
                cert_phone: None,
                cert_mail: None,
                role_ids: None,
                org_node_ids: None,
                scope_level: Some(RbumScopeLevelKind::L1),
                disabled: None,
                icon: None,
                exts: HashMap::new(),
                status: Some(RbumCertStatusKind::Pending),
            },
            funs,
            ctx,
        )
        .await?;
        Self::add_or_modify_cert(&IamCertLdapAddOrModifyReq { dn: TrimString(dn.to_string()) }, &account_id, cert_conf_id, funs, ctx).await?;
        Ok(account_id)
    }

    async fn get_ldap_client(tenant_id: Option<String>, code: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<(LdapClient, IamCertConfLdapResp, String)> {
        let cert_conf_id = IamCertServ::get_cert_conf_id_by_code(&format!("{}{}", IamCertExtKind::Ldap, code), tenant_id, funs).await?;
        let cert_conf = Self::get_cert_conf(&cert_conf_id, funs, ctx).await?;
        let client = LdapClient::new(&cert_conf.conn_uri, cert_conf.is_tls, &cert_conf.base_dn).await?;
        Ok((client, cert_conf, cert_conf_id))
    }

    async fn get_ldap_cert_account_by_account(account_id: &str, rel_rbum_cert_conf_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Option<String>> {
        let result = RbumCertServ::find_rbums(
            &RbumCertFilterReq {
                rel_rbum_cert_conf_ids: Some(vec![rel_rbum_cert_conf_id.to_string()]),
                rel_rbum_id: Some(account_id.to_string()),
                ..Default::default()
            },
            None,
            None,
            funs,
            ctx,
        )
        .await?
        .first()
        .map(|r| r.id.to_string());
        Ok(result)
    }
}

mod ldap {
    use std::collections::HashMap;

    use ldap3::{log::warn, Ldap, LdapConnAsync, LdapConnSettings, Scope, SearchEntry};
    use serde::{Deserialize, Serialize};
    use tardis::basic::{error::TardisError, result::TardisResult};

    pub struct LdapClient {
        ldap: Ldap,
        base_dn: String,
    }

    impl LdapClient {
        pub async fn new(url: &str, tls: bool, base_dn: &str) -> TardisResult<LdapClient> {
            let setting = if tls {
                LdapConnSettings::new().set_starttls(true).set_no_tls_verify(true)
            } else {
                LdapConnSettings::new()
            };
            let (conn, ldap) = LdapConnAsync::with_settings(setting, url).await.map_err(|e| TardisError::internal_error(&format!("[Iam.Ldap] connection error: {:?}", e), ""))?;
            ldap3::drive!(conn);
            Ok(LdapClient {
                ldap,
                base_dn: base_dn.to_string(),
            })
        }

        pub async fn bind(&mut self, cn: &str, pw: &str) -> TardisResult<Option<String>> {
            let dn = format!("cn={},{}", cn, self.base_dn);
            let result = self.ldap.simple_bind(&dn, pw).await.map_err(|e| TardisError::internal_error(&format!("[Iam.Ldap] bind error: {:?}", e), ""))?.success().map(|_| ());
            if let Some(err) = result.err() {
                warn!("[Iam.Ldap] ldap bind error: {:?}", err);
                Ok(None)
            } else {
                Ok(Some(dn))
            }
        }

        pub async fn search(&mut self, filter: &str, return_attr: &Vec<&str>) -> TardisResult<Vec<LdapSearchResp>> {
            let (rs, _) = self
                .ldap
                .search(&self.base_dn, Scope::Subtree, filter, return_attr)
                .await
                .map_err(|e| TardisError::internal_error(&format!("[Iam.Ldap] search error: {:?}", e), ""))?
                .success()
                .map_err(|e| TardisError::internal_error(&format!("[Iam.Ldap] search error: {:?}", e), ""))?;
            let result = rs.into_iter().map(SearchEntry::construct).map(|r| LdapSearchResp { dn: r.dn, attrs: r.attrs }).collect();
            Ok(result)
        }

        pub async fn get_by_dn(&mut self, dn: &str, return_attr: &Vec<&str>) -> TardisResult<Option<LdapSearchResp>> {
            let (rs, _) = self
                .ldap
                .search(dn, Scope::Subtree, "objectClass=*", return_attr)
                .await
                .map_err(|e| TardisError::internal_error(&format!("[Iam.Ldap] search error: {:?}", e), ""))?
                .success()
                .map_err(|e| TardisError::internal_error(&format!("[Iam.Ldap] search error: {:?}", e), ""))?;
            let result = rs.into_iter().map(SearchEntry::construct).map(|r| LdapSearchResp { dn: r.dn, attrs: r.attrs }).collect::<Vec<LdapSearchResp>>();
            if let Some(result) = result.first() {
                Ok(Some(result.clone()))
            } else {
                Ok(None)
            }
        }

        pub async fn unbind(&mut self) -> TardisResult<()> {
            self.ldap.unbind().await.map_err(|e| TardisError::internal_error(&format!("[Iam.Ldap] unbind error: {:?}", e), ""))
        }
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    pub struct LdapSearchResp {
        pub dn: String,
        pub attrs: HashMap<String, Vec<String>>,
    }

    impl LdapSearchResp {
        pub fn get_simple_attr(&self, attr_name: &str) -> Option<String> {
            if let Some(values) = self.attrs.get(attr_name) {
                if let Some(value) = values.first() {
                    return Some(value.to_string());
                }
            }
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use tardis::basic::result::TardisResult;

    use super::ldap::LdapClient;
    use tardis::tokio;

    const LDAP_URL: &str = "ldap://x.x.x.x";
    const LDAP_TLS: bool = false;
    const LDAP_BASE_DN: &str = "ou=x,dc=x,dc=x";
    const LDAP_USER: &str = "cn=admin";
    const LDAP_PW: &str = "123456";

    #[tokio::test]
    #[ignore]
    async fn bind() -> TardisResult<()> {
        let mut ldap = LdapClient::new(LDAP_URL, LDAP_TLS, LDAP_BASE_DN).await?;
        let result = ldap.bind(LDAP_USER, LDAP_PW).await?;
        assert!(result.is_some());
        Ok(())
    }

    #[tokio::test]
    #[ignore]
    async fn search() -> TardisResult<()> {
        let mut ldap = LdapClient::new(LDAP_URL, LDAP_TLS, LDAP_BASE_DN).await?;
        ldap.bind(LDAP_USER, LDAP_PW).await?;
        let result = ldap.search("(&(objectClass=inetOrgPerson)(cn=*130*))", &vec!["dn", "cn", "displayName"]).await?;
        // assert_eq!(result.len(), 1);
        Ok(())
    }

    #[tokio::test]
    #[ignore]
    async fn unbind() -> TardisResult<()> {
        let mut ldap = LdapClient::new(LDAP_URL, LDAP_TLS, LDAP_BASE_DN).await?;
        ldap.bind(LDAP_USER, LDAP_PW).await?;
        ldap.unbind().await?;
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
struct IamCertLdapServerAuthInfo {
    pub is_tls: bool,
    pub principal: String,
    pub credentials: String,
    pub base_dn: String,
    pub search_base_filter: String,
    pub field_display_name: String,
}
