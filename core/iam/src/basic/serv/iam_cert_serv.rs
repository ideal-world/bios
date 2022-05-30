use tardis::basic::dto::{TardisContext, TardisFunsInst};
use tardis::basic::error::TardisError;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::web::web_resp::TardisPage;
use tardis::TardisFuns;

use bios_basic::rbum::dto::rbum_cert_conf_dto::{RbumCertConfDetailResp, RbumCertConfSummaryResp};
use bios_basic::rbum::dto::rbum_cert_dto::RbumCertSummaryResp;
use bios_basic::rbum::dto::rbum_filer_dto::{RbumBasicFilterReq, RbumCertConfFilterReq, RbumCertFilterReq};
use bios_basic::rbum::dto::rbum_rel_dto::RbumRelBoneResp;
use bios_basic::rbum::helper::rbum_scope_helper;
use bios_basic::rbum::serv::rbum_cert_serv::{RbumCertConfServ, RbumCertServ};
use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;

use crate::basic::dto::iam_account_dto::{AccountAppInfoResp, AccountInfoResp};
use crate::basic::dto::iam_cert_conf_dto::{IamMailVCodeCertConfAddOrModifyReq, IamPhoneVCodeCertConfAddOrModifyReq, IamTokenCertConfAddReq, IamUserPwdCertConfAddOrModifyReq};
use crate::basic::dto::iam_filer_dto::{IamAccountFilterReq, IamAppFilterReq};
use crate::basic::serv::iam_account_serv::IamAccountServ;
use crate::basic::serv::iam_app_serv::IamAppServ;
use crate::basic::serv::iam_cert_mail_vcode_serv::IamCertMailVCodeServ;
use crate::basic::serv::iam_cert_phone_vcode_serv::IamCertPhoneVCodeServ;
use crate::basic::serv::iam_cert_token_serv::IamCertTokenServ;
use crate::basic::serv::iam_cert_user_pwd_serv::IamCertUserPwdServ;
use crate::basic::serv::iam_key_cache_serv::IamIdentCacheServ;
use crate::basic::serv::iam_role_serv::IamRoleServ;
use crate::basic::serv::iam_set_serv::IamSetServ;
use crate::iam_config::IamBasicInfoManager;
use crate::iam_constants;
use crate::iam_enumeration::{IamCertKind, IamCertTokenKind};

pub struct IamCertServ;

impl<'a> IamCertServ {
    pub fn get_new_pwd() -> String {
        TardisFuns::field.nanoid_len(10)
    }

    pub async fn init_default_ident_conf(
        user_pwd_cert_conf_add_req: IamUserPwdCertConfAddOrModifyReq,
        phone_vcode_cert_conf_add_req: Option<IamPhoneVCodeCertConfAddOrModifyReq>,
        mail_vcode_cert_conf_add_req: Option<IamMailVCodeCertConfAddOrModifyReq>,
        funs: &TardisFunsInst<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<String> {
        let rbum_cert_conf_user_pwd_id = IamCertUserPwdServ::add_cert_conf(&user_pwd_cert_conf_add_req, rbum_scope_helper::get_max_level_id_by_context(cxt), funs, cxt).await?;

        if let Some(phone_vcode_cert_conf_add_req) = phone_vcode_cert_conf_add_req {
            IamCertPhoneVCodeServ::add_cert_conf(&phone_vcode_cert_conf_add_req, rbum_scope_helper::get_max_level_id_by_context(cxt), funs, cxt).await?;
        }

        if let Some(mail_vcode_cert_conf_add_req) = mail_vcode_cert_conf_add_req {
            IamCertMailVCodeServ::add_cert_conf(&mail_vcode_cert_conf_add_req, rbum_scope_helper::get_max_level_id_by_context(cxt), funs, cxt).await?;
        }

        IamCertTokenServ::add_cert_conf(
            &IamTokenCertConfAddReq {
                name: TrimString(IamCertTokenKind::TokenDefault.to_string()),
                coexist_num: iam_constants::RBUM_CERT_CONF_TOKEN_DEFAULT_COEXIST_NUM,
                expire_sec: Some(iam_constants::RBUM_CERT_CONF_TOKEN_EXPIRE_SEC),
            },
            IamCertTokenKind::TokenDefault,
            rbum_scope_helper::get_max_level_id_by_context(cxt),
            funs,
            cxt,
        )
        .await?;

        IamCertTokenServ::add_cert_conf(
            &IamTokenCertConfAddReq {
                name: TrimString(IamCertTokenKind::TokenPc.to_string()),
                coexist_num: 1,
                expire_sec: Some(iam_constants::RBUM_CERT_CONF_TOKEN_EXPIRE_SEC),
            },
            IamCertTokenKind::TokenPc,
            rbum_scope_helper::get_max_level_id_by_context(cxt),
            funs,
            cxt,
        )
        .await?;

        IamCertTokenServ::add_cert_conf(
            &IamTokenCertConfAddReq {
                name: TrimString(IamCertTokenKind::TokenPhone.to_string()),
                coexist_num: 1,
                expire_sec: Some(iam_constants::RBUM_CERT_CONF_TOKEN_EXPIRE_SEC),
            },
            IamCertTokenKind::TokenPhone,
            rbum_scope_helper::get_max_level_id_by_context(cxt),
            funs,
            cxt,
        )
        .await?;

        IamCertTokenServ::add_cert_conf(
            &IamTokenCertConfAddReq {
                name: TrimString(IamCertTokenKind::TokenPad.to_string()),
                coexist_num: 1,
                expire_sec: Some(iam_constants::RBUM_CERT_CONF_TOKEN_EXPIRE_SEC),
            },
            IamCertTokenKind::TokenPad,
            rbum_scope_helper::get_max_level_id_by_context(cxt),
            funs,
            cxt,
        )
        .await?;

        Ok(rbum_cert_conf_user_pwd_id)
    }

    pub async fn get_cert_conf(id: &str, iam_item_id: Option<String>, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<RbumCertConfDetailResp> {
        RbumCertConfServ::get_rbum(
            id,
            &RbumCertConfFilterReq {
                rel_rbum_domain_id: Some(IamBasicInfoManager::get().domain_iam_id.to_string()),
                rel_rbum_item_id: iam_item_id,
                ..Default::default()
            },
            funs,
            cxt,
        )
        .await
    }

    pub async fn find_cert_conf_detail_without_token_kind(
        id: Option<String>,
        code: Option<String>,
        name: Option<String>,
        with_sub: Option<bool>,
        iam_item_id: Option<String>,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<Vec<RbumCertConfDetailResp>> {
        let result = RbumCertConfServ::find_detail_rbums(
            &RbumCertConfFilterReq {
                basic: RbumBasicFilterReq {
                    ids: id.map(|id| vec![id]),
                    code,
                    name,
                    with_sub_own_paths: with_sub.unwrap_or(false),
                    ..Default::default()
                },
                rel_rbum_domain_id: Some(IamBasicInfoManager::get().domain_iam_id.to_string()),
                rel_rbum_item_id: iam_item_id,
            },
            desc_sort_by_create,
            desc_sort_by_update,
            funs,
            cxt,
        )
        .await?;
        let result = result
            .into_iter()
            .filter(|r| r.code == IamCertKind::UserPwd.to_string() || r.code == IamCertKind::PhoneVCode.to_string() || r.code == IamCertKind::MailVCode.to_string())
            .collect();
        Ok(result)
    }

    pub async fn paginate_cert_conf(
        id: Option<String>,
        code: Option<String>,
        name: Option<String>,
        with_sub: Option<bool>,
        iam_item_id: Option<String>,
        page_number: u64,
        page_size: u64,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<TardisPage<RbumCertConfSummaryResp>> {
        RbumCertConfServ::paginate_rbums(
            &RbumCertConfFilterReq {
                basic: RbumBasicFilterReq {
                    ids: id.map(|id| vec![id]),
                    code,
                    name,
                    with_sub_own_paths: with_sub.unwrap_or(false),
                    ..Default::default()
                },
                rel_rbum_domain_id: Some(IamBasicInfoManager::get().domain_iam_id.to_string()),
                rel_rbum_item_id: iam_item_id,
            },
            page_number,
            page_size,
            desc_sort_by_create,
            desc_sort_by_update,
            funs,
            cxt,
        )
        .await
    }

    pub async fn delete_cert_conf(id: &str, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<u64> {
        let rbum_cert_conf = RbumCertConfServ::peek_rbum(
            id,
            &RbumCertConfFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                ..Default::default()
            },
            funs,
            cxt,
        )
        .await?;
        if rbum_cert_conf.code == IamCertKind::UserPwd.to_string() {
            return Err(TardisError::Conflict("Cannot delete default credential".to_string()));
        }
        RbumCertConfServ::delete_rbum(id, funs, cxt).await
    }

    pub async fn find_certs(
        filter: &RbumCertFilterReq,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<Vec<RbumCertSummaryResp>> {
        RbumCertServ::find_rbums(filter, desc_sort_by_create, desc_sort_by_update, funs, cxt).await
    }

    pub async fn delete_cert(id: &str, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<u64> {
        let cert = RbumCertServ::peek_rbum(
            id,
            &RbumCertFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                ..Default::default()
            },
            funs,
            cxt,
        )
        .await?;
        let result = RbumCertServ::delete_rbum(id, funs, cxt).await?;
        IamIdentCacheServ::delete_tokens_and_contexts_by_account_id(&cert.rel_rbum_id, funs).await?;
        Ok(result)
    }

    pub async fn get_cert_conf_id_by_code(code: &str, rel_iam_item_id: Option<String>, funs: &TardisFunsInst<'a>) -> TardisResult<String> {
        Self::get_cert_conf_id_opt_by_code(code, rel_iam_item_id, funs).await?.ok_or_else(|| TardisError::NotFound(format!("Cert Conf code {} not found", code)))
    }

    pub async fn get_cert_conf_id_opt_by_code(code: &str, rel_iam_item_id: Option<String>, funs: &TardisFunsInst<'a>) -> TardisResult<Option<String>> {
        RbumCertConfServ::get_rbum_cert_conf_id_by_code(
            code,
            &IamBasicInfoManager::get().domain_iam_id,
            rel_iam_item_id.unwrap_or_else(|| "".to_string()).as_str(),
            funs,
        )
        .await
    }

    pub async fn package_tardis_context_and_resp(
        tenant_id: Option<String>,
        ak: &str,
        account_id: &str,
        token_kind: Option<String>,
        funs: &TardisFunsInst<'a>,
    ) -> TardisResult<AccountInfoResp> {
        let token_kind = IamCertTokenKind::parse(&token_kind);
        let token = TardisFuns::crypto.key.generate_token()?;
        let tenant_id = if let Some(tenant_id) = tenant_id { tenant_id } else { "".to_string() };
        let context = TardisContext {
            own_paths: tenant_id.clone(),
            ak: ak.to_string(),
            owner: account_id.to_string(),
            roles: vec![],
            groups: vec![],
        };
        let rbum_cert_conf_id = Self::get_cert_conf_id_by_code(token_kind.to_string().as_str(), Some(tenant_id.clone()), funs).await?;
        IamCertTokenServ::add_cert(&token, &token_kind, account_id, &rbum_cert_conf_id, funs, &context).await?;

        let account_name = IamAccountServ::peek_item(account_id, &IamAccountFilterReq::default(), funs, &context).await?.name;
        let raw_roles = IamAccountServ::find_simple_rel_roles(account_id, true, Some(true), None, funs, &context).await?;
        let mut roles: Vec<RbumRelBoneResp> = vec![];
        for role in raw_roles {
            if !IamRoleServ::is_disabled(&role.rel_id, funs).await? {
                roles.push(role)
            }
        }

        let apps = if !tenant_id.is_empty() {
            let enabled_apps = IamAppServ::find_items(
                &IamAppFilterReq {
                    basic: RbumBasicFilterReq {
                        ignore_scope: false,
                        rel_cxt_owner: false,
                        with_sub_own_paths: true,
                        enabled: Some(true),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                None,
                None,
                funs,
                &context,
            )
            .await?;

            let mut apps: Vec<AccountAppInfoResp> = vec![];
            for app in enabled_apps {
                let set_id = IamSetServ::get_set_id_by_code(&IamSetServ::get_default_org_code_by_own_paths(&app.own_paths), funs, &context).await?;
                let groups = IamSetServ::find_flat_set_items(&set_id, &context.owner, funs, &context).await?;
                apps.push(AccountAppInfoResp {
                    app_id: app.id,
                    app_name: app.name,
                    roles: roles.iter().filter(|r| r.rel_own_paths == app.own_paths).map(|r| (r.rel_id.to_string(), r.rel_name.to_string())).collect(),
                    groups,
                });
            }
            apps
        } else {
            vec![]
        };

        let set_id = IamSetServ::get_set_id_by_code(&IamSetServ::get_default_org_code_by_own_paths(&context.own_paths), funs, &context).await?;
        let groups = IamSetServ::find_flat_set_items(&set_id, &context.owner, funs, &context).await?;
        let account_info = AccountInfoResp {
            account_id: account_id.to_string(),
            account_name: account_name.to_string(),
            token,
            roles: roles.iter().filter(|r| r.rel_own_paths == context.own_paths).map(|r| (r.rel_id.to_string(), r.rel_name.to_string())).collect(),
            groups,
            apps,
        };

        IamIdentCacheServ::add_contexts(&account_info, ak, &tenant_id, funs).await?;

        Ok(account_info)
    }

    pub fn try_use_tenant_ctx(cxt: TardisContext, tenant_id: Option<String>) -> TardisResult<TardisContext> {
        if let Some(tenant_id) = &tenant_id {
            Self::use_tenant_ctx(cxt, tenant_id)
        } else {
            Ok(cxt)
        }
    }

    pub fn use_tenant_ctx(cxt: TardisContext, tenant_id: &str) -> TardisResult<TardisContext> {
        rbum_scope_helper::degrade_own_paths(cxt, tenant_id.to_string().as_str())
    }

    pub fn try_use_app_ctx(cxt: TardisContext, app_id: Option<String>) -> TardisResult<TardisContext> {
        if let Some(app_id) = &app_id {
            Self::use_app_ctx(cxt, app_id)
        } else {
            Ok(cxt)
        }
    }

    pub fn use_app_ctx(cxt: TardisContext, app_id: &str) -> TardisResult<TardisContext> {
        let own_paths = cxt.own_paths.clone();
        rbum_scope_helper::degrade_own_paths(cxt, format!("{}/{}", own_paths, app_id).as_str())
    }

    pub fn get_anonymous_context() -> TardisContext {
        TardisContext {
            own_paths: "_/_/_/_/_/_".to_string(),
            ak: "".to_string(),
            owner: "".to_string(),
            roles: vec![],
            groups: vec![],
        }
    }
}
