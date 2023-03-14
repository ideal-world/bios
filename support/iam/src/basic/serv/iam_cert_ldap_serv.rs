use ldap3::log::{error, warn};
use std::collections::HashMap;

use self::ldap::LdapClient;
use super::{iam_account_serv::IamAccountServ, iam_cert_serv::IamCertServ, iam_tenant_serv::IamTenantServ};
use crate::basic::dto::iam_account_dto::{IamAccountAddByLdapResp, IamAccountAggModifyReq, IamAccountExtSysAddReq, IamAccountExtSysBatchAddReq};
use crate::basic::serv::iam_cert_user_pwd_serv::IamCertUserPwdServ;
use crate::console_passport::dto::iam_cp_cert_dto::IamCpUserPwdBindWithLdapReq;
use crate::console_passport::serv::iam_cp_cert_user_pwd_serv::IamCpCertUserPwdServ;
use crate::iam_enumeration::{IamCertExtKind, IamCertKernelKind, WayToAdd};
use crate::{
    basic::dto::{
        iam_account_dto::{IamAccountAggAddReq, IamAccountExtSysResp},
        iam_cert_conf_dto::{IamCertConfLdapAddOrModifyReq, IamCertConfLdapResp},
        iam_cert_dto::IamCertLdapAddOrModifyReq,
        iam_filer_dto::IamTenantFilterReq,
    },
    iam_config::IamBasicConfigApi,
    iam_enumeration,
};
use bios_basic::rbum::dto::rbum_cert_dto::RbumCertExt;
use bios_basic::rbum::dto::rbum_filer_dto::RbumBasicFilterReq;
use bios_basic::rbum::rbum_enumeration::{RbumCertConfStatusKind, RbumScopeLevelKind};
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
use tardis::regex::Regex;
use tardis::web::poem_openapi;
use tardis::{
    basic::{dto::TardisContext, field::TrimString, result::TardisResult},
    TardisFuns, TardisFunsInst,
};

pub struct IamCertLdapServ;

impl IamCertLdapServ {
    //ldap only can be one recode in each tenant
    pub async fn add_cert_conf(add_req: &IamCertConfLdapAddOrModifyReq, rel_iam_item_id: Option<String>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<String> {
        //验证cert conf配置是否正确
        let ldap_auth_info = IamCertLdapServerAuthInfo::from((*add_req).clone());
        let mut ldap_client = LdapClient::new(&add_req.conn_uri, ldap_auth_info.port, ldap_auth_info.is_tls, &ldap_auth_info.base_dn).await.map_err(|e| {
            funs.err().bad_request(
                "IamCertLdap",
                "add",
                &format!("add cert conf err: ldap conf parameter error,and err:{e}"),
                "400-iam--ldap-cert-add-parameter-incorrect",
            )
        })?;
        if ldap_client.bind(&ldap_auth_info.principal, &ldap_auth_info.credentials).await?.is_none() {
            ldap_client.unbind().await?;
            return Err(funs.err().unauthorized("ldap_cert_conf", "add", "validation error", "401-rbum-cert-valid-error"));
        }
        RbumCertConfServ::add_rbum(
            &mut RbumCertConfAddReq {
                kind: TrimString(IamCertExtKind::Ldap.to_string()),
                supplier: Some(add_req.supplier.clone()),
                name: TrimString(add_req.name.clone()),
                note: None,
                ak_note: None,
                ak_rule: None,
                sk_note: None,
                sk_rule: None,
                ext: Some(Self::iam_cert_ldap_server_auth_info_to_json(add_req)?),
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
                status: if add_req.enabled {
                    RbumCertConfStatusKind::Enabled
                } else {
                    RbumCertConfStatusKind::Disabled
                },
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
                ext: Some(Self::iam_cert_ldap_server_auth_info_to_json(modify_req)?),
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
                status: if modify_req.enabled {
                    Some(RbumCertConfStatusKind::Enabled)
                } else {
                    Some(RbumCertConfStatusKind::Disabled)
                },
            },
            funs,
            ctx,
        )
        .await
    }

    pub async fn get_cert_conf_by_ctx(funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Option<IamCertConfLdapResp>> {
        if let Some(resp) = RbumCertConfServ::find_one_rbum(
            &RbumCertConfFilterReq {
                basic: RbumBasicFilterReq {
                    own_paths: Some(ctx.own_paths.clone()),
                    ..Default::default()
                },
                kind: Some(TrimString("Ldap".to_string())),
                status: Some(RbumCertConfStatusKind::Enabled),
                rel_rbum_domain_id: Some(funs.iam_basic_domain_iam_id()),
                rel_rbum_item_id: Some(ctx.own_paths.clone()),
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?
        {
            let result = TardisFuns::json.str_to_obj::<IamCertLdapServerAuthInfo>(&resp.ext).map(|info| IamCertConfLdapResp {
                id: resp.id,
                supplier: resp.supplier,
                conn_uri: resp.conn_uri,
                is_tls: info.is_tls,
                principal: info.principal,
                credentials: info.credentials,
                base_dn: info.base_dn,
                port: info.port,
                account_unique_id: info.account_unique_id,
                account_field_map: info.account_field_map,
                // org_unique_id: info.org_unique_id,
                // org_field_map: info.org_field_map,
            })?;
            Ok(Some(result))
        } else {
            Ok(None)
        }
    }

    pub async fn get_cert_conf(id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<IamCertConfLdapResp> {
        RbumCertConfServ::get_rbum(id, &RbumCertConfFilterReq::default(), funs, ctx).await.map(|resp| {
            TardisFuns::json
                .str_to_obj::<IamCertLdapServerAuthInfo>(&resp.ext)
                .map(|info| IamCertConfLdapResp {
                    id: resp.id,
                    supplier: resp.supplier,
                    conn_uri: resp.conn_uri,
                    is_tls: info.is_tls,
                    principal: info.principal,
                    credentials: info.credentials,
                    base_dn: info.base_dn,
                    port: info.port,
                    account_unique_id: info.account_unique_id,
                    account_field_map: info.account_field_map,
                    // org_unique_id: info.org_unique_id,
                    // org_field_map: info.org_field_map,
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
                    ext: Some(TardisFuns::json.obj_to_string(&RbumCertExt {
                        account_unique_id: add_or_modify_req.account_unique_id.clone(),
                    })?),
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
                    kind: None,
                    supplier: None,
                    vcode: None,
                    ext: Some(TardisFuns::json.obj_to_string(&RbumCertExt {
                        account_unique_id: add_or_modify_req.account_unique_id.clone(),
                    })?),
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

    ///获取dn对应的account_id
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

    ///根据add_req的account_id（dn）获取或者添加账号
    /// 始终返回（account_id,dn）
    pub async fn get_or_add_account_without_verify(
        add_req: IamAccountExtSysAddReq,
        tenant_id: Option<String>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<(String, String)> {
        let dn = &add_req.account_id;
        let cert_conf_id = IamCertServ::get_cert_conf_id_by_kind_supplier(&IamCertExtKind::Ldap.to_string(), &add_req.code.clone(), tenant_id, funs).await?;
        let cert_conf = Self::get_cert_conf(&cert_conf_id, funs, ctx).await?;
        if let Some(account_id) = Self::get_cert_rel_account_by_dn(dn, &cert_conf_id, funs, ctx).await? {
            return Ok((account_id, dn.to_string()));
        }
        let mut ldap_client = LdapClient::new(&cert_conf.conn_uri, cert_conf.port, cert_conf.is_tls, &cert_conf.base_dn).await?;
        if ldap_client.bind(&cert_conf.principal, &cert_conf.credentials).await?.is_none() {
            ldap_client.unbind().await?;
            return Err(funs.err().unauthorized("rbum_cert", "search_accounts", "ldap admin validation error", "401-rbum-cert-valid-error"));
        };
        let account = ldap_client
            .get_by_dn(
                dn,
                &vec!["dn", "cn", &cert_conf.account_field_map.field_user_name, &cert_conf.account_field_map.field_display_name],
            )
            .await?;
        ldap_client.unbind().await?;
        if let Some(account) = account {
            let mock_ctx = TardisContext {
                own_paths: ctx.own_paths.clone(),
                owner: TardisFuns::field.nanoid(),
                ..Default::default()
            };
            let account_id = Self::do_add_account(
                &account.dn,
                &account.get_simple_attr(&cert_conf.account_field_map.field_display_name).unwrap_or_default(),
                &account.get_simple_attr(&cert_conf.account_field_map.field_user_name).unwrap_or_default(),
                &account.get_simple_attr(&cert_conf.account_unique_id).unwrap_or_default(),
                &format!("{}0Pw$", TardisFuns::field.nanoid_len(6)),
                &cert_conf_id,
                RbumCertStatusKind::Enabled,
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
        let mock_ctx = Self::generate_default_mock_ctx(code, tenant_id.clone(), funs).await;
        let (mut ldap_client, _, cert_conf_id) = Self::get_ldap_client(Some(mock_ctx.own_paths.clone()), code, funs, &mock_ctx).await?;
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
                &cert_conf.package_filter_by_fuzzy_search_account(user_or_display_name),
                &vec!["dn", "cn", &cert_conf.account_field_map.field_display_name],
            )
            .await?
            .into_iter()
            .map(|r| IamAccountExtSysResp {
                user_name: r.get_simple_attr(&cert_conf.account_field_map.field_user_name).unwrap_or_default(),
                display_name: r.get_simple_attr(&cert_conf.account_field_map.field_display_name).unwrap_or_default(),
                account_unique_id: r.get_simple_attr(&cert_conf.account_unique_id).unwrap_or_default(),
                account_id: r.dn,
            })
            .collect();
        ldap_client.unbind().await?;
        Ok(accounts)
    }

    pub async fn check_user_pwd_is_bind(ak: &str, supplier: &str, tenant_id: Option<String>, funs: &TardisFunsInst) -> TardisResult<bool> {
        if tenant_id.is_some() && IamTenantServ::is_disabled(&tenant_id.clone().unwrap(), funs).await? {
            return Err(funs.err().conflict(
                "user_pwd",
                "check_bind",
                &format!("tenant {} is disabled", tenant_id.unwrap()),
                "409-iam-tenant-is-disabled",
            ));
        }
        let tenant_ldap_cert_conf_id_result = IamCertServ::get_cert_conf_id_by_kind_supplier(&IamCertExtKind::Ldap.to_string(), supplier, tenant_id.clone(), funs).await;
        let global_ldap_cert_conf_id_result = IamCertServ::get_cert_conf_id_by_kind_supplier(&IamCertExtKind::Ldap.to_string(), supplier, None, funs).await;
        if tenant_ldap_cert_conf_id_result.is_err() && global_ldap_cert_conf_id_result.is_err() {
            return Ok(false);
        }
        let tenant_userpwd_cert_conf_id = IamCertServ::get_cert_conf_id_by_kind(&IamCertKernelKind::UserPwd.to_string(), tenant_id.clone(), funs).await?;
        let tenant_exist = RbumCertServ::check_exist(ak, &tenant_userpwd_cert_conf_id, &tenant_id.clone().unwrap_or_default(), funs).await?;
        //if tenant have cert_conf,then use tenant level
        let (ldap_cert_conf_id, userpwd_cert_conf_id, userpwd_cert_exist) = if tenant_ldap_cert_conf_id_result.is_ok() {
            (tenant_ldap_cert_conf_id_result?, tenant_userpwd_cert_conf_id, tenant_exist)
        } else {
            let userpwd_cert_conf_id = IamCertServ::get_cert_conf_id_by_kind(&IamCertKernelKind::UserPwd.to_string(), None, funs).await?;
            let global_userpwd_exist = RbumCertServ::check_exist(ak, &userpwd_cert_conf_id, "", funs).await?;
            let exist = if tenant_id.is_some() && !global_userpwd_exist {
                if tenant_exist {
                    return Err(funs.err().conflict("user_pwd", "check_bind", "user is private", "409-user-is-private"));
                } else {
                    false
                }
            } else {
                true
            };
            (global_ldap_cert_conf_id_result?, userpwd_cert_conf_id, exist)
        };

        if userpwd_cert_exist {
            let mock_ctx = Self::generate_default_mock_ctx(supplier, tenant_id.clone(), funs).await;
            if let Some(account_id) = IamCpCertUserPwdServ::get_cert_rel_account_by_user_name(ak, &userpwd_cert_conf_id, funs, &mock_ctx).await? {
                let cert_id = Self::get_ldap_cert_account_by_account(&account_id, &ldap_cert_conf_id, funs, &mock_ctx).await?;
                if cert_id.is_some() {
                    Ok(true)
                } else {
                    Ok(false)
                }
            } else {
                // Unreachable code
                error!("function:check_bind,code should not be executed");
                Ok(false)
            }
        } else {
            Err(funs.err().not_found("user_pwd", "check_bind", "not found cert record", "404-rbum-*-obj-not-exist"))
        }
    }

    pub async fn bind_or_create_user_pwd_by_ldap(login_req: &IamCpUserPwdBindWithLdapReq, funs: &TardisFunsInst) -> TardisResult<(String, String)> {
        let tenant_id = login_req.tenant_id.clone();
        // mock_ctx decide whether the login mode is global or tenant level
        let mut mock_ctx = Self::generate_default_mock_ctx(login_req.ldap_login.code.as_ref(), tenant_id.clone(), funs).await;

        let (mut ldap_client, cert_conf, cert_conf_id) =
            Self::get_ldap_client(Some(mock_ctx.own_paths.clone()), login_req.ldap_login.code.to_string().as_str(), funs, &mock_ctx).await?;
        let dn = if let Some(dn) = ldap_client.bind(login_req.ldap_login.name.to_string().as_str(), login_req.ldap_login.password.as_str()).await? {
            dn
        } else {
            ldap_client.unbind().await?;
            return Err(funs.err().unauthorized("rbum_cert", "get_or_add_account", "validation error", "401-rbum-cert-valid-error"));
        };

        let account = ldap_client.get_by_dn(&dn, &cert_conf.package_account_return_attr_with(vec!["dn", "cn"])).await?;
        ldap_client.unbind().await?;
        if let Some(account) = account {
            mock_ctx.owner = TardisFuns::field.nanoid();
            let account_id = if let Some(ak) = login_req.bind_user_pwd.ak.clone() {
                // bind user_pwd with ldap cert
                Self::bind_user_pwd_by_ldap(
                    &dn,
                    &account.get_simple_attr(&cert_conf.account_unique_id).unwrap_or_default(),
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
                if tenant_id.is_some() && !IamTenantServ::get_item(&tenant_id.unwrap(), &IamTenantFilterReq::default(), funs, &mock_ctx).await?.account_self_reg {
                    return Err(funs.err().not_found(
                        "rbum_cert",
                        "create_user_pwd_by_ldap",
                        &format!("not found ldap cert(openid): {} and self-registration disabled", &dn),
                        "401-rbum-cert-valid-error",
                    ));
                }

                Self::do_add_account(
                    &dn,
                    &account.get_simple_attr(&cert_conf.account_field_map.field_display_name).unwrap_or_default(),
                    &account.get_simple_attr(&cert_conf.account_field_map.field_user_name).unwrap_or_default(),
                    &account.get_simple_attr(&cert_conf.account_unique_id).unwrap_or_default(),
                    login_req.bind_user_pwd.sk.as_ref(),
                    &cert_conf_id,
                    RbumCertStatusKind::Enabled,
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

    pub async fn bind_user_pwd_by_ldap(
        dn: &str,
        account_unique_id: &str,
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
            let global_check = RbumCertServ::validate_by_ak_and_basic_sk(
                user_name,
                password,
                &RbumCertRelKind::Item,
                false,
                Some("".to_string()),
                vec![&IamCertKernelKind::UserPwd.to_string()],
                funs,
            )
            .await;
            if global_check.is_err() {
                let tenant_check = RbumCertServ::validate_by_ak_and_basic_sk(
                    user_name,
                    password,
                    &RbumCertRelKind::Item,
                    false,
                    Some(tenant_id.clone()),
                    vec![&IamCertKernelKind::UserPwd.to_string()],
                    funs,
                )
                .await;
                if tenant_check.is_ok() && ctx.own_paths.is_empty() {
                    return Err(funs.err().conflict("rbum_cert", "bind_user_pwd_by_ldap", "user is private", "409-user-is-private"));
                } else if tenant_check.is_err() {
                    return Err(funs.err().unauthorized("rbum_cert", "valid", "validation error", "401-rbum-cert-valid-error"));
                } else {
                    tenant_check?
                }
            } else {
                global_check?
            }
        } else {
            RbumCertServ::validate_by_ak_and_basic_sk(
                user_name,
                password,
                &RbumCertRelKind::Item,
                false,
                Some("".to_string()),
                vec![&IamCertKernelKind::UserPwd.to_string()],
                funs,
            )
            .await?
        };
        if Self::check_user_pwd_is_bind(user_name, code, tenant_id.clone(), funs).await? {
            return Err(funs.err().not_found("rbum_cert", "bind_user_pwd_by_ldap", "user is bound by ldap", "409-iam-user-is-bound"));
        }
        //添加这个用户的ldap登录cert
        Self::add_or_modify_cert(
            &IamCertLdapAddOrModifyReq {
                dn: TrimString(dn.to_string()),
                account_unique_id: Some(account_unique_id.to_string()),
                status: RbumCertStatusKind::Enabled,
            },
            &rbum_item_id,
            cert_conf_id,
            funs,
            ctx,
        )
        .await?;
        Ok(rbum_item_id)
    }

    //同步ldap人员到iam
    pub async fn iam_sync_ldap_user_to_iam(conf_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let sync_config = if let Some(sync_config) = IamCertServ::get_sync_third_integration_config(funs, ctx).await? {
            sync_config
        } else {
            return Err(funs.err().conflict("ldap_account", "sync", "should have sync config!", "iam-not-found-sync-config"));
        };

        let (mut ldap_client, cert_conf, cert_conf_id) = Self::get_ldap_client(Some(ctx.own_paths.clone()), "", funs, ctx).await?;
        if ldap_client.bind(&cert_conf.principal, &cert_conf.credentials).await?.is_none() {
            ldap_client.unbind().await?;
            return Err(funs.err().unauthorized("ldap_cert_conf", "add", "validation error", "401-rbum-cert-valid-error"));
        }
        let mut map = HashMap::new();
        let ldap_account: Vec<IamAccountExtSysResp> = ldap_client
            .search(
                &cert_conf.account_field_map.search_base_filter.clone().unwrap_or("objectClass=person".to_string()),
                &cert_conf.package_account_return_attr_with(vec!["dn"]),
            )
            .await?
            .into_iter()
            .map(|r| IamAccountExtSysResp {
                user_name: r.get_simple_attr(&cert_conf.account_field_map.field_user_name).unwrap_or_default(),
                display_name: r.get_simple_attr(&cert_conf.account_field_map.field_display_name).unwrap_or_default(),
                account_unique_id: r.get_simple_attr(&cert_conf.account_unique_id).unwrap_or_default(),
                account_id: r.dn,
            })
            .collect();
        ldap_account.iter().for_each(|r| {
            map.insert(r.account_unique_id.clone(), r);
        });

        ldap_client.unbind().await?;

        let certs = IamCertServ::find_certs(
            &RbumCertFilterReq {
                basic: RbumBasicFilterReq {
                    own_paths: Some(ctx.own_paths.clone()),
                    with_sub_own_paths: false,
                    ..Default::default()
                },
                kind: Some(IamCertExtKind::Ldap.to_string()),
                supplier: None,
                status: None,
                rel_rbum_kind: None,
                rel_rbum_id: None,
                ..Default::default()
            },
            None,
            None,
            funs,
            ctx,
        )
        .await?;
        for cert in certs {
            let account_unique_id = if cert.ext.is_empty() {
                continue;
            } else {
                let ext = TardisFuns::json.str_to_obj::<RbumCertExt>(&cert.ext)?;
                if let Some(account_unique_id) = ext.account_unique_id {
                    account_unique_id
                } else {
                    continue;
                }
            };
            if let Some(_iam_account_ext_sys_resp) = map.get(&account_unique_id) {
                //并集 两边都有相同的
                map.remove(&account_unique_id);
            } else {
                //ldap没有 iam有的 需要同步删除
                match sync_config.account_way_to_delete {
                    iam_enumeration::WayToDelete::DeleteCert => {
                        RbumCertServ::modify_rbum(
                            &cert.id,
                            &mut RbumCertModifyReq {
                                ak: None,
                                sk: None,
                                ext: None,
                                start_time: None,
                                end_time: None,
                                conn_uri: None,
                                status: Some(RbumCertStatusKind::Disabled),
                            },
                            funs,
                            ctx,
                        )
                        .await?;
                    }
                    iam_enumeration::WayToDelete::Disable => {
                        IamAccountServ::modify_account_agg(
                            &cert.rel_rbum_id,
                            &IamAccountAggModifyReq {
                                name: None,
                                scope_level: None,
                                disabled: Some(true),
                                icon: None,
                                role_ids: None,
                                org_cate_ids: None,
                                exts: None,
                            },
                            funs,
                            ctx,
                        )
                        .await?;
                    }
                    iam_enumeration::WayToDelete::DeleteAccount => {
                        IamAccountServ::delete_item_with_all_rels(&cert.rel_rbum_id, funs, ctx).await?;
                    }
                }
            };
        }
        //ldap有的 但是iam没有的 需要添加
        for account_unique_id in map.keys() {
            let mock_ctx = TardisContext {
                owner: TardisFuns::field.nanoid(),
                ..ctx.clone()
            };
            let ldap_resp = map.get(account_unique_id).unwrap();
            match sync_config.account_way_to_add {
                WayToAdd::SynchronizeCert => {
                    Self::do_add_account(
                        &ldap_resp.account_id,
                        &ldap_resp.display_name,
                        &ldap_resp.user_name,
                        &ldap_resp.account_unique_id,
                        &format!("{}0Pw$", TardisFuns::field.nanoid_len(6)),
                        &cert_conf_id,
                        RbumCertStatusKind::Enabled,
                        funs,
                        &mock_ctx,
                    )
                    .await?;
                }
                WayToAdd::NoSynchronizeCert => {
                    Self::do_add_account(
                        &ldap_resp.account_id,
                        &ldap_resp.display_name,
                        &ldap_resp.user_name,
                        &ldap_resp.account_unique_id,
                        &format!("{}0Pw$", TardisFuns::field.nanoid_len(6)),
                        &cert_conf_id,
                        RbumCertStatusKind::Disabled,
                        funs,
                        &mock_ctx,
                    )
                    .await?;
                }
            }
        }
        Ok(())
    }

    pub async fn generate_default_mock_ctx(supplier: &str, tenant_id: Option<String>, funs: &TardisFunsInst) -> TardisContext {
        //if tenant_id is some and tenant have cert_conf \
        // then assign tenant_id to own_paths
        if IamCertServ::get_cert_conf_id_by_kind_supplier(&IamCertExtKind::Ldap.to_string(), supplier, tenant_id.clone(), funs).await.is_ok() {
            if let Some(tenant_id) = tenant_id {
                return TardisContext {
                    own_paths: tenant_id,
                    ..Default::default()
                };
            }
        }
        TardisContext { ..Default::default() }
    }

    fn iam_cert_ldap_server_auth_info_to_json(add_req: &IamCertConfLdapAddOrModifyReq) -> TardisResult<String> {
        TardisFuns::json.obj_to_string::<IamCertLdapServerAuthInfo>(&(add_req.clone().into()))
    }

    /// do add account and ldap/userPwd cert \
    /// and return account_id
    async fn do_add_account(
        dn: &str,
        account_name: &str,
        cert_user_name: &str,
        account_unique_id: &str,
        userpwd_password: &str,
        ldap_cert_conf_id: &str,
        cert_status: RbumCertStatusKind,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<String> {
        let account_id = IamAccountServ::add_account_agg(
            &IamAccountAggAddReq {
                id: Some(TrimString(ctx.owner.clone())),
                name: TrimString(account_name.to_string()),
                cert_user_name: IamCertUserPwdServ::rename_ak_if_duplicate(cert_user_name, funs, ctx).await?,
                cert_password: userpwd_password.into(),
                cert_phone: None,
                cert_mail: None,
                role_ids: None,
                org_node_ids: None,
                scope_level: Some(RbumScopeLevelKind::Root),
                disabled: None,
                icon: None,
                exts: HashMap::new(),
                status: Some(RbumCertStatusKind::Pending),
            },
            funs,
            ctx,
        )
        .await?;
        Self::add_or_modify_cert(
            &IamCertLdapAddOrModifyReq {
                dn: TrimString(dn.to_string()),
                account_unique_id: Some(account_unique_id.to_string()),
                status: cert_status,
            },
            &account_id,
            ldap_cert_conf_id,
            funs,
            ctx,
        )
        .await?;
        Ok(account_id)
    }

    async fn get_ldap_client(tenant_id: Option<String>, supplier: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<(LdapClient, IamCertConfLdapResp, String)> {
        let cert_conf_id = IamCertServ::get_cert_conf_id_by_kind_supplier(&IamCertExtKind::Ldap.to_string(), supplier, tenant_id, funs).await?;
        let cert_conf = Self::get_cert_conf(&cert_conf_id, funs, ctx).await?;
        let client = LdapClient::new(&cert_conf.conn_uri, cert_conf.port, cert_conf.is_tls, &cert_conf.base_dn).await?;
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
    ///# Examples
    ///
    ///```
    ///  use bios_iam::basic::serv::iam_cert_ldap_serv::IamCertLdapServ;
    ///  assert_eq!(IamCertLdapServ::dn_to_cn("cn=admin,ou=x,dc=x,dc=x"), "admin".to_string());
    ///  assert_eq!(IamCertLdapServ::dn_to_cn("ou=x,dc=x,dc=x"), "ou=x,dc=x,dc=x".to_string());
    ///  assert_eq!(IamCertLdapServ::dn_to_cn("cn=,ou=x,dc=x,dc=x"), "".to_string());
    ///  assert_eq!(IamCertLdapServ::dn_to_cn("sdfafasdf"), "sdfafasdf".to_string());
    /// ```
    pub fn dn_to_cn(dn: &str) -> String {
        let dn_regex = Regex::new(r"(,|^)[cC][nN]=(.+?)(,|$)").expect("Regular parsing error");
        let cn = if dn_regex.is_match(dn) {
            let int = dn.find("cn=").unwrap_or_default();
            let a = &dn[int + 3..];
            let int = a.find(',').unwrap_or_default();
            &a[..int]
        } else {
            warn!("dn:{} is not match regex!", dn);
            dn
        };
        cn.to_string()
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
        pub async fn new(url: &str, port: u16, tls: bool, base_dn: &str) -> TardisResult<LdapClient> {
            let setting = if tls {
                LdapConnSettings::new().set_starttls(true).set_no_tls_verify(true)
            } else {
                LdapConnSettings::new()
            };
            let url = if &url[url.len() - 1..] == "/" {
                format!("{}:{port}", &url[..url.len() - 1])
            } else {
                format!("{url}:{port}")
            };
            let (conn, ldap) = LdapConnAsync::with_settings(setting, &url).await.map_err(|e| TardisError::internal_error(&format!("[Iam.Ldap] connection error: {e:?}"), ""))?;
            ldap3::drive!(conn);
            Ok(LdapClient {
                ldap,
                base_dn: base_dn.to_string(),
            })
        }

        pub async fn bind(&mut self, cn: &str, pw: &str) -> TardisResult<Option<String>> {
            let dn = format!("cn={},{}", cn, self.base_dn);
            let result = self.ldap.simple_bind(&dn, pw).await.map_err(|e| TardisError::internal_error(&format!("[Iam.Ldap] bind error: {e:?}"), ""))?.success().map(|_| ());
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
                .map_err(|e| TardisError::internal_error(&format!("[Iam.Ldap] search error: {e:?}"), ""))?
                .success()
                .map_err(|e| TardisError::internal_error(&format!("[Iam.Ldap] search error: {e:?}"), ""))?;
            let result = rs.into_iter().map(SearchEntry::construct).map(|r| LdapSearchResp { dn: r.dn, attrs: r.attrs }).collect();
            Ok(result)
        }

        pub async fn get_by_dn(&mut self, dn: &str, return_attr: &Vec<&str>) -> TardisResult<Option<LdapSearchResp>> {
            let (rs, _) = self
                .ldap
                .search(dn, Scope::Subtree, "objectClass=*", return_attr)
                .await
                .map_err(|e| TardisError::internal_error(&format!("[Iam.Ldap] search error: {e:?}"), ""))?
                .success()
                .map_err(|e| TardisError::internal_error(&format!("[Iam.Ldap] search error: {e:?}"), ""))?;
            let result = rs.into_iter().map(SearchEntry::construct).map(|r| LdapSearchResp { dn: r.dn, attrs: r.attrs }).collect::<Vec<LdapSearchResp>>();
            if let Some(result) = result.first() {
                Ok(Some(result.clone()))
            } else {
                Ok(None)
            }
        }

        pub async fn unbind(&mut self) -> TardisResult<()> {
            self.ldap.unbind().await.map_err(|e| TardisError::internal_error(&format!("[Iam.Ldap] unbind error: {e:?}"), ""))
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
    const LDAP_PORT: u16 = 389;
    const LDAP_TLS: bool = false;
    const LDAP_BASE_DN: &str = "ou=x,dc=x,dc=x";
    const LDAP_USER: &str = "cn=admin";
    const LDAP_PW: &str = "123456";

    #[tokio::test]
    #[ignore]
    async fn bind() -> TardisResult<()> {
        let mut ldap = LdapClient::new(LDAP_URL, LDAP_PORT, LDAP_TLS, LDAP_BASE_DN).await?;
        let result = ldap.bind(LDAP_USER, LDAP_PW).await?;
        assert!(result.is_some());
        Ok(())
    }

    #[tokio::test]
    #[ignore]
    async fn search() -> TardisResult<()> {
        let mut ldap = LdapClient::new(LDAP_URL, LDAP_PORT, LDAP_TLS, LDAP_BASE_DN).await?;
        ldap.bind(LDAP_USER, LDAP_PW).await?;
        let result = ldap.search("(&(objectClass=inetOrgPerson)(cn=*130*))", &vec!["dn", "cn", "displayName"]).await?;
        // assert_eq!(result.len(), 1);
        Ok(())
    }

    #[tokio::test]
    #[ignore]
    async fn unbind() -> TardisResult<()> {
        let mut ldap = LdapClient::new(LDAP_URL, LDAP_PORT, LDAP_TLS, LDAP_BASE_DN).await?;
        ldap.bind(LDAP_USER, LDAP_PW).await?;
        ldap.unbind().await?;
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
struct IamCertLdapServerAuthInfo {
    // server_uri is in RbumCertConf's conn_uri
    pub port: u16,
    pub is_tls: bool,
    pub principal: String,
    pub credentials: String,
    pub base_dn: String,

    pub account_unique_id: String,
    pub account_field_map: AccountFieldMap,

    // pub org_unique_id: String,
    // pub org_field_map: OrgFieldMap,
}

impl From<IamCertConfLdapAddOrModifyReq> for IamCertLdapServerAuthInfo {
    fn from(v: IamCertConfLdapAddOrModifyReq) -> Self {
        IamCertLdapServerAuthInfo {
            port: v.port.unwrap_or(if v.is_tls { 636 } else { 389 }),
            is_tls: v.is_tls,
            principal: v.principal.to_string(),
            credentials: v.credentials.to_string(),
            base_dn: v.base_dn.to_string(),
            account_unique_id: v.account_unique_id.clone(),
            account_field_map: v.account_field_map,
            // org_unique_id: v.org_unique_id.clone(),
            // org_field_map: v.org_field_map,
        }
    }
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Clone)]
pub struct AccountFieldMap {
    // The base condition fragment of the search filter,
    // without the outermost parentheses.
    // For example, the complete search filter is: (&(objectCategory=group)(|(cn=Test*)(cn=Admin*))),
    // this field can be &(objectCategory=group)
    pub search_base_filter: Option<String>,
    pub field_user_name: String,
    pub field_display_name: String,
    pub field_mobile: String,
    pub field_email: String,

    pub field_user_name_remarks: String,
    pub field_display_name_remarks: String,
    pub field_mobile_remarks: String,
    pub field_email_remarks: String,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Clone)]
pub struct OrgFieldMap {
    pub search_base_filter: Option<String>,
    pub field_dept_id: String,
    pub field_dept_name: String,
    pub field_parent_dept_id: String,

    pub field_dept_id_remarks: String,
    pub field_dept_name_remarks: String,
    pub field_parent_dept_id_remarks: String,
}
