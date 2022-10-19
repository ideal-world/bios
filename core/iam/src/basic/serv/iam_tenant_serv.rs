use async_trait::async_trait;
use std::collections::HashMap;
use tardis::basic::dto::TardisContext;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::db::sea_orm::sea_query::{Expr, SelectStatement};
use tardis::db::sea_orm::*;
use tardis::{TardisFuns, TardisFunsInst};

use bios_basic::rbum::dto::rbum_item_dto::{RbumItemKernelAddReq, RbumItemModifyReq};
use bios_basic::rbum::helper::rbum_scope_helper;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;

use crate::basic::domain::iam_tenant;
use crate::basic::dto::iam_account_dto::IamAccountAggAddReq;
use crate::basic::dto::iam_cert_conf_dto::{IamCertConfMailVCodeAddOrModifyReq, IamCertConfPhoneVCodeAddOrModifyReq};
use crate::basic::dto::iam_filer_dto::IamTenantFilterReq;
use crate::basic::dto::iam_tenant_dto::{
    IamTenantAddReq, IamTenantAggAddReq, IamTenantAggDetailResp, IamTenantAggModifyReq, IamTenantDetailResp, IamTenantModifyReq, IamTenantSummaryResp,
};
use crate::basic::serv::iam_account_serv::IamAccountServ;
use crate::basic::serv::iam_cert_ldap_serv::IamCertLdapServ;
use crate::basic::serv::iam_cert_mail_vcode_serv::IamCertMailVCodeServ;
use crate::basic::serv::iam_cert_phone_vcode_serv::IamCertPhoneVCodeServ;
use crate::basic::serv::iam_cert_serv::IamCertServ;
use crate::basic::serv::iam_cert_user_pwd_serv::IamCertUserPwdServ;
use crate::basic::serv::iam_key_cache_serv::IamIdentCacheServ;
use crate::basic::serv::iam_set_serv::IamSetServ;
use crate::iam_config::{IamBasicConfigApi, IamBasicInfoManager};
use crate::iam_constants;
use crate::iam_constants::{RBUM_ITEM_ID_TENANT_LEN, RBUM_SCOPE_LEVEL_TENANT};
use crate::iam_enumeration::{IamCertExtKind, IamCertKernelKind, IamSetKind};

use super::iam_cert_oauth2_serv::IamCertOAuth2Serv;

pub struct IamTenantServ;

#[async_trait]
impl RbumItemCrudOperation<iam_tenant::ActiveModel, IamTenantAddReq, IamTenantModifyReq, IamTenantSummaryResp, IamTenantDetailResp, IamTenantFilterReq> for IamTenantServ {
    fn get_ext_table_name() -> &'static str {
        iam_tenant::Entity.table_name()
    }

    fn get_rbum_kind_id() -> String {
        IamBasicInfoManager::get_config(|conf| conf.kind_tenant_id.clone())
    }

    fn get_rbum_domain_id() -> String {
        IamBasicInfoManager::get_config(|conf| conf.domain_iam_id.clone())
    }

    async fn package_item_add(add_req: &IamTenantAddReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<RbumItemKernelAddReq> {
        Ok(RbumItemKernelAddReq {
            id: add_req.id.clone(),
            code: None,
            name: add_req.name.clone(),
            scope_level: add_req.scope_level.clone(),
            disabled: add_req.disabled,
        })
    }

    async fn package_ext_add(id: &str, add_req: &IamTenantAddReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<iam_tenant::ActiveModel> {
        Ok(iam_tenant::ActiveModel {
            id: Set(id.to_string()),
            icon: Set(add_req.icon.as_ref().unwrap_or(&"".to_string()).to_string()),
            sort: Set(add_req.sort.unwrap_or(0)),
            contact_phone: Set(add_req.contact_phone.as_ref().unwrap_or(&"".to_string()).to_string()),
            note: Set(add_req.note.as_ref().unwrap_or(&"".to_string()).to_string()),
            account_self_reg: Set(add_req.account_self_reg.unwrap_or(false)),
            ..Default::default()
        })
    }

    async fn package_item_modify(_: &str, modify_req: &IamTenantModifyReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<Option<RbumItemModifyReq>> {
        if modify_req.name.is_none() && modify_req.scope_level.is_none() && modify_req.disabled.is_none() {
            return Ok(None);
        }
        Ok(Some(RbumItemModifyReq {
            code: None,
            name: modify_req.name.clone(),
            scope_level: modify_req.scope_level.clone(),
            disabled: modify_req.disabled,
        }))
    }

    async fn package_ext_modify(id: &str, modify_req: &IamTenantModifyReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<Option<iam_tenant::ActiveModel>> {
        if modify_req.icon.is_none() && modify_req.sort.is_none() && modify_req.contact_phone.is_none() && modify_req.note.is_none() {
            return Ok(None);
        }
        let mut iam_tenant = iam_tenant::ActiveModel {
            id: Set(id.to_string()),
            ..Default::default()
        };
        if let Some(icon) = &modify_req.icon {
            iam_tenant.icon = Set(icon.to_string());
        }
        if let Some(sort) = modify_req.sort {
            iam_tenant.sort = Set(sort);
        }
        if let Some(contact_phone) = &modify_req.contact_phone {
            iam_tenant.contact_phone = Set(contact_phone.to_string());
        }
        if let Some(note) = &modify_req.note {
            iam_tenant.note = Set(note.to_string());
        }
        if let Some(account_self_reg) = modify_req.account_self_reg {
            iam_tenant.account_self_reg = Set(account_self_reg);
        }
        Ok(Some(iam_tenant))
    }

    async fn after_modify_item(id: &str, modify_req: &mut IamTenantModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        if modify_req.disabled.unwrap_or(false) {
            IamIdentCacheServ::delete_tokens_and_contexts_by_tenant_or_app(id, false, funs, ctx).await?;
        }
        Ok(())
    }

    async fn before_delete_item(_: &str, funs: &TardisFunsInst, _: &TardisContext) -> TardisResult<Option<IamTenantDetailResp>> {
        Err(funs.err().conflict(
            &Self::get_obj_name(),
            "delete",
            "tenant can only be disabled but not deleted",
            "409-iam-tenant-can-not-delete",
        ))
    }

    async fn package_ext_query(query: &mut SelectStatement, _: bool, filter: &IamTenantFilterReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<()> {
        query.column((iam_tenant::Entity, iam_tenant::Column::Icon));
        query.column((iam_tenant::Entity, iam_tenant::Column::Sort));
        query.column((iam_tenant::Entity, iam_tenant::Column::ContactPhone));
        query.column((iam_tenant::Entity, iam_tenant::Column::Note));
        query.column((iam_tenant::Entity, iam_tenant::Column::AccountSelfReg));
        if let Some(contact_phone) = &filter.contact_phone {
            query.and_where(Expr::col(iam_tenant::Column::ContactPhone).eq(contact_phone.as_str()));
        }
        Ok(())
    }
}

impl IamTenantServ {
    pub fn get_new_id() -> String {
        TardisFuns::field.nanoid_len(RBUM_ITEM_ID_TENANT_LEN as usize)
    }

    pub fn get_id_by_ctx(ctx: &TardisContext, funs: &TardisFunsInst) -> TardisResult<String> {
        if ctx.own_paths.is_empty() {
            Ok("".to_string())
        } else if let Some(id) = rbum_scope_helper::get_path_item(RBUM_SCOPE_LEVEL_TENANT.to_int(), &ctx.own_paths) {
            Ok(id)
        } else {
            Err(funs.err().unauthorized(
                &Self::get_obj_name(),
                "get_id",
                &format!("tenant id not found in tardis content {}", ctx.own_paths),
                "401-iam-tenant-context-not-exist",
            ))
        }
    }

    pub async fn add_tenant_agg(add_req: &IamTenantAggAddReq, funs: &TardisFunsInst) -> TardisResult<(String, String)> {
        let tenant_admin_id = TardisFuns::field.nanoid();
        // TODO security check
        let tenant_id = IamTenantServ::get_new_id();
        let tenant_ctx = TardisContext {
            own_paths: tenant_id.clone(),
            ak: "".to_string(),
            roles: vec![],
            groups: vec![],
            owner: tenant_admin_id.to_string(),
            ..Default::default()
        };
        Self::add_item(
            &mut IamTenantAddReq {
                id: Some(TrimString(tenant_id.clone())),
                name: add_req.name.clone(),
                icon: add_req.icon.clone(),
                sort: None,
                contact_phone: add_req.contact_phone.clone(),
                disabled: add_req.disabled,
                scope_level: Some(iam_constants::RBUM_SCOPE_LEVEL_GLOBAL),
                note: add_req.note.clone(),
                account_self_reg: add_req.account_self_reg,
            },
            funs,
            &tenant_ctx,
        )
        .await?;

        IamSetServ::init_set(IamSetKind::Org, RBUM_SCOPE_LEVEL_TENANT, funs, &tenant_ctx).await?;
        IamSetServ::init_set(IamSetKind::Apps, RBUM_SCOPE_LEVEL_TENANT, funs, &tenant_ctx).await?;

        // Init cert conf
        let cert_conf_by_phone_vcode = if add_req.cert_conf_by_phone_vcode {
            Some(IamCertConfPhoneVCodeAddOrModifyReq { ak_note: None, ak_rule: None })
        } else {
            None
        };
        let cert_conf_by_mail_vcode = if add_req.cert_conf_by_mail_vcode {
            Some(IamCertConfMailVCodeAddOrModifyReq { ak_note: None, ak_rule: None })
        } else {
            None
        };
        IamCertServ::init_default_ident_conf(&add_req.cert_conf_by_user_pwd, cert_conf_by_phone_vcode, cert_conf_by_mail_vcode, funs, &tenant_ctx).await?;
        IamCertServ::init_default_ext_conf(funs, &tenant_ctx).await?;
        IamCertServ::init_default_manage_conf(funs, &tenant_ctx).await?;

        if let Some(cert_conf_by_wechat_mp) = &add_req.cert_conf_by_wechat_mp {
            IamCertOAuth2Serv::add_cert_conf(IamCertExtKind::WechatMp, cert_conf_by_wechat_mp, tenant_id.to_string(), funs, &tenant_ctx).await?;
        }

        if let Some(cert_conf_by_ldaps) = &add_req.cert_conf_by_ldap {
            if !cert_conf_by_ldaps.is_empty() {
                for cert_conf_by_ldap in cert_conf_by_ldaps {
                    IamCertLdapServ::add_cert_conf(cert_conf_by_ldap, tenant_id.to_string(), funs, &tenant_ctx).await?;
                }
            }
        }

        // Init pwd
        let pwd = if let Some(admin_password) = &add_req.admin_password {
            admin_password.to_string()
        } else {
            IamCertServ::get_new_pwd()
        };

        IamAccountServ::add_account_agg(
            &IamAccountAggAddReq {
                id: Some(TrimString(tenant_admin_id.clone())),
                name: add_req.admin_name.clone(),
                cert_user_name: TrimString(add_req.admin_username.0.to_string()),
                cert_password: TrimString(pwd.to_string()),
                cert_phone: None,
                cert_mail: None,
                icon: None,
                disabled: add_req.disabled,
                scope_level: Some(RBUM_SCOPE_LEVEL_TENANT),
                role_ids: Some(vec![funs.iam_basic_role_tenant_admin_id()]),
                org_node_ids: None,
                exts: Default::default(),
                status: None
            },
            funs,
            &tenant_ctx,
        )
        .await?;

        Ok((tenant_id, pwd))
    }

    pub async fn modify_tenant_agg(id: &str, modify_req: &IamTenantAggModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        Self::modify_item(
            id,
            &mut IamTenantModifyReq {
                name: modify_req.name.clone(),
                scope_level: None,
                disabled: modify_req.disabled,
                icon: modify_req.icon.clone(),
                sort: modify_req.sort,
                contact_phone: modify_req.contact_phone.clone(),
                note: modify_req.note.clone(),
                account_self_reg: modify_req.account_self_reg,
            },
            funs,
            ctx,
        )
        .await?;

        if modify_req.cert_conf_by_user_pwd.is_none() && modify_req.cert_conf_by_phone_vcode.is_none() && modify_req.cert_conf_by_mail_vcode.is_none() {
            return Ok(());
        }

        // Init cert conf
        let cert_confs = IamCertServ::find_cert_conf(true, Some(id.to_string()), None, None, funs, ctx).await?;

        // todo cert conf delete change disable status
        if let Some(cert_conf_by_user_pwd) = &modify_req.cert_conf_by_user_pwd {
            let cert_conf_by_user_pwd_id = cert_confs.iter().find(|r| r.code == IamCertKernelKind::UserPwd.to_string()).map(|r| r.id.clone()).unwrap();
            IamCertUserPwdServ::modify_cert_conf(&cert_conf_by_user_pwd_id, cert_conf_by_user_pwd, funs, ctx).await?;
        }
        if let Some(cert_conf_by_phone_vcode) = modify_req.cert_conf_by_phone_vcode {
            if let Some(cert_conf_by_phone_vcode_id) = cert_confs.iter().find(|r| r.code == IamCertKernelKind::PhoneVCode.to_string()).map(|r| r.id.clone()) {
                if !cert_conf_by_phone_vcode {
                    IamCertServ::delete_cert_conf(&cert_conf_by_phone_vcode_id, funs, ctx).await?;
                }
            } else if cert_conf_by_phone_vcode {
                IamCertPhoneVCodeServ::add_cert_conf(&IamCertConfPhoneVCodeAddOrModifyReq { ak_note: None, ak_rule: None }, Some(id.to_string()), funs, ctx).await?;
            }
        }

        if let Some(cert_conf_by_mail_vcode) = modify_req.cert_conf_by_mail_vcode {
            if let Some(cert_conf_by_mail_vcode_id) = cert_confs.iter().find(|r| r.code == IamCertKernelKind::MailVCode.to_string()).map(|r| r.id.clone()) {
                if !cert_conf_by_mail_vcode {
                    IamCertServ::delete_cert_conf(&cert_conf_by_mail_vcode_id, funs, ctx).await?;
                }
            } else if cert_conf_by_mail_vcode {
                IamCertMailVCodeServ::add_cert_conf(&IamCertConfMailVCodeAddOrModifyReq { ak_note: None, ak_rule: None }, Some(id.to_string()), funs, ctx).await?;
            }
        }

        if let Some(cert_conf_by_wechat_mp) = &modify_req.cert_conf_by_wechat_mp {
            if let Some(cert_conf_by_wechat_mp_id) = cert_confs.iter().find(|r| r.code == IamCertExtKind::WechatMp.to_string()).map(|r| r.id.clone()) {
                IamCertOAuth2Serv::modify_cert_conf(&cert_conf_by_wechat_mp_id, cert_conf_by_wechat_mp, funs, ctx).await?;
            } else {
                IamCertOAuth2Serv::add_cert_conf(IamCertExtKind::WechatMp, cert_conf_by_wechat_mp, id.to_string(), funs, ctx).await?;
            }
        } else if let Some(cert_conf_by_wechat_mp_id) = cert_confs.iter().find(|r| r.code == IamCertExtKind::WechatMp.to_string()).map(|r| r.id.clone()) {
            IamCertServ::delete_cert_conf(&cert_conf_by_wechat_mp_id, funs, ctx).await?;
        }

        //modify ldap config
        //The current ldap related configuration in the database/过滤出现在数据库中ldap相关的配置
        let old_cert_conf_by_ldaps: Vec<_> = cert_confs.iter().filter(|r| r.code.contains(&IamCertExtKind::Ldap.to_string())).collect();
        let cert_conf_by_ldap_code_id_map = old_cert_conf_by_ldaps.iter().map(|r| (r.code.clone(), r.id.clone())).collect::<HashMap<String, String>>();
        if let Some(cert_conf_by_ldaps) = &modify_req.cert_conf_by_ldap {
            if !cert_conf_by_ldaps.is_empty() {
                //get intersection of modify request certificate configuration and database certificate configuration/获取修改request和数据库中配置的交集
                let modify_cert_conf_by_ldap = cert_conf_by_ldaps.iter().filter(|r| cert_conf_by_ldap_code_id_map.contains_key(&r.code.to_string())).collect::<Vec<_>>();
                for modify in modify_cert_conf_by_ldap {
                    IamCertLdapServ::modify_cert_conf(cert_conf_by_ldap_code_id_map.get(&modify.code.to_string()).unwrap(), modify, funs, ctx).await?;
                }

                let add_cert_conf_by_ldap = cert_conf_by_ldaps.iter().filter(|r| !cert_conf_by_ldap_code_id_map.contains_key(&r.code.to_string())).collect::<Vec<_>>();
                for add in add_cert_conf_by_ldap {
                    IamCertLdapServ::add_cert_conf(add, id.to_string(), funs, ctx).await?;
                }

                let delete_cert_conf_code_by_ldap = cert_conf_by_ldap_code_id_map
                    .iter()
                    .map(|(k, _)| k)
                    .filter(|r| !cert_conf_by_ldaps.iter().map(|y| y.code.clone().to_string()).any(|x| x == r.to_string()))
                    .collect::<Vec<_>>();
                for delete_code in delete_cert_conf_code_by_ldap {
                    IamCertServ::delete_cert_conf(cert_conf_by_ldap_code_id_map.get(delete_code).unwrap(), funs, ctx).await?;
                }
            } else {
                for delete_id in old_cert_conf_by_ldaps.iter().map(|r| r.id.clone()).collect::<Vec<String>>() {
                    IamCertServ::delete_cert_conf(&delete_id, funs, ctx).await?;
                }
            }
        }

        Ok(())
    }

    pub async fn get_tenant_agg(id: &str, filter: &IamTenantFilterReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<IamTenantAggDetailResp> {
        let tenant = Self::get_item(id, filter, funs, ctx).await?;
        let cert_confs = IamCertServ::find_cert_conf(true, Some(id.to_string()), None, None, funs, ctx).await?;
        let cert_conf_by_user_pwd = cert_confs.iter().find(|r| r.code == IamCertKernelKind::UserPwd.to_string()).unwrap();

        let cert_conf_by_wechat_mp = if let Some(cert_conf_by_wechat_mp) = cert_confs.iter().find(|r| r.code == IamCertExtKind::WechatMp.to_string()) {
            Some(IamCertOAuth2Serv::get_cert_conf(&cert_conf_by_wechat_mp.id, funs, ctx).await?)
        } else {
            None
        };

        let tenant = IamTenantAggDetailResp {
            id: tenant.id.clone(),
            name: tenant.name.clone(),
            own_paths: tenant.own_paths.clone(),
            owner: tenant.owner.clone(),
            owner_name: tenant.owner_name.clone(),
            create_time: tenant.create_time,
            update_time: tenant.update_time,
            disabled: tenant.disabled,
            icon: tenant.icon.clone(),
            sort: tenant.sort,
            contact_phone: tenant.contact_phone.clone(),
            note: tenant.note.clone(),
            account_self_reg: tenant.account_self_reg,
            cert_conf_by_user_pwd: TardisFuns::json.str_to_obj(&cert_conf_by_user_pwd.ext)?,
            cert_conf_by_phone_vcode: cert_confs.iter().any(|r| r.code == IamCertKernelKind::PhoneVCode.to_string()),
            cert_conf_by_mail_vcode: cert_confs.iter().any(|r| r.code == IamCertKernelKind::MailVCode.to_string()),
            cert_conf_by_wechat_mp,
        };

        Ok(tenant)
    }
}
