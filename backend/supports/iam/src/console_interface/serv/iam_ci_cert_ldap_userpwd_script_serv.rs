use std::collections::HashMap;

use tardis::basic::dto::TardisContext;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::TardisFunsInst;

use bios_basic::rbum::dto::rbum_filer_dto::{RbumBasicFilterReq, RbumCertConfFilterReq, RbumCertFilterReq};
use bios_basic::rbum::rbum_enumeration::{RbumCertConfStatusKind, RbumCertRelKind, RbumCertStatusKind};
use bios_basic::rbum::serv::rbum_cert_serv::{RbumCertConfServ, RbumCertServ};
use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;

use crate::basic::dto::iam_cert_dto::{IamCertUserPwdAddReq, IamCiLdapBootstrapUserPwdItemResp};
use crate::basic::dto::iam_filer_dto::IamAccountFilterReq;
use crate::basic::serv::clients::iam_search_client::IamSearchClient;
use crate::basic::serv::iam_account_serv::IamAccountServ;
use crate::basic::serv::iam_cert_serv::IamCertServ;
use crate::basic::serv::iam_cert_user_pwd_serv::IamCertUserPwdServ;
use crate::iam_config::IamBasicConfigApi;
use crate::iam_enumeration::{IamCertExtKind, IamCertKernelKind};
use crate::integration::ldap::ldap_parser::extract_cn_from_dn;

pub struct IamCiCertLdapUserPwdScriptServ;

impl IamCiCertLdapUserPwdScriptServ {
    /// 为存在 LDAP 凭证、但不存在 UserPwd 凭证的账号生成随机默认密码并写入 UserPwd 证书。
    pub async fn bootstrap_userpwd_for_ldap_accounts_without(funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Vec<IamCiLdapBootstrapUserPwdItemResp>> {
        let ldap_conf_ids = Self::collect_ldap_cert_conf_ids(funs, ctx).await?;
        if ldap_conf_ids.is_empty() {
            return Ok(vec![]);
        }

        let ldap_certs = RbumCertServ::find_rbums(
            &RbumCertFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
                    own_paths: Some(ctx.own_paths.clone()),
                    ..Default::default()
                },
                rel_rbum_cert_conf_ids: Some(ldap_conf_ids),
                rel_rbum_kind: Some(RbumCertRelKind::Item),
                ..Default::default()
            },
            None,
            None,
            funs,
            ctx,
        )
        .await?;

        if ldap_certs.is_empty() {
            return Ok(vec![]);
        }

        let mut account_to_ldap_ak: HashMap<String, String> = HashMap::new();
        for cert in ldap_certs {
            account_to_ldap_ak.entry(cert.rel_rbum_id.clone()).or_insert(cert.ak.clone());
        }

        let mut results = vec![];
        for (account_id, ldap_ak) in account_to_ldap_ak {
            let account_ctx = IamAccountServ::is_global_account_context(account_id.as_str(), funs, ctx).await?;
            if IamCertServ::get_kernel_cert(account_id.as_str(), &IamCertKernelKind::UserPwd, funs, &account_ctx).await.is_ok() {
                continue;
            }

            let pwd_plain = IamCertServ::get_new_pwd();
            let ak_source = extract_cn_from_dn(ldap_ak.as_str()).unwrap_or_else(|| ldap_ak.clone());
            let ak = IamCertUserPwdServ::rename_ak_if_duplicate(ak_source.as_str(), funs, &account_ctx).await?;

            let userpwd_cert_conf_id = IamCertServ::get_cert_conf_id_by_kind(&IamCertKernelKind::UserPwd.to_string(), Some(account_ctx.own_paths.clone()), funs).await?;

            IamCertUserPwdServ::add_cert(
                &IamCertUserPwdAddReq {
                    ak: ak.clone(),
                    sk: TrimString(pwd_plain.clone()),
                    is_ignore_check_sk: true,
                    status: Some(RbumCertStatusKind::Enabled),
                },
                account_id.as_str(),
                Some(userpwd_cert_conf_id),
                funs,
                &account_ctx,
            )
            .await?;

            let _ = IamSearchClient::async_add_or_modify_account_search(account_id.as_str(), Box::new(false), "", funs, &account_ctx).await;

            let account = IamAccountServ::get_item(
                account_id.as_str(),
                &IamAccountFilterReq {
                    basic: RbumBasicFilterReq {
                        with_sub_own_paths: true,
                        own_paths: Some("".to_string()),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                funs,
                &account_ctx,
            )
            .await?;

            results.push(IamCiLdapBootstrapUserPwdItemResp {
                account_name: account.name,
                ak: ak.to_string(),
                password_plain: pwd_plain,
            });
        }

        Ok(results)
    }

    /// 仅 `own_paths` 为空的 LDAP cert_conf（全局配置）。
    async fn collect_ldap_cert_conf_ids(funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Vec<String>> {
        let filter = RbumCertConfFilterReq {
            basic: RbumBasicFilterReq {
                with_sub_own_paths: true,
                own_paths: Some("".to_string()),
                ..Default::default()
            },
            kind: Some(TrimString(IamCertExtKind::Ldap.to_string())),
            rel_rbum_domain_id: Some(funs.iam_basic_domain_iam_id()),
            status: Some(RbumCertConfStatusKind::Enabled),
            ..Default::default()
        };
        Ok(RbumCertConfServ::find_rbums(&filter, None, None, funs, ctx)
            .await?
            .into_iter()
            .map(|c| c.id)
            .collect())
    }
}
