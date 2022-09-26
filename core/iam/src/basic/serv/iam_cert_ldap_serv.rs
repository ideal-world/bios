use std::collections::HashMap;

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

use self::ldap::LdapClient;

use super::{iam_account_serv::IamAccountServ, iam_cert_serv::IamCertServ, iam_tenant_serv::IamTenantServ};

pub struct IamCertLdapServ;

impl IamCertLdapServ {
    pub async fn add_cert_conf(add_req: &IamCertConfLdapAddOrModifyReq, rel_iam_item_id: String, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<String> {
        RbumCertConfServ::add_rbum(
            &mut RbumCertConfAddReq {
                code: TrimString(IamCertExtKind::Ldap.to_string()),
                name: TrimString(IamCertExtKind::Ldap.to_string()),
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
                rest_by_kinds: None,
                expire_sec: None,
                sk_lock_cycle_sec: None,
                sk_lock_err_times: None,
                sk_lock_duration_sec: None,
                coexist_num: Some(1),
                conn_uri: Some(add_req.conn_uri.clone()),
                rel_rbum_domain_id: funs.iam_basic_domain_iam_id(),
                rel_rbum_item_id: Some(rel_iam_item_id.clone()),
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

    pub async fn get_or_add_account_with_verify(user_name: &str, password: &str, tenant_id: &str, funs: &TardisFunsInst) -> TardisResult<(String, String)> {
        let mut mock_ctx = TardisContext {
            own_paths: tenant_id.to_string(),
            ..Default::default()
        };
        let (mut ldap_client, cert_conf, cert_conf_id) = Self::get_ldap_client(tenant_id, funs, &mock_ctx).await?;
        let dn = if let Some(dn) = ldap_client.bind(user_name, password).await? {
            dn
        } else {
            ldap_client.unbind().await?;
            return Err(funs.err().unauthorized("rbum_cert", "get_or_add_account", "validation error", "401-rbum-cert-valid-error"));
        };
        if let Some(account_id) = Self::get_cert_rel_account_by_dn(&dn, &cert_conf_id, funs, &mock_ctx).await? {
            ldap_client.unbind().await?;
            return Ok((account_id, dn));
        }
        if !IamTenantServ::get_item(tenant_id, &IamTenantFilterReq::default(), funs, &mock_ctx).await?.account_self_reg {
            ldap_client.unbind().await?;
            return Err(funs.err().not_found(
                "rbum_cert",
                "get_or_add_account_with_verify",
                &format!("not found ldap cert(openid): {} and self-registration disabled", &dn),
                "401-rbum-cert-valid-error",
            ));
        }
        let account = ldap_client.get_by_dn(&dn, &vec!["dn", "cn", &cert_conf.field_display_name]).await?;
        ldap_client.unbind().await?;
        if let Some(account) = account {
            mock_ctx.owner = TardisFuns::field.nanoid();
            let account_id = Self::do_add_account(
                &account.dn,
                &account.get_simple_attr(&cert_conf.field_display_name).unwrap_or_else(|| "".to_string()),
                &cert_conf_id,
                funs,
                &mock_ctx,
            )
            .await?;
            Ok((account_id, dn))
        } else {
            return Err(funs.err().not_found(
                "rbum_cert",
                "get_or_add_account_without_verify",
                &format!("not found ldap cert(openid): {}", &dn),
                "401-rbum-cert-valid-error",
            ));
        }
    }

    pub async fn get_or_add_account_without_verify(dn: &str, tenant_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<(String, String)> {
        let cert_conf_id = IamCertServ::get_cert_conf_id_by_code(&IamCertExtKind::Ldap.to_string(), Some(tenant_id.to_string()), funs).await?;
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
            let account_id = Self::do_add_account(
                &account.dn,
                &account.get_simple_attr(&cert_conf.field_display_name).unwrap_or_else(|| "".to_string()),
                &cert_conf_id,
                funs,
                ctx,
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

    pub async fn search_accounts(user_or_display_name: &str, tenant_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Vec<IamAccountExtSysResp>> {
        let (mut ldap_client, cert_conf, _) = Self::get_ldap_client(tenant_id, funs, ctx).await?;
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

    async fn do_add_account(dn: &str, name: &str, cert_conf_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<String> {
        let account_id = IamAccountServ::add_account_agg(
            &IamAccountAggAddReq {
                id: Some(TrimString(ctx.owner.clone())),
                name: TrimString(name.to_string()),
                // TODO Auto match rule
                cert_user_name: TrimString(TardisFuns::field.nanoid_len(8).to_lowercase()),
                cert_password: TrimString(format!("{}Pw$", TardisFuns::field.nanoid_len(6))),
                cert_phone: None,
                cert_mail: None,
                role_ids: None,
                org_node_ids: None,
                scope_level: None,
                disabled: None,
                icon: None,
                exts: HashMap::new(),
            },
            funs,
            ctx,
        )
        .await?;
        Self::add_or_modify_cert(&IamCertLdapAddOrModifyReq { dn: TrimString(dn.to_string()) }, &account_id, cert_conf_id, funs, ctx).await?;
        Ok(account_id)
    }

    async fn get_ldap_client(tenant_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<(LdapClient, IamCertConfLdapResp, String)> {
        let cert_conf_id = IamCertServ::get_cert_conf_id_by_code(&IamCertExtKind::Ldap.to_string(), Some(tenant_id.to_string()), funs).await?;
        let cert_conf = Self::get_cert_conf(&cert_conf_id, funs, ctx).await?;
        let client = LdapClient::new(&cert_conf.conn_uri, cert_conf.is_tls, &cert_conf.base_dn).await?;
        Ok((client, cert_conf, cert_conf_id))
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
                // TODO
                .search(dn, Scope::Subtree, "", return_attr)
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
