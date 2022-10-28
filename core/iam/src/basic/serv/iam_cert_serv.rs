use bios_basic::rbum::dto::rbum_rel_agg_dto::RbumRelAggAddReq;
use bios_basic::rbum::serv::rbum_rel_serv::RbumRelServ;
use tardis::basic::dto::TardisContext;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::web::web_resp::TardisPage;
use tardis::{TardisFuns, TardisFunsInst};

use bios_basic::rbum::dto::rbum_cert_conf_dto::{RbumCertConfAddReq, RbumCertConfDetailResp, RbumCertConfIdAndExtResp, RbumCertConfSummaryResp};
use bios_basic::rbum::dto::rbum_cert_dto::{RbumCertAddReq, RbumCertDetailResp, RbumCertModifyReq, RbumCertSummaryResp, RbumCertSummaryWithSkResp};
use bios_basic::rbum::dto::rbum_filer_dto::{RbumBasicFilterReq, RbumCertConfFilterReq, RbumCertFilterReq, RbumRelFilterReq};
use bios_basic::rbum::dto::rbum_rel_dto::{RbumRelAddReq, RbumRelBoneResp};
use bios_basic::rbum::helper::rbum_scope_helper;
use bios_basic::rbum::rbum_enumeration::{RbumCertRelKind, RbumCertStatusKind, RbumRelFromKind};
use bios_basic::rbum::serv::rbum_cert_serv::{RbumCertConfServ, RbumCertServ};
use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;

use crate::basic::dto::iam_account_dto::IamAccountInfoResp;
use crate::basic::dto::iam_cert_conf_dto::{
    IamCertConfLdapAddOrModifyReq, IamCertConfMailVCodeAddOrModifyReq, IamCertConfPhoneVCodeAddOrModifyReq, IamCertConfTokenAddReq, IamCertConfUserPwdAddOrModifyReq,
};
use crate::basic::dto::iam_cert_dto::{IamCertExtAddReq, IamCertManageAddReq, IamCertManageModifyReq};
use crate::basic::dto::iam_filer_dto::IamAccountFilterReq;
use crate::basic::serv::iam_account_serv::IamAccountServ;
use crate::basic::serv::iam_cert_ldap_serv::IamCertLdapServ;
use crate::basic::serv::iam_cert_mail_vcode_serv::IamCertMailVCodeServ;
use crate::basic::serv::iam_cert_phone_vcode_serv::IamCertPhoneVCodeServ;
use crate::basic::serv::iam_cert_token_serv::IamCertTokenServ;
use crate::basic::serv::iam_cert_user_pwd_serv::IamCertUserPwdServ;
use crate::basic::serv::iam_key_cache_serv::IamIdentCacheServ;
use crate::iam_config::IamBasicConfigApi;
use crate::iam_constants::{self, RBUM_SCOPE_LEVEL_TENANT};
use crate::iam_enumeration::{IamCertExtKind, IamCertKernelKind, IamCertManageKind, IamCertTokenKind, IamRelKind};

use super::iam_rel_serv::IamRelServ;

pub struct IamCertServ;

impl IamCertServ {
    pub fn get_new_pwd() -> String {
        TardisFuns::field.nanoid_len(10)
    }

    pub async fn init_default_ident_conf(
        user_pwd_cert_conf_add_req: &IamCertConfUserPwdAddOrModifyReq,
        phone_vcode_cert_conf_add_req: Option<IamCertConfPhoneVCodeAddOrModifyReq>,
        mail_vcode_cert_conf_add_req: Option<IamCertConfMailVCodeAddOrModifyReq>,
        ldap_cert_conf_add_req: Option<Vec<IamCertConfLdapAddOrModifyReq>>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<String> {
        let rbum_cert_conf_user_pwd_id = IamCertUserPwdServ::add_cert_conf(user_pwd_cert_conf_add_req, rbum_scope_helper::get_max_level_id_by_context(ctx), funs, ctx).await?;

        if let Some(phone_vcode_cert_conf_add_req) = phone_vcode_cert_conf_add_req {
            IamCertPhoneVCodeServ::add_cert_conf(&phone_vcode_cert_conf_add_req, rbum_scope_helper::get_max_level_id_by_context(ctx), funs, ctx).await?;
        }

        if let Some(mail_vcode_cert_conf_add_req) = mail_vcode_cert_conf_add_req {
            IamCertMailVCodeServ::add_cert_conf(&mail_vcode_cert_conf_add_req, rbum_scope_helper::get_max_level_id_by_context(ctx), funs, ctx).await?;
        }

        if let Some(ldap_cert_conf_add_req) = ldap_cert_conf_add_req {
            if !ldap_cert_conf_add_req.is_empty() {
                for add_req in ldap_cert_conf_add_req {
                    let _ = IamCertLdapServ::add_cert_conf(&add_req, None, funs, ctx).await?;
                }
            }
        }

        IamCertTokenServ::add_cert_conf(
            &IamCertConfTokenAddReq {
                name: TrimString(IamCertTokenKind::TokenDefault.to_string()),
                coexist_num: iam_constants::RBUM_CERT_CONF_TOKEN_DEFAULT_COEXIST_NUM,
                expire_sec: Some(iam_constants::RBUM_CERT_CONF_TOKEN_EXPIRE_SEC),
            },
            IamCertTokenKind::TokenDefault,
            rbum_scope_helper::get_max_level_id_by_context(ctx),
            funs,
            ctx,
        )
        .await?;

        IamCertTokenServ::add_cert_conf(
            &IamCertConfTokenAddReq {
                name: TrimString(IamCertTokenKind::TokenPc.to_string()),
                coexist_num: 1,
                expire_sec: Some(iam_constants::RBUM_CERT_CONF_TOKEN_EXPIRE_SEC),
            },
            IamCertTokenKind::TokenPc,
            rbum_scope_helper::get_max_level_id_by_context(ctx),
            funs,
            ctx,
        )
        .await?;

        IamCertTokenServ::add_cert_conf(
            &IamCertConfTokenAddReq {
                name: TrimString(IamCertTokenKind::TokenPhone.to_string()),
                coexist_num: 1,
                expire_sec: Some(iam_constants::RBUM_CERT_CONF_TOKEN_EXPIRE_SEC),
            },
            IamCertTokenKind::TokenPhone,
            rbum_scope_helper::get_max_level_id_by_context(ctx),
            funs,
            ctx,
        )
        .await?;

        IamCertTokenServ::add_cert_conf(
            &IamCertConfTokenAddReq {
                name: TrimString(IamCertTokenKind::TokenPad.to_string()),
                coexist_num: 1,
                expire_sec: Some(iam_constants::RBUM_CERT_CONF_TOKEN_EXPIRE_SEC),
            },
            IamCertTokenKind::TokenPad,
            rbum_scope_helper::get_max_level_id_by_context(ctx),
            funs,
            ctx,
        )
        .await?;
        Ok(rbum_cert_conf_user_pwd_id)
    }

    #[deprecated]
    pub async fn init_default_ext_conf(funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        Self::add_ext_cert_conf(&IamCertExtKind::Gitlab, rbum_scope_helper::get_max_level_id_by_context(ctx), funs, ctx).await?;
        Self::add_ext_cert_conf(&IamCertExtKind::Github, rbum_scope_helper::get_max_level_id_by_context(ctx), funs, ctx).await?;
        Ok(())
    }

    #[deprecated = "name needs consideration"]
    pub async fn init_default_manage_conf(funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        Self::add_manage_cert_conf(&IamCertManageKind::ManageUserPwd, rbum_scope_helper::get_max_level_id_by_context(ctx), funs, ctx).await?;
        Self::add_manage_cert_conf(&IamCertManageKind::ManageUserVisa, rbum_scope_helper::get_max_level_id_by_context(ctx), funs, ctx).await?;
        Ok(())
    }

    pub async fn get_cert_conf(id: &str, iam_item_id: Option<String>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<RbumCertConfDetailResp> {
        RbumCertConfServ::get_rbum(
            id,
            &RbumCertConfFilterReq {
                rel_rbum_domain_id: Some(funs.iam_basic_domain_iam_id()),
                rel_rbum_item_id: iam_item_id,
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await
    }

    pub async fn find_cert_conf(
        with_sub: bool,
        iam_item_id: Option<String>,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<Vec<RbumCertConfSummaryResp>> {
        RbumCertConfServ::find_rbums(
            &RbumCertConfFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: with_sub,
                    ..Default::default()
                },
                rel_rbum_domain_id: Some(funs.iam_basic_domain_iam_id()),
                rel_rbum_item_id: iam_item_id,
            },
            desc_sort_by_create,
            desc_sort_by_update,
            funs,
            ctx,
        )
        .await
    }

    pub async fn find_cert_conf_detail_with_kernel_kind(
        id: Option<String>,
        code: Option<String>,
        name: Option<String>,
        with_sub: bool,
        iam_item_id: Option<String>,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<Vec<RbumCertConfDetailResp>> {
        let result = RbumCertConfServ::find_detail_rbums(
            &RbumCertConfFilterReq {
                basic: RbumBasicFilterReq {
                    ids: id.map(|id| vec![id]),
                    code,
                    name,
                    with_sub_own_paths: with_sub,
                    ..Default::default()
                },
                rel_rbum_domain_id: Some(funs.iam_basic_domain_iam_id()),
                rel_rbum_item_id: iam_item_id,
            },
            desc_sort_by_create,
            desc_sort_by_update,
            funs,
            ctx,
        )
        .await?;
        let result = result
            .into_iter()
            .filter(|r| {
                r.code == IamCertKernelKind::UserPwd.to_string() || r.code == IamCertKernelKind::PhoneVCode.to_string() || r.code == IamCertKernelKind::MailVCode.to_string()
            })
            .collect();
        Ok(result)
    }

    pub async fn get_kernel_cert(account_id: &str, rel_iam_cert_kind: &IamCertKernelKind, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<RbumCertSummaryWithSkResp> {
        let rel_rbum_cert_conf_id = &Self::get_cert_conf_id_by_code(rel_iam_cert_kind.to_string().as_str(), rbum_scope_helper::get_max_level_id_by_context(ctx), funs).await?;
        let kernel_cert = RbumCertServ::find_one_rbum(
            &RbumCertFilterReq {
                rel_rbum_id: Some(account_id.to_string()),
                rel_rbum_cert_conf_ids: Some(vec![rel_rbum_cert_conf_id.to_string()]),
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?;
        if let Some(kernel_cert) = kernel_cert {
            let now_sk = RbumCertServ::show_sk(kernel_cert.id.as_str(), &RbumCertFilterReq::default(), funs, ctx).await?;
            Ok(RbumCertSummaryWithSkResp {
                id: kernel_cert.id,
                ak: kernel_cert.ak,
                sk: now_sk,
                ext: kernel_cert.ext,
                start_time: kernel_cert.start_time,
                end_time: kernel_cert.end_time,
                status: kernel_cert.status,
                rel_rbum_cert_conf_id: kernel_cert.rel_rbum_cert_conf_id,
                rel_rbum_cert_conf_name: kernel_cert.rel_rbum_cert_conf_name,
                rel_rbum_cert_conf_code: kernel_cert.rel_rbum_cert_conf_code,
                rel_rbum_kind: kernel_cert.rel_rbum_kind,
                rel_rbum_id: kernel_cert.rel_rbum_id,
                own_paths: kernel_cert.own_paths,
                owner: kernel_cert.owner,
                create_time: kernel_cert.create_time,
                update_time: kernel_cert.update_time,
            })
        } else {
            Err(funs.err().not_found(
                "iam_cert",
                "get_kernel_cert",
                &format!("not found credential of kind {:?}", rel_iam_cert_kind),
                "404-iam-cert-kind-not-exist",
            ))
        }
    }

    pub async fn paginate_cert_conf(
        id: Option<String>,
        code: Option<String>,
        name: Option<String>,
        with_sub: bool,
        iam_item_id: Option<String>,
        page_number: u64,
        page_size: u64,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<TardisPage<RbumCertConfSummaryResp>> {
        RbumCertConfServ::paginate_rbums(
            &RbumCertConfFilterReq {
                basic: RbumBasicFilterReq {
                    ids: id.map(|id| vec![id]),
                    code,
                    name,
                    with_sub_own_paths: with_sub,
                    ..Default::default()
                },
                rel_rbum_domain_id: Some(funs.iam_basic_domain_iam_id()),
                rel_rbum_item_id: iam_item_id,
            },
            page_number,
            page_size,
            desc_sort_by_create,
            desc_sort_by_update,
            funs,
            ctx,
        )
        .await
    }

    pub async fn delete_cert_conf(id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<u64> {
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
            ctx,
        )
        .await?;
        if rbum_cert_conf.code == IamCertKernelKind::UserPwd.to_string() {
            return Err(funs.err().conflict("iam_cert_conf", "delete", "can not delete default credential", "409-rbum-cert-conf-basic-delete"));
        }
        let result = RbumCertConfServ::delete_rbum(id, funs, ctx).await?;
        Self::clean_cache_by_cert_conf(id, Some(rbum_cert_conf), funs, ctx).await?;
        Ok(result)
    }

    pub async fn clean_cache_by_cert_conf(id: &str, fetched_cert_conf: Option<RbumCertConfSummaryResp>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let rbum_cert_conf = if let Some(rbum_cert_conf) = fetched_cert_conf {
            rbum_cert_conf
        } else {
            RbumCertConfServ::peek_rbum(
                id,
                &RbumCertConfFilterReq {
                    basic: RbumBasicFilterReq {
                        with_sub_own_paths: true,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                funs,
                ctx,
            )
            .await?
        };
        if rbum_cert_conf.code == IamCertKernelKind::UserPwd.to_string()
            || rbum_cert_conf.code == IamCertKernelKind::MailVCode.to_string()
            || rbum_cert_conf.code == IamCertKernelKind::PhoneVCode.to_string()
        {
            IamIdentCacheServ::delete_tokens_and_contexts_by_tenant_or_app(&rbum_cert_conf.rel_rbum_item_id, false, funs, ctx).await?;
        }
        Ok(())
    }

    pub async fn add_manage_cert_conf(rel_iam_cert_kind: &IamCertManageKind, rel_iam_item_id: Option<String>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<String> {
        let id = RbumCertConfServ::add_rbum(
            &mut RbumCertConfAddReq {
                code: TrimString(rel_iam_cert_kind.to_string()),
                name: TrimString(rel_iam_cert_kind.to_string()),
                note: None,
                ak_note: None,
                ak_rule: None,
                sk_note: None,
                sk_rule: None,
                ext: None,
                sk_need: Some(false),
                sk_dynamic: None,
                sk_encrypted: Some(false),
                repeatable: None,
                is_basic: Some(false),
                is_ak_repeatable: Some(true),
                rest_by_kinds: None,
                expire_sec: None,
                sk_lock_cycle_sec: None,
                sk_lock_err_times: None,
                sk_lock_duration_sec: None,
                coexist_num: Some(1),
                conn_uri: None,
                rel_rbum_domain_id: funs.iam_basic_domain_iam_id(),
                rel_rbum_item_id: rel_iam_item_id,
            },
            funs,
            ctx,
        )
        .await?;
        Ok(id)
    }

    pub async fn add_manage_cert(add_req: &IamCertManageAddReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<String> {
        let id = RbumCertServ::add_rbum(
            &mut RbumCertAddReq {
                ak: TrimString(add_req.ak.trim().to_string()),
                sk: add_req.sk.as_ref().map(|sk| TrimString(sk.trim().to_string())),
                vcode: None,
                ext: Some(add_req.ext.as_ref().unwrap().to_string()),
                start_time: None,
                end_time: None,
                conn_uri: None,
                status: RbumCertStatusKind::Enabled,
                rel_rbum_cert_conf_id: Some(add_req.rel_rbum_cert_conf_id.as_ref().unwrap().to_string()),
                rel_rbum_kind: RbumCertRelKind::Item,
                rel_rbum_id: ctx.own_paths.to_string(),
                is_outside: true,
            },
            funs,
            ctx,
        )
        .await?;
        Ok(id)
    }

    pub async fn modify_manage_cert(id: &str, modify_req: &IamCertManageModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        RbumCertServ::modify_rbum(
            id,
            &mut RbumCertModifyReq {
                ext: modify_req.ext.clone(),
                ak: Some(TrimString(modify_req.ak.trim().to_string())),
                sk: Some(TrimString(modify_req.sk.as_ref().unwrap().to_string())),
                start_time: None,
                end_time: None,
                conn_uri: None,
                status: None,
            },
            funs,
            ctx,
        )
        .await?;
        Ok(())
    }

    pub async fn modify_manage_cert_ext(id: &str, ext: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        RbumCertServ::modify_rbum(
            id,
            &mut RbumCertModifyReq {
                ext: Some(ext.to_string()),
                ak: None,
                sk: None,
                start_time: None,
                end_time: None,
                conn_uri: None,
                status: None,
            },
            funs,
            ctx,
        )
        .await?;
        Ok(())
    }

    pub async fn delete_manage_cert(id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        RbumCertServ::delete_rbum(id, funs, ctx).await?;
        Ok(())
    }

    pub async fn get_manage_cert(id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<RbumCertSummaryWithSkResp> {
        if IamRelServ::find_rels(
            &RbumRelFilterReq {
                basic: RbumBasicFilterReq {
                    own_paths: Some("".to_string()),
                    with_sub_own_paths: true,
                    ignore_scope: true,
                    ..Default::default()
                },
                tag: Some(IamRelKind::IamCertRel.to_string()),
                from_rbum_id: Some(id.to_string()),
                ..Default::default()
            },
            None,
            None,
            funs,
            ctx,
        )
        .await?
        .into_iter()
        .filter(|x| x.rel.own_paths.contains(&ctx.own_paths.clone()) || x.rel.to_own_paths.contains(&ctx.own_paths.clone()))
        .next()
        .is_none()
        {
            return Err(funs.err().conflict("iam_cert", "get_manage_cert", "associated already exists", "409-rbum-rel-exist"));
        }
        let manage_cert = RbumCertServ::get_rbum(
            id,
            &RbumCertFilterReq {
                basic: RbumBasicFilterReq {
                    own_paths: Some("".to_string()),
                    with_sub_own_paths: true,
                    ignore_scope: true,
                    ..Default::default()
                },
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?;
        let manage_user_pwd_conf_id = &Self::get_cert_conf_id_by_code(IamCertManageKind::ManageUserPwd.to_string().as_str(), Some(manage_cert.own_paths.clone()), funs).await?;
        let manage_user_visa_conf_id = &Self::get_cert_conf_id_by_code(IamCertManageKind::ManageUserVisa.to_string().as_str(), Some(manage_cert.own_paths.clone()), funs).await?;
        if !(manage_cert.rel_rbum_cert_conf_id == Some(manage_user_pwd_conf_id.to_string()) || manage_cert.rel_rbum_cert_conf_id == Some(manage_user_visa_conf_id.to_string())) {
            return Err(funs.err().conflict("iam_cert", "get_manage_cert", "associated already exists", "409-rbum-rel-exist"));
        }
        let mut mock_ctx = ctx.clone();
        mock_ctx.own_paths = manage_cert.own_paths.clone();
        let now_sk = RbumCertServ::show_sk(manage_cert.id.as_str(), &RbumCertFilterReq::default(), funs, &mock_ctx).await?;
        Ok(RbumCertSummaryWithSkResp {
            id: manage_cert.id,
            ak: manage_cert.ak,
            sk: now_sk,
            ext: manage_cert.ext,
            start_time: manage_cert.start_time,
            end_time: manage_cert.end_time,
            status: manage_cert.status,
            rel_rbum_cert_conf_id: manage_cert.rel_rbum_cert_conf_id,
            rel_rbum_cert_conf_name: manage_cert.rel_rbum_cert_conf_name,
            rel_rbum_cert_conf_code: manage_cert.rel_rbum_cert_conf_code,
            rel_rbum_kind: manage_cert.rel_rbum_kind,
            rel_rbum_id: manage_cert.rel_rbum_id,
            own_paths: manage_cert.own_paths,
            owner: manage_cert.owner,
            create_time: manage_cert.create_time,
            update_time: manage_cert.update_time,
        })
    }

    pub async fn add_ext_cert_conf(rel_iam_cert_kind: &IamCertExtKind, rel_iam_item_id: Option<String>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<String> {
        let id = RbumCertConfServ::add_rbum(
            &mut RbumCertConfAddReq {
                code: TrimString(rel_iam_cert_kind.to_string()),
                name: TrimString(rel_iam_cert_kind.to_string()),
                note: None,
                ak_note: None,
                ak_rule: None,
                sk_note: None,
                sk_rule: None,
                ext: None,
                sk_need: Some(false),
                sk_dynamic: None,
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
                conn_uri: None,
                rel_rbum_domain_id: funs.iam_basic_domain_iam_id(),
                rel_rbum_item_id: rel_iam_item_id,
            },
            funs,
            ctx,
        )
        .await?;
        Ok(id)
    }

    pub async fn add_ext_cert(
        add_req: &mut IamCertExtAddReq,
        account_id: &str,
        rel_iam_cert_kind: &IamCertExtKind,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<String> {
        let rel_rbum_cert_conf_id = &Self::get_cert_conf_id_by_code(rel_iam_cert_kind.to_string().as_str(), rbum_scope_helper::get_max_level_id_by_context(ctx), funs).await?;
        let id = RbumCertServ::add_rbum(
            &mut RbumCertAddReq {
                ak: TrimString(add_req.ak.trim().to_string()),
                sk: add_req.sk.as_ref().map(|sk| TrimString(sk.trim().to_string())),
                vcode: None,
                ext: add_req.ext.clone(),
                start_time: None,
                end_time: None,
                conn_uri: None,
                status: RbumCertStatusKind::Enabled,
                rel_rbum_cert_conf_id: Some(rel_rbum_cert_conf_id.to_string()),
                rel_rbum_kind: RbumCertRelKind::Item,
                rel_rbum_id: account_id.to_string(),
                is_outside: true,
            },
            funs,
            ctx,
        )
        .await?;
        Ok(id)
    }

    pub async fn get_ext_cert(account_id: &str, rel_iam_cert_kind: &IamCertExtKind, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<RbumCertSummaryWithSkResp> {
        let rel_rbum_cert_conf_id = &Self::get_cert_conf_id_by_code(rel_iam_cert_kind.to_string().as_str(), rbum_scope_helper::get_max_level_id_by_context(ctx), funs).await?;
        let ext_cert = RbumCertServ::find_one_rbum(
            &RbumCertFilterReq {
                rel_rbum_id: Some(account_id.to_string()),
                rel_rbum_cert_conf_ids: Some(vec![rel_rbum_cert_conf_id.to_string()]),
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?;
        if let Some(ext_cert) = ext_cert {
            let now_sk = RbumCertServ::show_sk(ext_cert.id.as_str(), &RbumCertFilterReq::default(), funs, ctx).await?;
            Ok(RbumCertSummaryWithSkResp {
                id: ext_cert.id,
                ak: ext_cert.ak,
                sk: now_sk,
                ext: ext_cert.ext,
                start_time: ext_cert.start_time,
                end_time: ext_cert.end_time,
                status: ext_cert.status,
                rel_rbum_cert_conf_id: ext_cert.rel_rbum_cert_conf_id,
                rel_rbum_cert_conf_name: ext_cert.rel_rbum_cert_conf_name,
                rel_rbum_cert_conf_code: ext_cert.rel_rbum_cert_conf_code,
                rel_rbum_kind: ext_cert.rel_rbum_kind,
                rel_rbum_id: ext_cert.rel_rbum_id,
                own_paths: ext_cert.own_paths,
                owner: ext_cert.owner,
                create_time: ext_cert.create_time,
                update_time: ext_cert.update_time,
            })
        } else {
            Err(funs.err().not_found(
                "iam_cert",
                "get_ext_cert",
                &format!("not found credential of kind {:?}", rel_iam_cert_kind),
                "404-iam-cert-kind-not-exist",
            ))
        }
    }

    pub async fn paginate_certs(
        filter: &RbumCertFilterReq,
        page_number: u64,
        page_size: u64,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<TardisPage<RbumCertDetailResp>> {
        RbumCertServ::paginate_detail_rbums(filter, page_number, page_size, desc_sort_by_create, desc_sort_by_update, funs, ctx).await
    }

    pub async fn add_rel_cert(
        cert_id: &str,
        item_id: &str,
        note: Option<String>,
        ext: Option<String>,
        own_paths: Option<String>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<()> {
        let req = &mut RbumRelAggAddReq {
            rel: RbumRelAddReq {
                tag: IamRelKind::IamCertRel.to_string(),
                note,
                from_rbum_kind: RbumRelFromKind::Cert,
                from_rbum_id: cert_id.to_string(),
                to_rbum_item_id: item_id.to_string(),
                to_own_paths: own_paths.unwrap_or_else(|| ctx.own_paths.clone()),
                to_is_outside: true,
                ext,
            },
            attrs: vec![],
            envs: vec![],
        };
        RbumRelServ::add_rel(req, funs, ctx).await?;
        Ok(())
    }

    pub async fn delete_rel_cert(cert_id: &str, item_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let rel_ids = RbumRelServ::find_id_rbums(
            &RbumRelFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                tag: Some(IamRelKind::IamCertRel.to_string()),
                from_rbum_kind: Some(RbumRelFromKind::Cert),
                from_rbum_id: Some(cert_id.to_string()),
                to_rbum_item_id: Some(item_id.to_string()),
                ..Default::default()
            },
            None,
            None,
            funs,
            ctx,
        )
        .await?;
        if rel_ids.is_empty() {
            return Ok(());
        }
        for rel_id in rel_ids {
            RbumRelServ::delete_rbum(&rel_id, funs, ctx).await?;
        }
        Ok(())
    }

    pub async fn find_to_simple_rel_cert(
        item_id: &str,
        desc_by_create: Option<bool>,
        desc_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<Vec<RbumRelBoneResp>> {
        let rel = IamRelServ::find_rels(
            &RbumRelFilterReq {
                basic: RbumBasicFilterReq {
                    own_paths: Some("".to_string()),
                    with_sub_own_paths: true,
                    ignore_scope: true,
                    ..Default::default()
                },
                tag: Some(IamRelKind::IamCertRel.to_string()),
                to_rbum_item_id: Some(item_id.to_string()),
                ..Default::default()
            },
            desc_by_create,
            desc_by_update,
            funs,
            ctx,
        )
        .await?
        .into_iter()
        .filter(|x| x.rel.own_paths.contains(&ctx.own_paths.clone()) || x.rel.to_own_paths.contains(&ctx.own_paths.clone()))
        .map(|r| RbumRelBoneResp::new(r.rel, false))
        .collect::<Vec<_>>();
        Ok(rel)
    }

    pub async fn find_certs(
        filter: &RbumCertFilterReq,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<Vec<RbumCertSummaryResp>> {
        RbumCertServ::find_rbums(filter, desc_sort_by_create, desc_sort_by_update, funs, ctx).await
    }

    pub async fn delete_cert(id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<u64> {
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
            ctx,
        )
        .await?;
        let result = RbumCertServ::delete_rbum(id, funs, ctx).await?;
        IamIdentCacheServ::delete_tokens_and_contexts_by_account_id(&cert.rel_rbum_id, funs).await?;
        Ok(result)
    }

    pub async fn get_cert_conf_id_by_code(code: &str, rel_iam_item_id: Option<String>, funs: &TardisFunsInst) -> TardisResult<String> {
        Self::get_cert_conf_id_and_ext_by_code(code, rel_iam_item_id, funs).await.map(|r| r.id)
    }

    pub async fn get_cert_conf_id_and_ext_by_code(code: &str, rel_iam_item_id: Option<String>, funs: &TardisFunsInst) -> TardisResult<RbumCertConfIdAndExtResp> {
        Self::get_cert_conf_id_and_ext_opt_by_code(code, rel_iam_item_id, funs)
            .await?
            .ok_or_else(|| funs.err().not_found("iam_cert_conf", "get", &format!("not found cert conf code {}", code), "401-iam-cert-code-not-exist"))
    }

    pub async fn get_cert_conf_id_and_ext_opt_by_code(code: &str, rel_iam_item_id: Option<String>, funs: &TardisFunsInst) -> TardisResult<Option<RbumCertConfIdAndExtResp>> {
        RbumCertConfServ::get_rbum_cert_conf_id_and_ext_by_code(code, &funs.iam_basic_domain_iam_id(), rel_iam_item_id.unwrap_or_else(|| "".to_string()).as_str(), funs).await
    }

    pub async fn package_tardis_context_and_resp(
        tenant_id: Option<String>,
        account_id: &str,
        token_kind: Option<String>,
        access_token: Option<String>,
        funs: &TardisFunsInst,
    ) -> TardisResult<IamAccountInfoResp> {
        let token_kind = IamCertTokenKind::parse(&token_kind);
        let token = TardisFuns::crypto.key.generate_token()?;
        let tenant_id = if let Some(tenant_id) = tenant_id { tenant_id } else { "".to_string() };
        let context = TardisContext {
            own_paths: tenant_id.clone(),
            owner: account_id.to_string(),
            roles: vec![],
            groups: vec![],
            ..Default::default()
        };
        let rbum_cert_conf_id = Self::get_cert_conf_id_by_code(token_kind.to_string().as_str(), Some(tenant_id.clone()), funs).await?;

        let account_info = Self::package_tardis_account_context_and_resp(account_id, &tenant_id, token, access_token, funs, &context).await?;

        IamCertTokenServ::add_cert(&account_info.token, &token_kind, account_id, &rbum_cert_conf_id, funs, &context).await?;

        Ok(account_info)
    }

    pub async fn package_tardis_account_context_and_resp(
        account_id: &str,
        tenant_id: &str,
        token: String,
        access_token: Option<String>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<IamAccountInfoResp> {
        let account_agg = IamAccountServ::get_account_detail_aggs(
            account_id,
            &IamAccountFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                ..Default::default()
            },
            false,
            true,
            funs,
            ctx,
        )
        .await?;
        if account_agg.disabled {
            return Err(funs.err().unauthorized("iam_account", "account_context", "cert is locked", "401-rbum-cert-lock"));
        }
        let account_info = IamAccountInfoResp {
            account_id: account_id.to_string(),
            account_name: account_agg.name.to_string(),
            token,
            access_token,
            roles: account_agg.roles,
            groups: account_agg.groups,
            apps: account_agg.apps,
        };
        IamIdentCacheServ::add_contexts(&account_info, tenant_id, funs).await?;
        Ok(account_info)
    }

    pub fn try_use_tenant_ctx(ctx: TardisContext, tenant_id: Option<String>) -> TardisResult<TardisContext> {
        if let Some(tenant_id) = &tenant_id {
            Self::use_tenant_ctx(ctx, tenant_id)
        } else {
            Ok(ctx)
        }
    }

    pub fn use_sys_or_tenant_ctx_unsafe(mut ctx: TardisContext) -> TardisResult<TardisContext> {
        ctx.own_paths = rbum_scope_helper::get_path_item(RBUM_SCOPE_LEVEL_TENANT.to_int(), &ctx.own_paths).unwrap_or_else(|| "".to_string());
        Ok(ctx)
    }

    pub fn use_sys_ctx_unsafe(mut ctx: TardisContext) -> TardisResult<TardisContext> {
        ctx.own_paths = "".to_string();
        Ok(ctx)
    }

    pub fn use_tenant_ctx(ctx: TardisContext, tenant_id: &str) -> TardisResult<TardisContext> {
        rbum_scope_helper::degrade_own_paths(ctx, tenant_id.to_string().as_str())
    }

    pub fn try_use_app_ctx(ctx: TardisContext, app_id: Option<String>) -> TardisResult<TardisContext> {
        if let Some(app_id) = &app_id {
            Self::use_app_ctx(ctx, app_id)
        } else {
            Ok(ctx)
        }
    }

    pub fn use_app_ctx(ctx: TardisContext, app_id: &str) -> TardisResult<TardisContext> {
        let own_paths = ctx.own_paths.clone();
        rbum_scope_helper::degrade_own_paths(ctx, format!("{}/{}", own_paths, app_id).as_str())
    }

    pub fn get_anonymous_context() -> TardisContext {
        TardisContext {
            own_paths: "_/_/_/_/_/_".to_string(),
            ak: "".to_string(),
            owner: "".to_string(),
            roles: vec![],
            groups: vec![],
            ..Default::default()
        }
    }
}
