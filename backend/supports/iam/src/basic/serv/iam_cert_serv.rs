use bios_basic::helper::request_helper::{add_ip, get_real_ip_from_ctx};
use bios_basic::process::task_processor::TaskProcessor;
use bios_basic::rbum::dto::rbum_rel_agg_dto::RbumRelAggAddReq;
use bios_basic::rbum::serv::rbum_rel_serv::RbumRelServ;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;
use std::vec;
use tardis::basic::dto::TardisContext;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::futures_util::future::join_all;
use tardis::serde_json::json;
use tardis::tokio::sync::Mutex;

use tardis::web::web_resp::TardisPage;
use tardis::{TardisFuns, TardisFunsInst};

use bios_basic::rbum::dto::rbum_cert_conf_dto::{RbumCertConfDetailResp, RbumCertConfIdAndExtResp, RbumCertConfModifyReq, RbumCertConfSummaryResp};
use bios_basic::rbum::dto::rbum_cert_dto::{RbumCertAddReq, RbumCertDetailResp, RbumCertModifyReq, RbumCertSummaryResp, RbumCertSummaryWithSkResp};
use bios_basic::rbum::dto::rbum_filer_dto::{RbumBasicFilterReq, RbumCertConfFilterReq, RbumCertFilterReq, RbumRelFilterReq};
use bios_basic::rbum::dto::rbum_rel_dto::{RbumRelAddReq, RbumRelBoneResp};
use bios_basic::rbum::helper::rbum_scope_helper;
use bios_basic::rbum::rbum_enumeration::{RbumCertConfStatusKind, RbumCertRelKind, RbumCertStatusKind, RbumRelFromKind};
use bios_basic::rbum::serv::rbum_cert_serv::{RbumCertConfServ, RbumCertServ};
use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;

use super::clients::iam_log_client::{IamLogClient, LogParamTag};
use super::iam_rel_serv::IamRelServ;
use super::iam_res_serv::IamResServ;
use super::iam_role_serv::IamRoleServ;
use super::iam_set_serv::IamSetServ;
use crate::basic::dto::iam_account_dto::{IamAccountAppInfoResp, IamAccountInfoResp};
use crate::basic::dto::iam_app_dto::IamAppKind;
use crate::basic::dto::iam_cert_conf_dto::{
    IamCertConfLdapAddOrModifyReq, IamCertConfMailVCodeAddOrModifyReq, IamCertConfPhoneVCodeAddOrModifyReq, IamCertConfTokenAddReq, IamCertConfUserPwdAddOrModifyReq,
};
use crate::basic::dto::iam_cert_dto::{
    IamCertManageAddReq, IamCertManageModifyReq, IamCertModifyVisibilityRequest, IamThirdIntegrationConfigDto, IamThirdIntegrationSyncAddReq, IamThirdIntegrationSyncStatusDto,
    IamThirdPartyCertExtAddReq, IamThirdPartyCertExtModifyReq,
};
use crate::basic::dto::iam_filer_dto::{IamAccountFilterReq, IamAppFilterReq, IamResFilterReq, IamRoleFilterReq};
use crate::basic::serv::iam_account_serv::IamAccountServ;
use crate::basic::serv::iam_app_serv::IamAppServ;
use crate::basic::serv::iam_cert_ldap_serv::IamCertLdapServ;
use crate::basic::serv::iam_cert_mail_vcode_serv::IamCertMailVCodeServ;
use crate::basic::serv::iam_cert_phone_vcode_serv::IamCertPhoneVCodeServ;
use crate::basic::serv::iam_cert_token_serv::IamCertTokenServ;
use crate::basic::serv::iam_cert_user_pwd_serv::IamCertUserPwdServ;
use crate::basic::serv::iam_key_cache_serv::IamIdentCacheServ;
use crate::basic::serv::iam_tenant_serv::IamTenantServ;
use crate::iam_config::{IamBasicConfigApi, IamConfig};
use crate::iam_constants::{self, IAM_AVATAR, RBUM_SCOPE_LEVEL_TENANT};
use crate::iam_enumeration::{IamAccountLockStateKind, IamCertExtKind, IamCertKernelKind, IamCertTokenKind, IamRelKind, IamResKind, IamSetKind};

lazy_static! {
    static ref SYNC_LOCK: Arc<Mutex<Option<usize>>> = Arc::new(Mutex::new(None));
}

pub struct IamCertServ;

impl IamCertServ {
    pub fn get_new_pwd() -> String {
        // TODO 等待 bios_basic::field::nanoid_len(10) 支持自定义 alphabet
        // TardisFuns::field.nanoid_len(10)
        let alphabet: [char; 62] = [
            '1', '2', '3', '4', '5', '6', '7', '8', '9', '0', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w',
            'x', 'y', 'z', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
        ];
        nanoid::nanoid!(10, &alphabet)
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
            IamCertPhoneVCodeServ::add_or_enable_cert_conf(&phone_vcode_cert_conf_add_req, rbum_scope_helper::get_max_level_id_by_context(ctx), funs, ctx).await?;
        }

        if let Some(mail_vcode_cert_conf_add_req) = mail_vcode_cert_conf_add_req {
            IamCertMailVCodeServ::add_or_enable_cert_conf(&mail_vcode_cert_conf_add_req, rbum_scope_helper::get_max_level_id_by_context(ctx), funs, ctx).await?;
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

    pub async fn get_cert_conf(id: &str, iam_item_id: Option<String>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<RbumCertConfDetailResp> {
        RbumCertConfServ::get_rbum(
            id,
            &RbumCertConfFilterReq {
                rel_rbum_domain_id: Some(funs.iam_basic_domain_iam_id()),
                rel_rbum_item_id: iam_item_id,
                status: Some(RbumCertConfStatusKind::Enabled),
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
                status: Some(RbumCertConfStatusKind::Enabled),
                ..Default::default()
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
                status: Some(RbumCertConfStatusKind::Enabled),
                ..Default::default()
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
                r.kind == IamCertKernelKind::UserPwd.to_string() || r.kind == IamCertKernelKind::PhoneVCode.to_string() || r.kind == IamCertKernelKind::MailVCode.to_string()
            })
            .collect();
        Ok(result)
    }

    pub async fn get_cert_detail_by_id_and_kind(
        account_id: &str,
        rel_iam_cert_kind: &IamCertKernelKind,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<RbumCertDetailResp> {
        let ctx = IamAccountServ::is_global_account_context(account_id, funs, ctx).await?;
        let rel_rbum_cert_conf_id = Self::get_cert_conf_id_by_kind(rel_iam_cert_kind.to_string().as_str(), Some(ctx.clone().own_paths), funs).await?;
        let cert_detail = RbumCertServ::find_one_detail_rbum(
            &RbumCertFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
                    own_paths: Some("".to_string()),
                    ..Default::default()
                },
                rel_rbum_id: Some(account_id.to_string()),
                rel_rbum_cert_conf_ids: Some(vec![rel_rbum_cert_conf_id]),
                ..Default::default()
            },
            funs,
            &ctx,
        )
        .await?;
        if let Some(cert_detail) = cert_detail {
            Ok(cert_detail)
        } else {
            Err(funs.err().not_found(
                "iam_cert",
                "get_cert_detail_by_id_and_kind",
                &format!("not found credential of kind {rel_iam_cert_kind:?}"),
                "404-iam-cert-kind-not-exist",
            ))
        }
    }

    pub async fn get_kernel_cert(account_id: &str, rel_iam_cert_kind: &IamCertKernelKind, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<RbumCertSummaryWithSkResp> {
        let kernel_cert = Self::get_cert_detail_by_id_and_kind(account_id, rel_iam_cert_kind, funs, ctx).await;
        if let Ok(kernel_cert) = kernel_cert {
            let now_sk = RbumCertServ::show_sk(
                kernel_cert.id.as_str(),
                &RbumCertFilterReq {
                    basic: RbumBasicFilterReq {
                        ignore_scope: true,
                        own_paths: Some("".to_string()),
                        with_sub_own_paths: true,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                funs,
                ctx,
            )
            .await?;
            Ok(RbumCertSummaryWithSkResp {
                id: kernel_cert.id,
                ak: kernel_cert.ak,
                sk: now_sk,
                sk_invisible: kernel_cert.sk_invisible,
                ext: kernel_cert.ext,
                conn_uri: kernel_cert.conn_uri,
                start_time: kernel_cert.start_time,
                end_time: kernel_cert.end_time,
                status: kernel_cert.status,
                kind: "".to_string(),
                supplier: "".to_string(),
                rel_rbum_cert_conf_id: kernel_cert.rel_rbum_cert_conf_id,
                rel_rbum_cert_conf_name: kernel_cert.rel_rbum_cert_conf_name,
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
                &format!("not found credential of kind {rel_iam_cert_kind:?}"),
                "404-iam-cert-kind-not-exist",
            ))
        }
    }

    pub async fn paginate_cert_conf(
        id: Option<String>,
        kind: Option<TrimString>,
        name: Option<String>,
        with_sub: bool,
        iam_item_id: Option<String>,
        page_number: u32,
        page_size: u32,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<TardisPage<RbumCertConfSummaryResp>> {
        RbumCertConfServ::paginate_rbums(
            &RbumCertConfFilterReq {
                basic: RbumBasicFilterReq {
                    ids: id.map(|id| vec![id]),
                    name,
                    with_sub_own_paths: with_sub,
                    ..Default::default()
                },
                kind,
                rel_rbum_domain_id: Some(funs.iam_basic_domain_iam_id()),
                rel_rbum_item_id: iam_item_id,
                status: Some(RbumCertConfStatusKind::Enabled),
                ..Default::default()
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
        if rbum_cert_conf.kind == IamCertKernelKind::UserPwd.to_string() {
            return Err(funs.err().conflict("iam_cert_conf", "delete", "can not delete default credential", "409-rbum-cert-conf-basic-delete"));
        }
        let result = RbumCertConfServ::delete_rbum(id, funs, ctx).await?;
        Self::clean_cache_by_cert_conf(id, Some(rbum_cert_conf), funs, ctx).await?;
        Ok(result)
    }

    pub async fn delete_cert_and_conf_by_conf_id(id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let task_ctx = ctx.clone();
        let cert_id = id.to_string();
        TaskProcessor::execute_task_with_ctx(
            &funs.conf::<IamConfig>().cache_key_async_task_status,
            move |_task_id| async move {
                let funs = iam_constants::get_tardis_inst();
                let rbum_cert_conf = RbumCertConfServ::peek_rbum(
                    &cert_id,
                    &RbumCertConfFilterReq {
                        basic: RbumBasicFilterReq {
                            with_sub_own_paths: true,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    &funs,
                    &task_ctx,
                )
                .await?;
                if rbum_cert_conf.kind == IamCertKernelKind::UserPwd.to_string() {
                    return Err(funs.err().conflict("iam_cert_conf", "delete", "can not delete default credential", "409-rbum-cert-conf-basic-delete"));
                }
                let certs = IamCertServ::find_certs(
                    &RbumCertFilterReq {
                        basic: RbumBasicFilterReq {
                            own_paths: Some(task_ctx.own_paths.clone()),
                            with_sub_own_paths: true,
                            ..Default::default()
                        },
                        rel_rbum_cert_conf_ids: Some(vec![cert_id.clone()]),
                        ..Default::default()
                    },
                    None,
                    None,
                    &funs,
                    &task_ctx,
                )
                .await?;
                for cert in certs {
                    IamCertServ::delete_cert(&cert.id, &funs, &task_ctx).await?;
                }
                let _result = RbumCertConfServ::delete_rbum(&cert_id, &funs, &task_ctx).await?;
                Self::clean_cache_by_cert_conf(&cert_id, Some(rbum_cert_conf), &funs, &task_ctx).await?;
                Ok(())
            },
            &funs.cache(),
            IAM_AVATAR.to_string(),
            Some(vec![format!("account/{}", ctx.owner)]),
            ctx,
        )
        .await?;
        Ok(())
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
        if rbum_cert_conf.kind == IamCertKernelKind::UserPwd.to_string()
            || rbum_cert_conf.kind == IamCertKernelKind::MailVCode.to_string()
            || rbum_cert_conf.kind == IamCertKernelKind::PhoneVCode.to_string()
        {
            IamIdentCacheServ::delete_tokens_and_contexts_by_tenant_or_app(&rbum_cert_conf.rel_rbum_item_id, false, funs, ctx).await?;
        }
        Ok(())
    }

    /// TODO 需要精简代码 统一使用 3th 的方法
    pub async fn add_manage_cert(add_req: &IamCertManageAddReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<String> {
        let id = RbumCertServ::add_rbum(
            &mut RbumCertAddReq {
                ak: TrimString(add_req.ak.trim().to_string()),
                sk: add_req.sk.as_ref().map(|sk| TrimString(sk.trim().to_string())),
                sk_invisible: add_req.sk_invisible,
                kind: Some(IamCertExtKind::ThirdParty.to_string()),
                supplier: Some(add_req.supplier.clone()),
                vcode: None,
                ext: Some(add_req.ext.clone().unwrap_or_default()),
                start_time: None,
                end_time: None,
                conn_uri: add_req.conn_uri.clone(),
                status: RbumCertStatusKind::Enabled,
                rel_rbum_cert_conf_id: None,
                rel_rbum_kind: RbumCertRelKind::Item,
                rel_rbum_id: ctx.own_paths.to_string(),
                is_outside: true,
                ignore_check_sk: false,
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
                sk: Some(TrimString(modify_req.sk.clone().unwrap_or_default())),
                sk_invisible: modify_req.sk_invisible,
                ignore_check_sk: false,
                start_time: None,
                end_time: None,
                conn_uri: modify_req.conn_uri.clone(),
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
                sk_invisible: None,

                ignore_check_sk: false,
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
        let rel_ids = RbumRelServ::find_id_rbums(
            &RbumRelFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                tag: Some(IamRelKind::IamCertRel.to_string()),
                from_rbum_kind: Some(RbumRelFromKind::Cert),
                from_rbum_id: Some(id.to_string()),
                ..Default::default()
            },
            None,
            None,
            funs,
            ctx,
        )
        .await?;
        if !rel_ids.is_empty() {
            return Err(funs.err().conflict(
                "cert",
                "delete",
                &format!("can not delete cert.{id} when there are associated by rel.{rel_ids:?}"),
                "409-iam-delete-conflict",
            ));
        }
        RbumCertServ::delete_rbum(id, funs, ctx).await?;
        Ok(())
    }

    pub async fn add_3th_kind_cert(add_req: &mut IamThirdPartyCertExtAddReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<String> {
        let id = RbumCertServ::add_rbum(
            &mut RbumCertAddReq {
                ak: TrimString(add_req.ak.trim().to_string()),
                sk: add_req.sk.as_ref().map(|sk| TrimString(sk.trim().to_string())),
                sk_invisible: None,
                kind: Some(IamCertExtKind::ThirdParty.to_string()),
                supplier: add_req.supplier.clone(),
                vcode: None,
                ext: add_req.ext.clone(),
                start_time: add_req.start_time,
                end_time: add_req.end_time,
                conn_uri: None,
                status: RbumCertStatusKind::Enabled,
                rel_rbum_cert_conf_id: None,
                rel_rbum_kind: RbumCertRelKind::Item,
                rel_rbum_id: add_req.rel_rbum_id.clone(),
                is_outside: true,
                ignore_check_sk: false,
            },
            funs,
            ctx,
        )
        .await?;
        Ok(id)
    }

    pub async fn modify_3th_kind_cert(modify_req: &mut IamThirdPartyCertExtModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let cert_3th = Self::get_3th_kind_cert_by_rel_rbum_id(Some(modify_req.rel_rbum_id.clone()), Some(vec![modify_req.supplier.clone()]), false, None, funs, ctx).await?;
        RbumCertServ::modify_rbum(
            &cert_3th.id,
            &mut RbumCertModifyReq {
                ak: Some(TrimString(modify_req.ak.trim().to_string())),
                sk: modify_req.sk.as_ref().map(|sk| TrimString(sk.trim().to_string())),
                sk_invisible: None,
                conn_uri: None,
                ignore_check_sk: false,
                ext: modify_req.ext.clone(),
                start_time: modify_req.start_time,
                end_time: modify_req.end_time,
                status: None,
            },
            funs,
            ctx,
        )
        .await
    }

    /// Get general cert method \
    /// if cert_conf_id is Some then use cert_conf_id as query param \
    /// otherwise use kind、cert_supplier as query param
    pub async fn get_cert_by_rel_rbum_id_kind_supplier(
        rel_rbum_id: &str,
        kind: &str,
        cert_supplier: Vec<String>,
        cert_conf_id: Option<String>,
        tenant_id: &str,
        ldap_dn: bool,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<RbumCertSummaryWithSkResp> {
        let ldap_cn = !ldap_dn;
        let mut is_ldap = false;
        let rbum_cert_filter_req = if let Some(cert_conf_id) = cert_conf_id {
            let cert_conf = RbumCertConfServ::get_rbum(
                &cert_conf_id,
                &RbumCertConfFilterReq {
                    basic: RbumBasicFilterReq {
                        ignore_scope: true,
                        own_paths: Some("".to_string()),
                        with_sub_own_paths: true,
                        ..Default::default()
                    },
                    status: Some(RbumCertConfStatusKind::Enabled),
                    ..Default::default()
                },
                funs,
                ctx,
            )
            .await;
            is_ldap = if let Ok(resp) = cert_conf { resp.kind == "Ldap" } else { false };
            RbumCertFilterReq {
                basic: RbumBasicFilterReq {
                    own_paths: Some(tenant_id.to_string()),
                    ..Default::default()
                },
                rel_rbum_id: Some(rel_rbum_id.to_string()),
                rel_rbum_cert_conf_ids: Some(vec![cert_conf_id]),
                ..Default::default()
            }
        } else {
            RbumCertFilterReq {
                basic: RbumBasicFilterReq {
                    own_paths: Some(tenant_id.to_string()),
                    ..Default::default()
                },
                kind: Some(kind.to_string()),
                suppliers: Some(cert_supplier.clone()),
                rel_rbum_id: Some(rel_rbum_id.to_string()),
                ..Default::default()
            }
        };
        let ext_cert = RbumCertServ::find_one_detail_rbum(&rbum_cert_filter_req, funs, ctx).await?;
        if let Some(ext_cert) = ext_cert {
            Ok(RbumCertSummaryWithSkResp {
                id: ext_cert.id,
                ak: if is_ldap && ldap_cn { IamCertLdapServ::dn_to_cn(&ext_cert.ak) } else { ext_cert.ak },
                sk: "".to_string(),
                sk_invisible: ext_cert.sk_invisible,
                ext: ext_cert.ext,

                conn_uri: ext_cert.conn_uri,
                start_time: ext_cert.start_time,
                end_time: ext_cert.end_time,
                status: ext_cert.status,
                kind: ext_cert.kind,
                supplier: ext_cert.supplier,
                rel_rbum_cert_conf_id: ext_cert.rel_rbum_cert_conf_id,
                rel_rbum_cert_conf_name: ext_cert.rel_rbum_cert_conf_name,
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
                "get_cert_by_rel_rbum_id_kind_supplier",
                &format!("not found credential of kind:{kind} supplier {cert_supplier:?}"),
                "404-iam-cert-kind-not-exist",
            ))
        }
    }

    /// 通过关联rbum_item id 查询三方凭证
    pub async fn get_3th_kind_cert_by_rel_rbum_id(
        rel_rbum_id: Option<String>,
        cert_supplier: Option<Vec<String>>,
        show_sk: bool,
        ext: Option<String>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<RbumCertSummaryWithSkResp> {
        let ext_cert = RbumCertServ::find_one_detail_rbum(
            &RbumCertFilterReq {
                kind: Some(IamCertExtKind::ThirdParty.to_string()),
                suppliers: cert_supplier.clone(),
                rel_rbum_id,
                ext,
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?;
        if let Some(ext_cert) = ext_cert {
            let encoded_sk = if show_sk {
                let now_sk = RbumCertServ::show_sk(ext_cert.id.as_str(), &RbumCertFilterReq::default(), funs, ctx).await?;
                encode_cert(&ext_cert.id, now_sk, ext_cert.sk_invisible)?
            } else {
                "".to_string()
            };
            Ok(RbumCertSummaryWithSkResp {
                id: ext_cert.id,
                ak: ext_cert.ak,
                sk: encoded_sk,
                sk_invisible: ext_cert.sk_invisible,
                ext: ext_cert.ext,
                conn_uri: ext_cert.conn_uri,
                start_time: ext_cert.start_time,
                end_time: ext_cert.end_time,
                status: ext_cert.status,
                kind: ext_cert.kind,
                supplier: ext_cert.supplier,
                rel_rbum_cert_conf_id: ext_cert.rel_rbum_cert_conf_id,
                rel_rbum_cert_conf_name: ext_cert.rel_rbum_cert_conf_name,
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
                "get_3th_kind_cert_by_rel_rbum_id",
                &format!("not found credential of supplier {cert_supplier:?}"),
                "404-iam-cert-kind-not-exist",
            ))
        }
    }

    /// 通过cert id 查询三方凭证
    pub async fn get_3th_kind_cert_by_id(id: &str, show_sk: bool, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<RbumCertSummaryWithSkResp> {
        // query rel ,get owner
        let rels = IamRelServ::find_rels(
            &RbumRelFilterReq {
                basic: RbumBasicFilterReq {
                    own_paths: Some("".to_string()),
                    with_sub_own_paths: true,
                    ignore_scope: true,
                    ..Default::default()
                },
                tag: Some(IamRelKind::IamCertRel.to_string()),
                from_rbum_id: Some(id.to_string()),
                to_own_paths: Some(ctx.own_paths.clone()),
                ..Default::default()
            },
            None,
            None,
            funs,
            ctx,
        )
        .await?;
        let mut mock_ctx = TardisContext { ..ctx.clone() };
        if let Some(rel) = rels.first() {
            mock_ctx.own_paths.clone_from(&rel.rel.own_paths)
        }
        let ext_cert = RbumCertServ::do_find_one_detail_rbum(
            &RbumCertFilterReq {
                basic: RbumBasicFilterReq {
                    ids: Some(vec![id.into()]),
                    ..Default::default()
                },
                kind: Some(IamCertExtKind::ThirdParty.to_string()),
                ..Default::default()
            },
            funs,
            &mock_ctx,
        )
        .await?;
        if let Some(ext_cert) = ext_cert {
            let encoded_sk = if show_sk {
                let now_sk = RbumCertServ::show_sk(ext_cert.id.as_str(), &RbumCertFilterReq::default(), funs, &mock_ctx).await?;
                encode_cert(&ext_cert.id, now_sk, ext_cert.sk_invisible)?
            } else {
                "".to_string()
            };
            Ok(RbumCertSummaryWithSkResp {
                id: ext_cert.id,
                ak: ext_cert.ak,
                sk: encoded_sk,
                ext: ext_cert.ext,
                start_time: ext_cert.start_time,
                end_time: ext_cert.end_time,
                status: ext_cert.status,
                kind: ext_cert.kind,
                supplier: ext_cert.supplier,
                rel_rbum_cert_conf_id: ext_cert.rel_rbum_cert_conf_id,
                rel_rbum_cert_conf_name: ext_cert.rel_rbum_cert_conf_name,
                rel_rbum_kind: ext_cert.rel_rbum_kind,
                rel_rbum_id: ext_cert.rel_rbum_id,
                own_paths: ext_cert.own_paths,
                owner: ext_cert.owner,
                create_time: ext_cert.create_time,
                update_time: ext_cert.update_time,
                conn_uri: ext_cert.conn_uri,
                sk_invisible: ext_cert.sk_invisible,
            })
        } else {
            Err(funs.err().not_found(
                "iam_cert",
                "get_3th_kind_cert_by_id",
                &format!("not found credential by id {id}"),
                "404-rbum-cert-not-exist",
            ))
        }
    }

    /// 通过ak supplier 查询三方凭证
    pub async fn get_3th_kind_cert_by_ak(supplier: &str, ak: &str, show_sk: bool, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<RbumCertSummaryWithSkResp> {
        let query_cert = RbumCertServ::find_one_detail_rbum(
            &RbumCertFilterReq {
                basic: RbumBasicFilterReq {
                    own_paths: Some(ctx.own_paths.clone()),
                    ..Default::default()
                },
                ak: Some(ak.to_string()),
                status: Some(RbumCertStatusKind::Enabled),
                kind: Some(IamCertExtKind::ThirdParty.to_string()),
                suppliers: Some(vec![supplier.to_string()]),
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?;
        if let Some(ext_cert) = query_cert {
            let encoded_sk = if show_sk {
                let now_sk = RbumCertServ::show_sk(ext_cert.id.as_str(), &RbumCertFilterReq::default(), funs, ctx).await?;
                encode_cert(&ext_cert.id, now_sk, ext_cert.sk_invisible)?
            } else {
                "".to_string()
            };
            Ok(RbumCertSummaryWithSkResp {
                id: ext_cert.id,
                ak: ext_cert.ak,
                sk: encoded_sk,
                sk_invisible: ext_cert.sk_invisible,
                ext: ext_cert.ext,
                conn_uri: ext_cert.conn_uri,
                start_time: ext_cert.start_time,
                end_time: ext_cert.end_time,
                status: ext_cert.status,
                kind: ext_cert.kind,
                supplier: ext_cert.supplier,
                rel_rbum_cert_conf_id: ext_cert.rel_rbum_cert_conf_id,
                rel_rbum_cert_conf_name: ext_cert.rel_rbum_cert_conf_name,
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
                "get_3th_kind_cert_by_rel_rbum_id",
                &format!("not found credential of ak {ak}"),
                "404-iam-cert-kind-not-exist",
            ))
        }
    }

    /// 获取相关厂商三方凭证
    pub async fn find_3th_kind_cert(
        rel_rbum_id: Option<String>,
        suppliers: Option<Vec<String>>,
        show_sk: bool,
        ext: Option<String>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<Vec<RbumCertSummaryWithSkResp>> {
        let mut find_cert = vec![];
        let query_cert = RbumCertServ::find_detail_rbums(
            &RbumCertFilterReq {
                basic: RbumBasicFilterReq {
                    own_paths: Some("".to_string()),
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                status: Some(RbumCertStatusKind::Enabled),
                kind: Some(IamCertExtKind::ThirdParty.to_string()),
                ext,
                suppliers,
                rel_rbum_id,
                ..Default::default()
            },
            None,
            None,
            funs,
            ctx,
        )
        .await?;
        for ext_cert in query_cert {
            let encoded_sk = if show_sk {
                let now_sk = RbumCertServ::show_sk(
                    ext_cert.id.as_str(),
                    &RbumCertFilterReq {
                        basic: RbumBasicFilterReq {
                            own_paths: Some("".to_string()),
                            with_sub_own_paths: true,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    funs,
                    ctx,
                )
                .await?;
                encode_cert(&ext_cert.id, now_sk, ext_cert.sk_invisible)?
            } else {
                "".to_string()
            };
            find_cert.push(RbumCertSummaryWithSkResp {
                id: ext_cert.id,
                ak: ext_cert.ak,
                sk: encoded_sk,
                sk_invisible: ext_cert.sk_invisible,
                ext: ext_cert.ext,
                conn_uri: ext_cert.conn_uri,
                start_time: ext_cert.start_time,
                end_time: ext_cert.end_time,
                status: ext_cert.status,
                kind: ext_cert.kind,
                supplier: ext_cert.supplier,
                rel_rbum_cert_conf_id: ext_cert.rel_rbum_cert_conf_id,
                rel_rbum_cert_conf_name: ext_cert.rel_rbum_cert_conf_name,
                rel_rbum_kind: ext_cert.rel_rbum_kind,
                rel_rbum_id: ext_cert.rel_rbum_id,
                own_paths: ext_cert.own_paths,
                owner: ext_cert.owner,
                create_time: ext_cert.create_time,
                update_time: ext_cert.update_time,
            });
        }
        Ok(find_cert)
    }

    pub async fn paginate_certs(
        filter: &RbumCertFilterReq,
        page_number: u32,
        page_size: u32,
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
            RbumRelServ::delete_rel_with_ext(&rel_id, funs, ctx).await?;
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
        IamIdentCacheServ::delete_tokens_and_contexts_by_account_id(&cert.rel_rbum_id, get_real_ip_from_ctx(ctx).await?, funs).await?;
        Ok(result)
    }

    pub async fn find_global_cert_conf_id_by_kind(kind: &str, tenant_id: Option<String>, funs: &TardisFunsInst) -> TardisResult<Vec<String>> {
        let mut cert_conf_ids = Vec::new();
        let global_rbum_cert_conf_id = IamCertServ::get_cert_conf_id_by_kind(kind, None, funs).await?;
        cert_conf_ids.push(global_rbum_cert_conf_id);
        if let Some(tenant_id) = tenant_id {
            let rbum_cert_conf_id = IamCertServ::get_cert_conf_id_by_kind(kind, Some(tenant_id.to_owned()), funs).await?;
            cert_conf_ids.push(rbum_cert_conf_id);
        }
        Ok(cert_conf_ids)
    }

    pub async fn count_cert_ak_by_kind(kind: &str, ak: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<u64> {
        let cert_conf_ids = Self::find_global_cert_conf_id_by_kind(kind, rbum_scope_helper::get_path_item(RBUM_SCOPE_LEVEL_TENANT.to_int(), &ctx.own_paths), funs).await?;
        let count = RbumCertServ::count_rbums(
            &RbumCertFilterReq {
                basic: RbumBasicFilterReq {
                    own_paths: Some("".to_string()),
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                ak: Some(ak.to_string()),
                rel_rbum_kind: Some(RbumCertRelKind::Item),
                rel_rbum_cert_conf_ids: Some(cert_conf_ids),
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?;
        Ok(count)
    }

    pub async fn get_cert_conf_id_by_kind(kind: &str, rel_iam_item_id: Option<String>, funs: &TardisFunsInst) -> TardisResult<String> {
        Self::get_cert_conf_id_and_ext_by_kind_supplier(kind, "", rel_iam_item_id, funs).await.map(|r| r.id)
    }

    pub async fn get_cert_conf_id_by_kind_supplier(kind: &str, supplier: &str, rel_iam_item_id: Option<String>, funs: &TardisFunsInst) -> TardisResult<String> {
        Self::get_cert_conf_id_and_ext_by_kind_supplier(kind, supplier, rel_iam_item_id, funs).await.map(|r| r.id)
    }

    pub async fn get_cert_conf_id_and_ext_by_kind_supplier(
        kind: &str,
        supplier: &str,
        rel_iam_item_id: Option<String>,
        funs: &TardisFunsInst,
    ) -> TardisResult<RbumCertConfIdAndExtResp> {
        Self::get_cert_conf_id_and_ext_opt_by_kind_supplier(kind, supplier, rel_iam_item_id.clone(), funs).await?.ok_or_else(|| {
            funs.err().not_found(
                "iam_cert_conf",
                "get",
                &format!("not found cert conf kind:{kind} supplier:{supplier} rel_iam_item_id:{rel_iam_item_id:?}"),
                "401-iam-cert-code-not-exist",
            )
        })
    }

    pub async fn get_cert_conf_id_and_ext_opt_by_kind(code: &str, rel_iam_item_id: Option<String>, funs: &TardisFunsInst) -> TardisResult<Option<RbumCertConfIdAndExtResp>> {
        RbumCertConfServ::get_rbum_cert_conf_id_and_ext_by_kind_supplier(code, "", true, &funs.iam_basic_domain_iam_id(), rel_iam_item_id.unwrap_or_default().as_str(), funs).await
    }

    pub async fn get_cert_conf_id_and_ext_opt_by_kind_supplier(
        kind: &str,
        mut supplier: &str,
        rel_iam_item_id: Option<String>,
        funs: &TardisFunsInst,
    ) -> TardisResult<Option<RbumCertConfIdAndExtResp>> {
        if kind != "Ldap" {
            supplier = "";
        }
        RbumCertConfServ::get_rbum_cert_conf_id_and_ext_by_kind_supplier(kind, supplier, false, &funs.iam_basic_domain_iam_id(), rel_iam_item_id.unwrap_or_default().as_str(), funs)
            .await
    }

    pub async fn package_tardis_context_and_resp(
        tenant_id: Option<String>,
        account_id: &str,
        token_kind: Option<String>,
        access_token: Option<String>,
        ip: Option<String>,
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
        add_ip(ip, &context).await?;
        let rbum_cert_conf_id = Self::get_cert_conf_id_by_kind(token_kind.to_string().as_str(), Some(tenant_id.clone()), funs).await?;

        let account_info = Self::package_tardis_account_context_and_resp(account_id, &tenant_id, token, access_token, funs, &context).await?;

        IamCertTokenServ::add_cert(&account_info.token, &token_kind, account_id, &rbum_cert_conf_id, funs, &context).await?;
        context.execute_task().await?;
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
            false,
            funs,
            ctx,
        )
        .await?;
        if account_agg.disabled {
            return Err(funs.err().unauthorized("iam_account", "account_context", "cert is disabled", "401-iam-account-disabled"));
        }
        if account_agg.lock_status != IamAccountLockStateKind::Unlocked {
            return Err(funs.err().unauthorized("iam_account", "account_context", "cert is locked", "401-rbum-account-lock"));
        }
        let mut old_app_ids = account_agg.apps.iter().map(|app| app.app_id.clone()).collect::<Vec<String>>();
        let mut apps = account_agg.apps;
        if tenant_id != "" {
            let set_id = IamSetServ::get_default_set_id_by_ctx(&IamSetKind::Apps, &funs, &ctx).await?;
            let app_items = IamSetServ::get_app_with_auth_by_account(&set_id, &ctx.owner, &funs, &ctx).await?;
            let mut app_role_read = HashMap::new();
            app_role_read.insert(IamBasicConfigApi::iam_basic_role_app_read_id(funs), iam_constants::RBUM_ITEM_NAME_APP_READ_ROLE.to_string());
            for (app_id, app_name) in app_items {
                if old_app_ids.contains(&app_id) {
                    continue;
                }
                old_app_ids.push(app_id.clone());
                apps.push(IamAccountAppInfoResp {
                    app_id: app_id.clone(),
                    app_name: app_name.clone(),
                    app_own_paths: format!("{}/{}", tenant_id, app_id),
                    app_kind: IamAppKind::Product,
                    app_icon: "".to_string(),
                    roles: app_role_read.clone(),
                    groups: HashMap::new(),
                });
            }
            // todo 拥有全部应用数据权限 则可以查看跨租户的应用，以及当前租户的应用权限 `后续需要优化`
            if IamResServ::is_res_code_with_context(funs.conf::<IamConfig>().app_res_data_guard_code.clone(), &ctx, funs).await? {
                let mock_global_ctx = TardisContext {
                    own_paths: "".to_string(),
                    ..ctx.clone()
                };
                let app_ids = IamTenantServ::find_id_rel_app(tenant_id, None, None, funs, &mock_global_ctx).await?;
                let proj_app = IamAppServ::find_items(
                    &IamAppFilterReq {
                        basic: RbumBasicFilterReq {
                            // own_paths: Some("".to_string()),
                            with_sub_own_paths: true,
                            ..Default::default()
                        },
                        kind: Some(IamAppKind::Project),
                        ..Default::default()
                    },
                    None,
                    None,
                    funs,
                    ctx,
                )
                .await?;
                for app in proj_app {
                    if !old_app_ids.contains(&app.id) {
                        old_app_ids.push(app.id.clone());
                        apps.push(IamAccountAppInfoResp {
                            app_id: app.id.clone(),
                            app_name: app.name.clone(),
                            app_own_paths: format!("{}/{}", tenant_id, app.id),
                            app_kind: IamAppKind::Project,
                            app_icon: app.icon,
                            roles: app_role_read.clone(),
                            groups: HashMap::new(),
                        });
                    }
                }
                for app_id in app_ids {
                    if !old_app_ids.contains(&app_id) {
                        old_app_ids.push(app_id.clone());
                        apps.push(IamAccountAppInfoResp {
                            app_id: app_id.clone(),
                            app_name: "".to_string(),
                            app_own_paths: format!("{}/{}", tenant_id, app_id),
                            app_kind: IamAppKind::Project,
                            app_icon: "".to_string(),
                            roles: app_role_read.clone(),
                            groups: HashMap::new(),
                        });
                    }
                }
            }
        }
        let account_info = IamAccountInfoResp {
            account_id: account_id.to_string(),
            account_name: account_agg.name.to_string(),
            token,
            access_token,
            roles: account_agg.roles,
            groups: account_agg.groups,
            apps,
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
        ctx.own_paths = rbum_scope_helper::get_path_item(RBUM_SCOPE_LEVEL_TENANT.to_int(), &ctx.own_paths).unwrap_or_default();
        Ok(ctx)
    }

    pub fn use_sys_ctx_unsafe(mut ctx: TardisContext) -> TardisResult<TardisContext> {
        ctx.own_paths = "".to_string();
        Ok(ctx)
    }

    pub async fn use_global_account_ctx(mut ctx: TardisContext, account_id: &str, funs: &TardisFunsInst) -> TardisResult<TardisContext> {
        let account: crate::basic::dto::iam_account_dto::IamAccountDetailResp = IamAccountServ::get_item(
            account_id,
            &IamAccountFilterReq {
                basic: RbumBasicFilterReq {
                    own_paths: Some("".to_string()),
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                ..Default::default()
            },
            funs,
            &ctx,
        )
        .await?;
        match account.scope_level {
            bios_basic::rbum::rbum_enumeration::RbumScopeLevelKind::Private => {}
            bios_basic::rbum::rbum_enumeration::RbumScopeLevelKind::Root => {
                ctx.own_paths = "".to_string();
            }
            bios_basic::rbum::rbum_enumeration::RbumScopeLevelKind::L1 => {}
            bios_basic::rbum::rbum_enumeration::RbumScopeLevelKind::L2 => {}
            bios_basic::rbum::rbum_enumeration::RbumScopeLevelKind::L3 => {}
        }
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
        rbum_scope_helper::degrade_own_paths(ctx, format!("{own_paths}/{app_id}").as_str())
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

    pub async fn enabled_cert_conf(cert_conf_by_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        RbumCertConfServ::modify_rbum(
            cert_conf_by_id,
            &mut RbumCertConfModifyReq {
                name: None,
                note: None,
                ak_note: None,
                ak_rule: None,
                sk_note: None,
                sk_rule: None,
                ext: None,
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
                conn_uri: None,
                status: Some(RbumCertConfStatusKind::Enabled),
            },
            funs,
            ctx,
        )
        .await
    }

    pub async fn disable_cert_conf(cert_conf_by_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        RbumCertConfServ::modify_rbum(
            cert_conf_by_id,
            &mut RbumCertConfModifyReq {
                name: None,
                note: None,
                ak_note: None,
                ak_rule: None,
                sk_note: None,
                sk_rule: None,
                ext: None,
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
                conn_uri: None,
                status: Some(RbumCertConfStatusKind::Disabled),
            },
            funs,
            ctx,
        )
        .await
    }

    /// 获取手动同步按钮的操作角色和资源
    async fn find_sync_ele_role_res(funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<(String, String)> {
        if let Some(res_sum) = IamResServ::find_one_item(
            &IamResFilterReq {
                basic: RbumBasicFilterReq {
                    code: Some("2/*/account*client*sync".to_string()),
                    ..Default::default()
                },
                kind: Some(IamResKind::Ele),
                method: Some("*".to_string()),
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?
        {
            let role_sys_admin_id = IamRoleServ::find_one_item(
                &IamRoleFilterReq {
                    basic: RbumBasicFilterReq {
                        code: Some(String::from(iam_constants::RBUM_ITEM_NAME_SYS_ADMIN_ROLE)),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                funs,
                ctx,
            )
            .await?
            .map(|r| r.id)
            .ok_or_else(|| funs.err().not_found("iam", "init", "not found sys admin role", ""))?;
            Ok((role_sys_admin_id, res_sum.id))
        } else {
            Err(funs.err().not_found("third_integration_config", "enable", "sync element should be add first", "404-sync-element-not-found-error"))
        }
    }

    pub async fn add_or_modify_sync_third_integration_config(reqs: Vec<IamThirdIntegrationSyncAddReq>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let tardis_ctx = TardisFuns::crypto.base64.encode(TardisFuns::json.obj_to_string(&ctx)?);
        let headers = vec![("Tardis-Context".to_string(), tardis_ctx.clone())];
        let schedule_url = funs.conf::<IamConfig>().spi.schedule_url.clone();
        if schedule_url.is_empty() {
            return Err(funs.err().not_implemented("third_integration_config", "add_or_modify", "schedule is not impl!", "501-iam-schedule_not_impl_error"));
        };

        match reqs.iter().find(|req| req.account_sync_cron.is_some()).and_then(|req| req.account_sync_cron.clone()) {
            Some(sync_cron) => {
                if !sync_cron.is_empty() {
                    funs.web_client()
                        .put_obj_to_str(
                            &format!("{schedule_url}/ci/schedule/jobs"),
                            &json!({"code": funs.conf::<IamConfig>().third_integration_schedule_code.clone(),
                            "cron": vec![sync_cron],
                            "callback_url": format!("{}/ci/cert/sync", funs.conf::<IamConfig>().iam_base_url,),
                            "callback_headers":{
                              "Tardis-Context":tardis_ctx
                            }}),
                            headers.clone(),
                        )
                        .await?;
                }
            }
            None => {
                funs.web_client()
                    .delete_to_void(
                        &format!("{schedule_url}/ci/schedule/jobs/{}", funs.conf::<IamConfig>().third_integration_schedule_code.clone()),
                        headers.clone(),
                    )
                    .await?;
            }
        }

        let (role_sys_admin_id, res_id) = Self::find_sync_ele_role_res(funs, ctx).await?;
        match reqs.iter().find(|req| req.account_sync_cron.is_none()) {
            Some(_) => {
                // 添加手动同步按钮权限
                let _ = IamRoleServ::add_rel_res(&role_sys_admin_id, &res_id, funs, ctx).await;
            }
            None => {
                let _ = IamRoleServ::delete_rel_res(&role_sys_admin_id, &res_id, funs, ctx).await;
            }
        }

        let values = reqs
            .into_iter()
            .map(|req| IamThirdIntegrationConfigDto {
                account_sync_from: req.account_sync_from,
                account_sync_cron: req.account_sync_cron,
                account_way_to_add: req.account_way_to_add.unwrap_or_default(),
                account_way_to_delete: req.account_way_to_delete.unwrap_or_default(),
            })
            .collect::<Vec<IamThirdIntegrationConfigDto>>();
        //将来切换到spi-kv里
        let third_integration_config_key = funs.conf::<IamConfig>().third_integration_config_key.clone();
        funs.cache().set(&format!("{third_integration_config_key}:{}", ctx.own_paths), &TardisFuns::json.obj_to_string(&values)?).await?;
        Ok(())
    }

    pub async fn get_sync_third_integration_config(funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Option<Vec<IamThirdIntegrationConfigDto>>> {
        let conf = funs.conf::<IamConfig>();
        if let Some(iam_third_integration_sync_add_req_string) = funs.cache().get(&format!("{}:{}", conf.third_integration_config_key, ctx.own_paths)).await? {
            let result = TardisFuns::json.str_to_obj::<Vec<IamThirdIntegrationConfigDto>>(&iam_third_integration_sync_add_req_string)?;
            Ok(Some(result))
        } else {
            Ok(None)
        }
    }

    // TODO fixme 这是干什么？
    pub async fn get_third_intg_sync_status(task_id: &str, funs: &TardisFunsInst) -> TardisResult<Option<IamThirdIntegrationSyncStatusDto>> {
        let mut result = None;
        let task_id = task_id.parse().map_err(|_| funs.err().format_error("system", "task", "task id format error", "406-iam-task-id-format"))?;
        let mut is_end = TaskProcessor::check_status(&funs.conf::<IamConfig>().cache_key_async_task_status, task_id, &funs.cache()).await?;
        for _i in 0..5 {
            result = funs
                .cache()
                .get(&funs.conf::<IamConfig>().cache_key_sync_ldap_status)
                .await?
                .and_then(|s| TardisFuns::json.str_to_obj::<IamThirdIntegrationSyncStatusDto>(&s).ok());
            is_end = TaskProcessor::check_status(&funs.conf::<IamConfig>().cache_key_async_task_status, task_id, &funs.cache()).await?;
            if is_end || (result.is_some() && result.as_ref().expect("").total != 0) {
                break;
            }
            tardis::tokio::time::sleep(Duration::from_millis(10)).await;
        }
        if !is_end && result.is_some() && (result.as_ref().expect("").total <= (result.as_ref().expect("").success + result.as_ref().expect("").failed) as usize) {
            result = Some(IamThirdIntegrationSyncStatusDto {
                total: (result.as_ref().expect("").success + result.as_ref().expect("").failed) as usize + 1,
                success: result.as_ref().expect("").success,
                failed: result.as_ref().expect("").failed,
            });
        }
        if is_end && result.is_some() {
            result = Some(IamThirdIntegrationSyncStatusDto {
                total: (result.as_ref().expect("").success + result.as_ref().expect("").failed) as usize,
                success: result.as_ref().expect("").success,
                failed: result.as_ref().expect("").failed,
            });
        }
        Ok(result)
    }

    /// 第三方集成手动同步方法入口
    pub async fn manual_third_integration_sync(account_sync_from: IamCertExtKind, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let task_ctx = ctx.clone();
        let sync = SYNC_LOCK.try_lock().map_err(|_| funs.err().conflict("third_integration_config", "sync", "The last synchronization has not ended yet", "iam-sync-not-ended"))?;
        if let Some(task_id) = funs.cache().get(&funs.conf::<IamConfig>().cache_key_sync_ldap_task_lock).await?.and_then(|task_id| task_id.parse().ok()) {
            if !TaskProcessor::check_status(&funs.conf::<IamConfig>().cache_key_async_task_status, task_id, &funs.cache()).await? {
                return Err(funs.err().conflict("third_integration_config", "sync", "The last synchronization has not ended yet", "iam-sync-not-ended"));
            };
        }

        let task_id = TaskProcessor::execute_task_with_ctx(
            &funs.conf::<IamConfig>().cache_key_async_task_status,
            move |_task_id| async move {
                let sync = sync;
                let funs = iam_constants::get_tardis_inst();

                let sync_config = if let Some(sync_config) = IamCertServ::get_sync_third_integration_config(&funs, &task_ctx).await? {
                    match sync_config.into_iter().find(|sync_config| sync_config.account_sync_cron.is_none() && sync_config.account_sync_from == account_sync_from) {
                        Some(config) => config,
                        None => return Err(funs.err().conflict("ldap_account", "sync", "should have sync config!", "iam-not-found-sync-config")),
                    }
                } else {
                    return Err(funs.err().conflict("ldap_account", "sync", "should have sync config!", "iam-not-found-sync-config"));
                };

                match sync_config.account_sync_from {
                    IamCertExtKind::Ldap => {
                        IamCertLdapServ::iam_sync_ldap_user_to_iam(sync_config, &funs, &task_ctx).await?;
                    }
                    _ => {
                        let _ = funs.err().not_implemented("third_integration", "sync", "501-sync-from-is-not-implemented", "501-sync-from-is-not-implemented");
                    }
                };
                drop(sync);
                Ok(())
            },
            &funs.cache(),
            IAM_AVATAR.to_owned(),
            Some(vec![format!("account/{}", ctx.owner)]),
            ctx,
        )
        .await?;
        funs.cache().set(&funs.conf::<IamConfig>().cache_key_sync_ldap_task_lock, &task_id.to_string()).await?;
        Ok(())
    }

    pub async fn third_integration_sync_without_config(funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<String> {
        let sync_config = if let Some(sync_config) = IamCertServ::get_sync_third_integration_config(funs, ctx).await? {
            match sync_config.into_iter().find(|sync_config| sync_config.account_sync_cron.is_some()) {
                Some(config) => config,
                None => return Err(funs.err().conflict("ldap_account", "sync", "should have sync config!", "iam-not-found-sync-config")),
            }
        } else {
            return Err(funs.err().conflict("ldap_account", "sync", "should have sync config!", "iam-not-found-sync-config"));
        };
        match sync_config.account_sync_from {
            IamCertExtKind::Ldap => IamCertLdapServ::iam_sync_ldap_user_to_iam(sync_config, funs, ctx).await,
            _ => Err(funs.err().not_implemented("third_integration", "sync", "501-sync-from-is-not-implemented", "501-sync-from-is-not-implemented")),
        }
    }

    pub async fn validate_by_ak_and_sk(
        ak: &str,
        input_sk: &str,
        rbum_cert_conf_id: Option<&str>,
        rel_rbum_kind: Option<&RbumCertRelKind>,
        ignore_end_time: bool,
        own_paths: Option<String>,
        allowed_kinds: Option<Vec<&str>>,
        ip: Option<String>,
        funs: &TardisFunsInst,
    ) -> TardisResult<(String, RbumCertRelKind, String)> {
        let result: Result<(String, RbumCertRelKind, String), tardis::basic::error::TardisError> = if rbum_cert_conf_id.is_some() {
            RbumCertServ::validate_by_spec_cert_conf(
                ak,
                input_sk,
                rbum_cert_conf_id.unwrap_or(""),
                ignore_end_time,
                own_paths.clone().unwrap_or_default().as_str(),
                funs,
            )
            .await
        } else {
            RbumCertServ::validate_by_ak_and_basic_sk(
                ak,
                input_sk,
                rel_rbum_kind.unwrap_or(&RbumCertRelKind::Item),
                ignore_end_time,
                own_paths.clone(),
                allowed_kinds.unwrap_or_default(),
                funs,
            )
            .await
        };
        if let Err(e) = result.as_ref() {
            if e.message.as_str() == "cert is locked" {
                let mut mock_ctx = TardisContext { ..Default::default() };
                if let Some(own_paths) = own_paths {
                    mock_ctx.own_paths = own_paths;
                }
                add_ip(ip, &mock_ctx).await?;
                let _ = IamLogClient::add_ctx_task(
                    LogParamTag::IamAccount,
                    None,
                    "密码锁定账号".to_string(),
                    Some("PasswordLockAccount".to_string()),
                    &mock_ctx,
                )
                .await;
                let _ = IamLogClient::add_ctx_task(
                    LogParamTag::SecurityVisit,
                    None,
                    "连续登录失败".to_string(),
                    Some("ContinuLoginFail".to_string()),
                    &mock_ctx,
                )
                .await;
                mock_ctx.execute_task().await?;
            }
        }
        result
    }

    pub async fn batch_decode_cert(codes: HashSet<String>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<HashMap<String, String>> {
        let batch_result = join_all(codes.into_iter().filter_map(|code| {
            if let Some((id, "sk")) = code.split_once('/') {
                let id = id.to_owned();
                Some(async move {
                    let Ok(rels) = IamRelServ::find_rels(
                        &RbumRelFilterReq {
                            basic: RbumBasicFilterReq {
                                own_paths: Some("".to_string()),
                                with_sub_own_paths: true,
                                ignore_scope: true,
                                ..Default::default()
                            },
                            tag: Some(IamRelKind::IamCertRel.to_string()),
                            from_rbum_id: Some(id.to_string()),
                            to_own_paths: Some(ctx.own_paths.clone()),
                            ..Default::default()
                        },
                        None,
                        None,
                        funs,
                        ctx,
                    )
                    .await
                    else {
                        return None;
                    };
                    let mut mock_ctx = TardisContext { ..ctx.clone() };
                    if let Some(rel) = rels.first() {
                        mock_ctx.own_paths.clone_from(&rel.rel.own_paths)
                    }
                    let Ok(sk) = RbumCertServ::show_sk(&id, &RbumCertFilterReq::default(), funs, &mock_ctx).await else {
                        return None;
                    };
                    Some((format!("{id}/sk"), sk))
                })
            } else {
                None
            }
        }))
        .await
        .into_iter()
        .fold(HashMap::default(), |mut map, output| {
            if let Some((id, sk)) = output {
                map.insert(id, sk);
            }
            map
        });
        Ok(batch_result)
    }

    pub async fn modify_sk_visibility(id: &str, req: IamCertModifyVisibilityRequest, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let rels = IamRelServ::find_rels(
            &RbumRelFilterReq {
                basic: RbumBasicFilterReq {
                    own_paths: Some("".to_string()),
                    with_sub_own_paths: true,
                    ignore_scope: true,
                    ..Default::default()
                },
                tag: Some(IamRelKind::IamCertRel.to_string()),
                from_rbum_id: Some(id.to_string()),
                to_own_paths: Some(ctx.own_paths.clone()),
                ..Default::default()
            },
            None,
            None,
            funs,
            ctx,
        )
        .await?;
        let mut mock_ctx = TardisContext { ..ctx.clone() };
        if let Some(rel) = rels.first() {
            mock_ctx.own_paths.clone_from(&rel.rel.own_paths)
        }
        let ext_cert = RbumCertServ::do_find_one_detail_rbum(
            &RbumCertFilterReq {
                basic: RbumBasicFilterReq {
                    ids: Some(vec![id.into()]),
                    ..Default::default()
                },
                kind: Some(IamCertExtKind::ThirdParty.to_string()),
                ..Default::default()
            },
            funs,
            &mock_ctx,
        )
        .await?;
        if let Some(ext_cert) = ext_cert {
            RbumCertServ::modify_rbum(
                ext_cert.id.as_str(),
                &mut RbumCertModifyReq {
                    sk_invisible: Some(req.sk_invisible),
                    ak: None,
                    sk: None,
                    ignore_check_sk: true,
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
            Ok(())
        } else {
            Err(funs.err().not_found(
                "iam_cert",
                "get_3th_kind_cert_by_id",
                &format!("not found credential by id {id}"),
                "404-rbum-cert-not-exist",
            ))
        }
    }
}

fn encode_cert(id: &str, sk: String, invisible: bool) -> TardisResult<String> {
    let usage = "CERT";
    let field = "sk";
    if invisible {
        let key = format!("${usage}{{{id}/{field}}}");
        Ok(key)
    } else {
        Ok(sk)
    }
}
