use async_trait::async_trait;
use bios_basic::rbum::dto::rbum_filer_dto::RbumBasicFilterReq;
use bios_basic::rbum::rbum_enumeration::RbumCertStatusKind;
use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;
use std::collections::HashMap;
use tardis::basic::dto::TardisContext;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::db::sea_orm::sea_query::{Expr, SelectStatement};
use tardis::db::sea_orm::*;
use tardis::{TardisFuns, TardisFunsInst};

use bios_basic::rbum::dto::rbum_item_dto::{RbumItemKernelAddReq, RbumItemKernelModifyReq};
use bios_basic::rbum::helper::rbum_scope_helper;

use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;

use crate::basic::domain::iam_tenant;
use crate::basic::dto::iam_account_dto::IamAccountAggAddReq;
use crate::basic::dto::iam_cert_conf_dto::{IamCertConfLdapResp, IamCertConfMailVCodeAddOrModifyReq, IamCertConfPhoneVCodeAddOrModifyReq, IamCertConfUserPwdAddOrModifyReq};
use crate::basic::dto::iam_config_dto::IamConfigAggOrModifyReq;
use crate::basic::dto::iam_filer_dto::{IamConfigFilterReq, IamTenantFilterReq};
use crate::basic::dto::iam_tenant_dto::{
    IamTenantAddReq, IamTenantAggAddReq, IamTenantAggDetailResp, IamTenantAggModifyReq, IamTenantConfigReq, IamTenantConfigResp, IamTenantDetailResp, IamTenantModifyReq,
    IamTenantSummaryResp,
};
#[cfg(feature = "spi_kv")]
use crate::basic::serv::clients::spi_kv_client::SpiKvClient;
use crate::basic::serv::iam_account_serv::IamAccountServ;
use crate::basic::serv::iam_cert_ldap_serv::IamCertLdapServ;
use crate::basic::serv::iam_cert_mail_vcode_serv::IamCertMailVCodeServ;
use crate::basic::serv::iam_cert_phone_vcode_serv::IamCertPhoneVCodeServ;
use crate::basic::serv::iam_cert_serv::IamCertServ;
use crate::basic::serv::iam_cert_user_pwd_serv::IamCertUserPwdServ;
use crate::basic::serv::iam_key_cache_serv::IamIdentCacheServ;
use crate::basic::serv::iam_set_serv::IamSetServ;
use crate::iam_config::{IamBasicConfigApi, IamBasicInfoManager, IamConfig};
use crate::iam_constants;
use crate::iam_constants::{RBUM_ITEM_ID_TENANT_LEN, RBUM_SCOPE_LEVEL_TENANT};
use crate::iam_enumeration::{IamCertExtKind, IamCertKernelKind, IamCertOAuth2Supplier, IamConfigDataTypeKind, IamConfigKind, IamSetKind};

use super::clients::spi_log_client::{LogParamTag, SpiLogClient};
use super::iam_cert_oauth2_serv::IamCertOAuth2Serv;
use super::iam_config_serv::IamConfigServ;
use super::iam_platform_serv::IamPlatformServ;

pub struct IamTenantServ;

#[async_trait]
impl RbumItemCrudOperation<iam_tenant::ActiveModel, IamTenantAddReq, IamTenantModifyReq, IamTenantSummaryResp, IamTenantDetailResp, IamTenantFilterReq> for IamTenantServ {
    fn get_ext_table_name() -> &'static str {
        iam_tenant::Entity.table_name()
    }

    fn get_rbum_kind_id() -> Option<String> {
        Some(IamBasicInfoManager::get_config(|conf| conf.kind_tenant_id.clone()))
    }

    fn get_rbum_domain_id() -> Option<String> {
        Some(IamBasicInfoManager::get_config(|conf| conf.domain_iam_id.clone()))
    }

    async fn package_item_add(add_req: &IamTenantAddReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<RbumItemKernelAddReq> {
        Ok(RbumItemKernelAddReq {
            id: add_req.id.clone(),
            name: add_req.name.clone(),
            scope_level: add_req.scope_level.clone(),
            disabled: add_req.disabled,
            ..Default::default()
        })
    }

    async fn package_ext_add(id: &str, add_req: &IamTenantAddReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<iam_tenant::ActiveModel> {
        Ok(iam_tenant::ActiveModel {
            id: Set(id.to_string()),
            icon: Set(add_req.icon.as_ref().unwrap_or(&"".to_string()).to_string()),
            sort: Set(add_req.sort.unwrap_or(0)),
            contact_phone: Set(add_req.contact_phone.as_ref().unwrap_or(&"".to_string()).to_string()),
            note: Set(add_req.note.as_ref().unwrap_or(&"".to_string()).to_string()),
            account_self_reg: Set(add_req.account_self_reg.unwrap_or(true)),
            ..Default::default()
        })
    }

    async fn package_item_modify(_: &str, modify_req: &IamTenantModifyReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<Option<RbumItemKernelModifyReq>> {
        if modify_req.name.is_none() && modify_req.scope_level.is_none() && modify_req.disabled.is_none() {
            return Ok(None);
        }
        Ok(Some(RbumItemKernelModifyReq {
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

    async fn after_add_item(id: &str, _: &mut IamTenantAddReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        #[cfg(feature = "spi_kv")]
        Self::add_or_modify_tenant_kv(id, funs, ctx).await?;
        let _ = SpiLogClient::add_ctx_task(LogParamTag::IamTenant, Some(id.to_string()), "添加租户".to_string(), Some("Add".to_string()), ctx).await;

        Ok(())
    }
    async fn after_modify_item(id: &str, modify_req: &mut IamTenantModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        if modify_req.disabled.unwrap_or(false) {
            IamIdentCacheServ::delete_tokens_and_contexts_by_tenant_or_app(id, false, funs, ctx).await?;
        }
        #[cfg(feature = "spi_kv")]
        Self::add_or_modify_tenant_kv(id, funs, ctx).await?;

        let mut op_describe = "编辑租户".to_string();
        let mut op_kind = "Modify".to_string();
        if modify_req.disabled == Some(false) {
            op_describe = "禁用租户".to_string();
            op_kind = "Disabled".to_string();
        } else if modify_req.disabled == Some(true) {
            op_describe = "启用租户".to_string();
            op_kind = "Enabled".to_string();
        }
        let _ = SpiLogClient::add_ctx_task(LogParamTag::IamTenant, Some(id.to_string()), op_describe, Some(op_kind), ctx).await;

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

    pub async fn add_tenant_agg(add_req: &IamTenantAggAddReq, funs: &TardisFunsInst) -> TardisResult<(String, String, String)> {
        let tenant_admin_id = TardisFuns::field.nanoid();
        let tenant_audit_id = TardisFuns::field.nanoid();
        // TODO security check
        let tenant_id = IamTenantServ::get_new_id();
        let platform_ctx = TardisContext {
            own_paths: "".to_string(),
            ak: "".to_string(),
            roles: vec![],
            groups: vec![],
            owner: tenant_admin_id.to_string(),
            ..Default::default()
        };
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

        // // Init cert conf
        let platform_config = IamPlatformServ::get_platform_config_agg(funs, &platform_ctx).await?;

        let cert_conf_by_phone_vcode = if platform_config.cert_conf_by_phone_vcode {
            Some(IamCertConfPhoneVCodeAddOrModifyReq { ak_note: None, ak_rule: None })
        } else {
            None
        };
        let cert_conf_by_mail_vcode = if platform_config.cert_conf_by_mail_vcode {
            Some(IamCertConfMailVCodeAddOrModifyReq { ak_note: None, ak_rule: None })
        } else {
            None
        };
        IamCertServ::init_default_ident_conf(
            &mut IamCertConfUserPwdAddOrModifyReq {
                ak_rule_len_min: platform_config.cert_conf_by_user_pwd.ak_rule_len_min,
                ak_rule_len_max: platform_config.cert_conf_by_user_pwd.ak_rule_len_max,
                sk_rule_len_min: platform_config.cert_conf_by_user_pwd.sk_rule_len_min,
                sk_rule_len_max: platform_config.cert_conf_by_user_pwd.sk_rule_len_max,
                sk_rule_need_num: platform_config.cert_conf_by_user_pwd.sk_rule_need_num,
                sk_rule_need_uppercase: platform_config.cert_conf_by_user_pwd.sk_rule_need_uppercase,
                sk_rule_need_lowercase: platform_config.cert_conf_by_user_pwd.sk_rule_need_lowercase,
                sk_rule_need_spec_char: platform_config.cert_conf_by_user_pwd.sk_rule_need_spec_char,
                sk_lock_cycle_sec: platform_config.cert_conf_by_user_pwd.sk_lock_cycle_sec,
                sk_lock_err_times: platform_config.cert_conf_by_user_pwd.sk_lock_err_times,
                sk_lock_duration_sec: platform_config.cert_conf_by_user_pwd.sk_lock_duration_sec,
                repeatable: platform_config.cert_conf_by_user_pwd.repeatable,
                expire_sec: platform_config.cert_conf_by_user_pwd.expire_sec,
            },
            cert_conf_by_phone_vcode,
            cert_conf_by_mail_vcode,
            None,
            funs,
            &tenant_ctx,
        )
        .await?;

        if let Some(cert_conf_by_oauth2) = &add_req.cert_conf_by_oauth2 {
            for add_req in cert_conf_by_oauth2 {
                IamCertOAuth2Serv::add_or_enable_cert_conf(IamCertOAuth2Supplier::WechatMp, add_req, &tenant_id, funs, &tenant_ctx).await?;
            }
        }
        if let Some(cert_conf_by_ldap) = &add_req.cert_conf_by_ldap {
            IamCertLdapServ::add_cert_conf(cert_conf_by_ldap, tenant_id.clone().into(), funs, &tenant_ctx).await?;
        }

        // Init tenant config
        IamConfigServ::add_or_modify_batch(
            &tenant_id,
            platform_config
                .config
                .iter()
                .map(|r| IamConfigAggOrModifyReq {
                    name: Some(r.name.clone()),
                    note: Some(r.note.clone()),
                    value1: Some(r.value1.clone()),
                    value2: Some(r.value2.clone()),
                    ext: Some(r.ext.clone()),
                    disabled: Some(r.disabled),
                    data_type: IamConfigDataTypeKind::parse(&r.data_type).unwrap(),
                    code: IamConfigKind::parse(&r.code).unwrap(),
                })
                .collect::<Vec<IamConfigAggOrModifyReq>>(),
            funs,
            &tenant_ctx,
        )
        .await?;

        // Init admin pwd
        let admin_pwd: String = if let Some(admin_password) = &add_req.admin_password {
            admin_password.to_string()
        } else {
            IamCertServ::get_new_pwd()
        };
        let admin_id = IamAccountServ::add_account_agg(
            &IamAccountAggAddReq {
                id: Some(TrimString(tenant_admin_id.clone())),
                name: add_req.admin_name.clone(),
                cert_user_name: TrimString(add_req.admin_username.0.to_string()),
                cert_password: Some(TrimString(admin_pwd.to_string())),
                cert_phone: add_req.admin_phone.clone(),
                cert_mail: add_req.admin_mail.clone(),
                icon: None,
                disabled: add_req.disabled,
                scope_level: Some(RBUM_SCOPE_LEVEL_TENANT),
                role_ids: Some(vec![funs.iam_basic_role_tenant_admin_id()]),
                org_node_ids: None,
                exts: Default::default(),
                status: Some(RbumCertStatusKind::Pending),
                temporary: None,
                lock_status: None,
            },
            true,
            funs,
            &tenant_ctx,
        )
        .await?;
        // Init audit pwd
        let audit_pwd: String = if let Some(audit_password) = &add_req.audit_password {
            audit_password.to_string()
        } else {
            IamCertServ::get_new_pwd()
        };
        let audit_id = IamAccountServ::add_account_agg(
            &IamAccountAggAddReq {
                id: Some(TrimString(tenant_audit_id.clone())),
                name: add_req.audit_name.clone(),
                cert_user_name: TrimString(add_req.audit_username.0.to_string()),
                cert_password: Some(TrimString(audit_pwd.to_string())),
                cert_phone: add_req.audit_phone.clone(),
                cert_mail: add_req.audit_mail.clone(),
                icon: None,
                disabled: add_req.disabled,
                scope_level: Some(RBUM_SCOPE_LEVEL_TENANT),
                role_ids: Some(vec![funs.iam_basic_role_tenant_audit_id()]),
                org_node_ids: None,
                exts: Default::default(),
                status: Some(RbumCertStatusKind::Pending),
                temporary: None,
                lock_status: None,
            },
            true,
            funs,
            &tenant_ctx,
        )
        .await?;
        IamAccountServ::async_add_or_modify_account_search(admin_id, false, "".to_string(), funs, tenant_ctx.clone()).await?;
        IamAccountServ::async_add_or_modify_account_search(audit_id, false, "".to_string(), funs, tenant_ctx).await?;
        Ok((tenant_id, admin_pwd, audit_pwd))
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
        Ok(())
    }

    pub async fn modify_tenant_config_agg(id: &str, modify_req: &IamTenantConfigReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        //option conf,fast return
        if modify_req.cert_conf_by_user_pwd.is_none()
            && modify_req.cert_conf_by_phone_vcode.is_none()
            && modify_req.cert_conf_by_mail_vcode.is_none()
            && modify_req.cert_conf_by_oauth2.is_none()
            && modify_req.cert_conf_by_ldap.is_none()
        {
            return Ok(());
        }

        // Init cert conf
        let cert_confs = IamCertServ::find_cert_conf(true, Some(id.to_string()), None, None, funs, ctx).await?;

        if let Some(cert_conf_by_user_pwd) = &modify_req.cert_conf_by_user_pwd {
            let cert_conf_by_user_pwd_id = cert_confs.iter().find(|r| r.kind == IamCertKernelKind::UserPwd.to_string()).map(|r| r.id.clone()).unwrap();
            IamCertUserPwdServ::modify_cert_conf(&cert_conf_by_user_pwd_id, cert_conf_by_user_pwd, funs, ctx).await?;
        }
        if let Some(cert_conf_by_phone_vcode) = modify_req.cert_conf_by_phone_vcode {
            if let Some(cert_conf_by_phone_vcode_id) = cert_confs.iter().find(|r| r.kind == IamCertKernelKind::PhoneVCode.to_string()).map(|r| r.id.clone()) {
                if !cert_conf_by_phone_vcode {
                    IamCertServ::disable_cert_conf(&cert_conf_by_phone_vcode_id, funs, ctx).await?;
                }
            } else if cert_conf_by_phone_vcode {
                IamCertPhoneVCodeServ::add_or_enable_cert_conf(&IamCertConfPhoneVCodeAddOrModifyReq { ak_note: None, ak_rule: None }, Some(id.into()), funs, ctx).await?;
            }
        }

        if let Some(cert_conf_by_mail_vcode) = modify_req.cert_conf_by_mail_vcode {
            if let Some(cert_conf_by_mail_vcode_id) = cert_confs.iter().find(|r| r.kind == IamCertKernelKind::MailVCode.to_string()).map(|r| r.id.clone()) {
                if !cert_conf_by_mail_vcode {
                    IamCertServ::disable_cert_conf(&cert_conf_by_mail_vcode_id, funs, ctx).await?;
                }
            } else if cert_conf_by_mail_vcode {
                IamCertMailVCodeServ::add_or_enable_cert_conf(&IamCertConfMailVCodeAddOrModifyReq { ak_note: None, ak_rule: None }, Some(id.into()), funs, ctx).await?;
            }
        }

        //modify oauth2 config
        //The current oauth2 related configuration in the database/过滤出现在数据库中oauth2相关的配置
        let old_cert_conf_by_oauth2: Vec<_> = cert_confs.iter().filter(|r| r.kind == IamCertExtKind::OAuth2.to_string()).collect();
        let cert_conf_by_oauth2_supplier_id_map = old_cert_conf_by_oauth2.iter().map(|r| (r.supplier.clone(), r.id.clone())).collect::<HashMap<String, String>>();
        if let Some(cert_conf_by_oauth2) = &modify_req.cert_conf_by_oauth2 {
            if !cert_conf_by_oauth2.is_empty() {
                //get intersection of modify request certificate configuration and database certificate configuration/获取修改request和数据库中配置的交集
                let modify_cert_conf_by_oauth2 =
                    cert_conf_by_oauth2.iter().filter(|r| cert_conf_by_oauth2_supplier_id_map.contains_key(&r.supplier.to_string())).collect::<Vec<_>>();
                for modify in modify_cert_conf_by_oauth2 {
                    IamCertOAuth2Serv::modify_cert_conf(cert_conf_by_oauth2_supplier_id_map.get(&modify.supplier.to_string()).unwrap(), modify, funs, ctx).await?;
                }

                let add_cert_conf_by_oauth2 = cert_conf_by_oauth2.iter().filter(|r| !cert_conf_by_oauth2_supplier_id_map.contains_key(&r.supplier.to_string())).collect::<Vec<_>>();
                for add in add_cert_conf_by_oauth2 {
                    IamCertOAuth2Serv::add_or_enable_cert_conf(IamCertOAuth2Supplier::parse(&add.supplier)?, add, id, funs, ctx).await?;
                }

                let delete_cert_conf_code_by_oauth2 = cert_conf_by_oauth2_supplier_id_map
                    .keys()
                    .filter(|r| !cert_conf_by_oauth2.iter().map(|y| y.supplier.clone().to_string()).any(|x| x == r.to_string()))
                    .collect::<Vec<_>>();
                for delete in delete_cert_conf_code_by_oauth2 {
                    IamCertServ::disable_cert_conf(cert_conf_by_oauth2_supplier_id_map.get(delete).unwrap(), funs, ctx).await?;
                }
            } else {
                for delete_id in old_cert_conf_by_oauth2.iter().map(|r| r.id.clone()).collect::<Vec<String>>() {
                    IamCertServ::disable_cert_conf(&delete_id, funs, ctx).await?;
                }
            }
        }

        //ldap only can be one recode in each tenant
        if let Some(cert_conf_by_ladp) = &modify_req.cert_conf_by_ldap {
            if let Some(cert_conf_id_by_ldap) = cert_confs.iter().find(|r| r.kind == IamCertExtKind::Ldap.to_string()).map(|r| r.id.clone()) {
                IamCertLdapServ::modify_cert_conf(&cert_conf_id_by_ldap, cert_conf_by_ladp, funs, ctx).await?;
            } else {
                IamCertLdapServ::add_cert_conf(cert_conf_by_ladp, Some(id.to_string()), funs, ctx).await?;
            }
        }

        // modify config
        if let Some(config) = &modify_req.config {
            IamConfigServ::add_or_modify_batch(id, config.to_vec(), funs, ctx).await?;
        }
        Ok(())
    }

    pub async fn get_tenant_config_agg(id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<IamTenantConfigResp> {
        let cert_confs = IamCertServ::find_cert_conf(true, Some(id.to_owned()), None, None, funs, ctx).await?;
        let cert_conf_by_user_pwd = cert_confs.iter().find(|r| r.kind == IamCertKernelKind::UserPwd.to_string()).unwrap();
        let config = IamConfigServ::find_rbums(
            &IamConfigFilterReq {
                rel_item_id: Some(id.to_owned()),
                ..Default::default()
            },
            Some(true),
            None,
            funs,
            ctx,
        )
        .await?;
        let cert_conf_by_oauth2s = cert_confs.iter().filter(|r| r.kind == IamCertExtKind::OAuth2.to_string()).collect::<Vec<_>>();
        let cert_conf_by_oauth2 = if !cert_conf_by_oauth2s.is_empty() {
            let mut result = Vec::new();
            for cert_conf in cert_conf_by_oauth2s {
                result.push(IamCertOAuth2Serv::get_cert_conf(&cert_conf.id, funs, ctx).await?);
            }
            Some(result)
        } else {
            None
        };
        let mut vec1: Vec<IamCertConfLdapResp> = Vec::new();
        for ldap_conf in cert_confs.iter().filter(|r| r.kind == IamCertExtKind::Ldap.to_string()) {
            let conf = IamCertLdapServ::get_cert_conf(&ldap_conf.id, funs, ctx).await?;
            vec1.push(IamCertConfLdapResp {
                id: ldap_conf.id.clone(),
                supplier: ldap_conf.kind.clone(),
                conn_uri: ldap_conf.conn_uri.clone(),
                is_tls: conf.is_tls,
                timeout: conf.timeout,
                principal: conf.principal.clone(),
                credentials: "".to_string(),
                base_dn: conf.base_dn,
                port: conf.port,
                account_unique_id: conf.account_unique_id,
                account_field_map: conf.account_field_map,
                // org_unique_id: conf.org_unique_id,
                // org_field_map: conf.org_field_map,
            })
        }
        let cert_conf_by_ldap = if vec1.is_empty() { None } else { Some(vec1) };
        let tenant_config = IamTenantConfigResp {
            cert_conf_by_user_pwd: TardisFuns::json.str_to_obj(&cert_conf_by_user_pwd.ext)?,
            cert_conf_by_phone_vcode: cert_confs.iter().any(|r| r.kind == IamCertKernelKind::PhoneVCode.to_string()),
            cert_conf_by_mail_vcode: cert_confs.iter().any(|r| r.kind == IamCertKernelKind::MailVCode.to_string()),
            config,
            cert_conf_by_oauth2,
            cert_conf_by_ldap,
            strict_security_mode: funs.conf::<IamConfig>().strict_security_mode,
        };

        Ok(tenant_config)
    }

    pub async fn get_tenant_agg(id: &str, filter: &IamTenantFilterReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<IamTenantAggDetailResp> {
        let tenant = Self::get_item(id, filter, funs, ctx).await?;
        let cert_confs = IamCertServ::find_cert_conf(true, Some(id.to_string()), None, None, funs, ctx).await?;
        let cert_conf_by_user_pwd = cert_confs.iter().find(|r| r.kind == IamCertKernelKind::UserPwd.to_string()).unwrap();

        let cert_conf_by_oauth2s = cert_confs.iter().filter(|r| r.kind == IamCertExtKind::OAuth2.to_string()).collect::<Vec<_>>();
        let cert_conf_by_oauth2 = if !cert_conf_by_oauth2s.is_empty() {
            let mut result = Vec::new();
            for cert_conf in cert_conf_by_oauth2s {
                result.push(IamCertOAuth2Serv::get_cert_conf(&cert_conf.id, funs, ctx).await?);
            }
            Some(result)
        } else {
            None
        };
        let mut vec1: Vec<IamCertConfLdapResp> = Vec::new();
        for ldap_conf in cert_confs.iter().filter(|r| r.kind == IamCertExtKind::Ldap.to_string()) {
            let conf = IamCertLdapServ::get_cert_conf(&ldap_conf.id, funs, ctx).await?;
            vec1.push(IamCertConfLdapResp {
                id: ldap_conf.id.clone(),
                supplier: ldap_conf.kind.clone(),
                conn_uri: ldap_conf.conn_uri.clone(),
                is_tls: conf.is_tls,
                timeout: conf.timeout,
                principal: conf.principal.clone(),
                credentials: "".to_string(),
                base_dn: conf.base_dn,
                port: conf.port,
                account_unique_id: conf.account_unique_id,
                account_field_map: conf.account_field_map,
                // org_unique_id: conf.org_unique_id,
                // org_field_map: conf.org_field_map,
            })
        }
        let cert_conf_by_ldap = if vec1.is_empty() { None } else { Some(vec1) };

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
            cert_conf_by_phone_vcode: cert_confs.iter().any(|r| r.kind == IamCertKernelKind::PhoneVCode.to_string()),
            cert_conf_by_mail_vcode: cert_confs.iter().any(|r| r.kind == IamCertKernelKind::MailVCode.to_string()),
            cert_conf_by_oauth2,
            cert_conf_by_ldap,
        };

        Ok(tenant)
    }

    pub async fn find_name_by_ids(ids: Vec<String>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Vec<String>> {
        Self::find_items(
            &IamTenantFilterReq {
                basic: RbumBasicFilterReq {
                    ids: Some(ids),
                    with_sub_own_paths: true,
                    own_paths: Some("".to_string()),
                    ..Default::default()
                },
                ..Default::default()
            },
            None,
            None,
            funs,
            ctx,
        )
        .await
        .map(|r| r.into_iter().map(|r| format!("{},{}", r.id, r.name)).collect())
    }
    #[cfg(feature = "spi_kv")]
    async fn add_or_modify_tenant_kv(tenant_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let tenant = IamTenantServ::get_item(
            tenant_id,
            &IamTenantFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
                    own_paths: Some("".to_string()),
                    ..Default::default()
                },
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?;
        SpiKvClient::add_or_modify_key_name(&format!("{}:{tenant_id}", funs.conf::<IamConfig>().spi.kv_tenant_prefix.clone()), &tenant.name, funs, ctx).await?;
        Ok(())
    }
}
