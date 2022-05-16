use tardis::basic::dto::{TardisContext, TardisFunsInst};
use tardis::basic::error::TardisError;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::web::web_resp::TardisPage;
use tardis::TardisFuns;

use bios_basic::rbum::dto::rbum_cert_conf_dto::{RbumCertConfDetailResp, RbumCertConfSummaryResp};
use bios_basic::rbum::dto::rbum_filer_dto::{RbumBasicFilterReq, RbumCertConfFilterReq};
use bios_basic::rbum::helper::rbum_scope_helper;
use bios_basic::rbum::serv::rbum_cert_serv::{RbumCertConfServ, RbumCertServ};
use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;

use crate::basic::dto::iam_account_dto::{AccountAppInfoResp, AccountInfoResp};
use crate::basic::dto::iam_cert_conf_dto::{IamMailVCodeCertConfAddOrModifyReq, IamPhoneVCodeCertConfAddOrModifyReq, IamTokenCertConfAddReq, IamUserPwdCertConfAddOrModifyReq};
use crate::basic::dto::iam_cert_dto::IamContextFetchReq;
use crate::basic::dto::iam_filer_dto::{IamAccountFilterReq, IamAppFilterReq};
use crate::basic::serv::iam_account_serv::IamAccountServ;
use crate::basic::serv::iam_app_serv::IamAppServ;
use crate::basic::serv::iam_cert_mail_vcode_serv::IamCertMailVCodeServ;
use crate::basic::serv::iam_cert_phone_vcode_serv::IamCertPhoneVCodeServ;
use crate::basic::serv::iam_cert_token_serv::IamCertTokenServ;
use crate::basic::serv::iam_cert_user_pwd_serv::IamCertUserPwdServ;
use crate::basic::serv::iam_rel_serv::IamRelServ;
use crate::iam_config::{IamBasicInfoManager, IamConfig};
use crate::iam_constants;
use crate::iam_enumeration::{IamCertKind, IamCertTokenKind, IamRelKind};

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
        let rbum_cert_conf = RbumCertConfServ::get_rbum(
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

    pub async fn delete_cert(id: &str, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<u64> {
        RbumCertServ::delete_rbum(id, funs, cxt).await
    }

    pub async fn get_cert_conf_id_by_code(code: &str, rel_iam_item_id: Option<String>, funs: &TardisFunsInst<'a>) -> TardisResult<String> {
        Self::get_cert_conf_id_opt_by_code(code, rel_iam_item_id, funs).await?.ok_or_else(|| TardisError::NotFound(format!("cert config code {} not found", code)))
    }

    pub async fn get_cert_conf_id_opt_by_code(code: &str, rel_iam_item_id: Option<String>, funs: &TardisFunsInst<'a>) -> TardisResult<Option<String>> {
        RbumCertConfServ::get_rbum_cert_conf_id_by_code(code, &IamBasicInfoManager::get().domain_iam_id, rel_iam_item_id.unwrap_or("".to_string()).as_str(), funs).await
    }

    pub async fn package_tardis_context_and_resp(
        tenant_id: Option<String>,
        ak: &str,
        account_id: &str,
        rbum_cert_id: &str,
        token_kind: Option<String>,
        funs: &TardisFunsInst<'a>,
    ) -> TardisResult<AccountInfoResp> {
        let token_kind = IamCertTokenKind::parse(&token_kind);
        let tenant_id = if let Some(tenant_id) = tenant_id { tenant_id } else { "".to_string() };
        let context = TardisContext {
            own_paths: tenant_id.clone(),
            ak: ak.to_string(),
            owner: account_id.to_string(),
            token: TardisFuns::crypto.key.generate_token()?,
            token_kind: token_kind.to_string(),
            roles: vec![],
            groups: vec![],
        };
        let rbum_cert_conf_id = Self::get_cert_conf_id_by_code(token_kind.to_string().as_str(), Some(tenant_id.clone()), funs).await?;
        IamCertTokenServ::add_cert(&context.token, &token_kind, account_id, &rbum_cert_conf_id, rbum_cert_id, funs, &context).await?;

        let account_name = IamAccountServ::get_item(account_id, &IamAccountFilterReq::default(), funs, &context).await?.name;
        let roles = IamRelServ::find_from_rels(IamRelKind::IamAccountRole, true, account_id, Some(true), None, funs, &context).await?;

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
            enabled_apps
                .into_iter()
                .map(|app| {
                    AccountAppInfoResp {
                        app_id: app.id,
                        app_name: app.name,
                        roles: roles
                            .iter()
                            .filter(|r| r.rel.own_paths == app.own_paths)
                            .map(|r| (r.rel.to_rbum_item_id.to_string(), r.rel.to_rbum_item_name.to_string()))
                            .collect(),
                        // TODO
                        groups: Default::default(),
                    }
                })
                .collect()
        } else {
            vec![]
        };

        let account_info = AccountInfoResp {
            account_id: account_id.to_string(),
            account_name: account_name.to_string(),
            token: context.token.to_string(),
            roles: roles.iter().filter(|r| r.rel.own_paths == context.own_paths).map(|r| (r.rel.to_rbum_item_id.to_string(), r.rel.to_rbum_item_name.to_string())).collect(),
            groups: Default::default(),
            apps,
        };

        Self::add_cached_contexts(&account_info, ak, &token_kind.to_string(), &tenant_id, funs).await?;

        Ok(account_info)
    }

    pub async fn add_cached_contexts(account_info: &AccountInfoResp, ak: &str, token_kind: &str, tenant_id: &str, funs: &TardisFunsInst<'a>) -> TardisResult<()> {
        funs.cache()
            .hset(
                format!("{}{}", funs.conf::<IamConfig>().cache_key_account_info_, account_info.account_id).as_str(),
                "",
                &TardisFuns::json.obj_to_string(&TardisContext {
                    own_paths: tenant_id.to_string(),
                    ak: ak.to_string(),
                    owner: account_info.account_id.to_string(),
                    token: account_info.token.to_string(),
                    token_kind: token_kind.to_string(),
                    roles: account_info.roles.iter().map(|(id, _)| id.to_string()).collect(),
                    groups: account_info.groups.iter().map(|(id, _)| id.to_string()).collect(),
                })?,
            )
            .await?;
        for account_app_info in &account_info.apps {
            funs.cache()
                .hset(
                    format!("{}{}", funs.conf::<IamConfig>().cache_key_account_info_, account_info.account_id).as_str(),
                    &account_app_info.app_id,
                    &TardisFuns::json.obj_to_string(&TardisContext {
                        own_paths: format!("{}/{}", tenant_id, account_app_info.app_id).to_string(),
                        ak: ak.to_string(),
                        owner: account_info.account_id.to_string(),
                        token: account_info.token.to_string(),
                        token_kind: token_kind.to_string(),
                        roles: account_app_info.roles.iter().map(|(id, _)| id.to_string()).collect(),
                        groups: account_app_info.groups.iter().map(|(id, _)| id.to_string()).collect(),
                    })?,
                )
                .await?;
        }
        Ok(())
    }

    pub async fn fetch_context(fetch_req: &IamContextFetchReq, funs: &TardisFunsInst<'a>) -> TardisResult<TardisContext> {
        if let Some(token_info) = funs.cache().get(format!("{}{}", funs.conf::<IamConfig>().cache_key_token_info_, &fetch_req.token).as_str()).await? {
            let account_id = token_info.split(",").nth(1).unwrap_or("");
            if let Some(context) = funs
                .cache()
                .hget(
                    format!("{}{}", funs.conf::<IamConfig>().cache_key_account_info_, account_id).as_str(),
                    fetch_req.app_id.as_ref().unwrap_or(&"".to_string()),
                )
                .await?
            {
                return TardisFuns::json.str_to_obj(&context);
            }
        }
        Err(TardisError::NotFound("context not found".to_string()))
    }

    pub fn use_tenant_ctx(cxt: TardisContext, tenant_id: &str) -> TardisResult<TardisContext> {
        Self::degrade_own_paths(cxt, tenant_id.to_string().as_str())
    }

    pub fn use_app_ctx(cxt: TardisContext, app_id: &str) -> TardisResult<TardisContext> {
        let own_paths = cxt.own_paths.clone();
        Self::degrade_own_paths(cxt, format!("{}/{}", own_paths, app_id).as_str())
    }

    fn degrade_own_paths(mut cxt: TardisContext, new_own_paths: &str) -> TardisResult<TardisContext> {
        if !new_own_paths.contains(&cxt.own_paths) {
            return Err(TardisError::Conflict("Not qualified for downgrade".to_string()));
        }
        cxt.own_paths = new_own_paths.to_string();
        Ok(cxt)
    }

    pub fn get_anonymous_context() -> TardisContext {
        TardisContext {
            own_paths: "_/_/_/_/_/_".to_string(),
            ak: "".to_string(),
            owner: "".to_string(),
            token: "".to_string(),
            token_kind: "".to_string(),
            roles: vec![],
            groups: vec![],
        }
    }
}
