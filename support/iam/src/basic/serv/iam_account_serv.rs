use async_trait::async_trait;
use bios_basic::rbum::rbum_config::RbumConfigApi;
use bios_basic::rbum::rbum_enumeration::RbumRelFromKind;
use itertools::Itertools;
use std::collections::HashMap;
use tardis::basic::dto::TardisContext;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::chrono::Utc;
use tardis::db::sea_orm::sea_query::{Expr, SelectStatement};
use tardis::db::sea_orm::*;
use tardis::serde_json::json;
use tardis::web::web_resp::{TardisPage, Void};
use tardis::{serde_json, TardisFuns, TardisFunsInst};

use bios_basic::rbum::dto::rbum_filer_dto::{RbumBasicFilterReq, RbumCertFilterReq, RbumItemRelFilterReq};
use bios_basic::rbum::dto::rbum_item_dto::{RbumItemKernelAddReq, RbumItemKernelModifyReq};
use bios_basic::rbum::dto::rbum_rel_dto::RbumRelBoneResp;
use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;
use bios_basic::rbum::serv::rbum_item_serv::{RbumItemCrudOperation, RbumItemServ};

use crate::basic::domain::iam_account;
use crate::basic::dto::iam_account_dto::{
    AccountTenantInfo, AccountTenantInfoResp, IamAccountAddReq, IamAccountAggAddReq, IamAccountAggModifyReq, IamAccountAppInfoResp, IamAccountAttrResp, IamAccountDetailAggResp,
    IamAccountDetailResp, IamAccountModifyReq, IamAccountSelfModifyReq, IamAccountSummaryAggResp, IamAccountSummaryResp,
};
use crate::basic::dto::iam_cert_dto::{IamCertMailVCodeAddReq, IamCertPhoneVCodeAddReq, IamCertUserPwdAddReq};
use crate::basic::dto::iam_filer_dto::{IamAccountFilterReq, IamAppFilterReq, IamTenantFilterReq};
use crate::basic::dto::iam_set_dto::IamSetItemAddReq;
use crate::basic::serv::iam_attr_serv::IamAttrServ;
use crate::basic::serv::iam_cert_mail_vcode_serv::IamCertMailVCodeServ;
use crate::basic::serv::iam_cert_phone_vcode_serv::IamCertPhoneVCodeServ;
use crate::basic::serv::iam_cert_serv::IamCertServ;
use crate::basic::serv::iam_cert_user_pwd_serv::IamCertUserPwdServ;
use crate::basic::serv::iam_key_cache_serv::IamIdentCacheServ;
use crate::basic::serv::iam_rel_serv::IamRelServ;
use crate::basic::serv::iam_role_serv::IamRoleServ;
use crate::basic::serv::iam_set_serv::IamSetServ;
use crate::basic::serv::iam_tenant_serv::IamTenantServ;
use crate::iam_config::{IamBasicInfoManager, IamConfig};
use crate::iam_enumeration::{IamCertKernelKind, IamRelKind, IamSetKind};

use super::iam_app_serv::IamAppServ;

pub struct IamAccountServ;

#[async_trait]
impl RbumItemCrudOperation<iam_account::ActiveModel, IamAccountAddReq, IamAccountModifyReq, IamAccountSummaryResp, IamAccountDetailResp, IamAccountFilterReq> for IamAccountServ {
    fn get_ext_table_name() -> &'static str {
        iam_account::Entity.table_name()
    }

    fn get_rbum_kind_id() -> Option<String> {
        Some(IamBasicInfoManager::get_config(|conf| conf.kind_account_id.clone()))
    }

    fn get_rbum_domain_id() -> Option<String> {
        Some(IamBasicInfoManager::get_config(|conf| conf.domain_iam_id.clone()))
    }

    async fn package_item_add(add_req: &IamAccountAddReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<RbumItemKernelAddReq> {
        Ok(RbumItemKernelAddReq {
            id: add_req.id.clone(),
            name: add_req.name.clone(),
            disabled: add_req.disabled,
            scope_level: add_req.scope_level.clone(),
            ..Default::default()
        })
    }

    async fn package_ext_add(id: &str, add_req: &IamAccountAddReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<iam_account::ActiveModel> {
        Ok(iam_account::ActiveModel {
            id: Set(id.to_string()),
            icon: Set(add_req.icon.as_ref().unwrap_or(&"".to_string()).to_string()),
            ext1_idx: Set("".to_string()),
            ext2_idx: Set("".to_string()),
            ext3_idx: Set("".to_string()),
            ext4: Set("".to_string()),
            ext5: Set("".to_string()),
            ext6: Set("".to_string()),
            ext7: Set("".to_string()),
            ext8: Set("".to_string()),
            ext9: Set("".to_string()),
            ..Default::default()
        })
    }

    async fn package_item_modify(_: &str, modify_req: &IamAccountModifyReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<Option<RbumItemKernelModifyReq>> {
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

    async fn package_ext_modify(id: &str, modify_req: &IamAccountModifyReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<Option<iam_account::ActiveModel>> {
        if modify_req.icon.is_none() {
            return Ok(None);
        }
        let mut iam_account = iam_account::ActiveModel {
            id: Set(id.to_string()),
            ..Default::default()
        };
        if let Some(icon) = &modify_req.icon {
            iam_account.icon = Set(icon.to_string());
        }
        Ok(Some(iam_account))
    }

    async fn after_modify_item(id: &str, modify_req: &mut IamAccountModifyReq, funs: &TardisFunsInst, _: &TardisContext) -> TardisResult<()> {
        if modify_req.disabled.is_some() || modify_req.scope_level.is_some() {
            IamIdentCacheServ::delete_tokens_and_contexts_by_account_id(id, funs).await?;
        }
        Ok(())
    }

    async fn before_delete_item(id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Option<IamAccountDetailResp>> {
        if id == ctx.owner {
            return Err(funs.err().conflict(&Self::get_obj_name(), "delete", "account invalid", "409-iam-current-can-not-account-delete"));
        }
        Ok(None)
    }

    async fn after_delete_item(id: &str, _: &Option<IamAccountDetailResp>, funs: &TardisFunsInst, _: &TardisContext) -> TardisResult<()> {
        IamIdentCacheServ::delete_tokens_and_contexts_by_account_id(id, funs).await?;
        Ok(())
    }

    async fn package_ext_query(query: &mut SelectStatement, _: bool, filter: &IamAccountFilterReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<()> {
        query.column((iam_account::Entity, iam_account::Column::Icon));
        query.column((iam_account::Entity, iam_account::Column::Ext1Idx));
        query.column((iam_account::Entity, iam_account::Column::Ext2Idx));
        query.column((iam_account::Entity, iam_account::Column::Ext3Idx));
        query.column((iam_account::Entity, iam_account::Column::Ext4));
        query.column((iam_account::Entity, iam_account::Column::Ext5));
        query.column((iam_account::Entity, iam_account::Column::Ext6));
        query.column((iam_account::Entity, iam_account::Column::Ext7));
        query.column((iam_account::Entity, iam_account::Column::Ext8));
        query.column((iam_account::Entity, iam_account::Column::Ext9));
        if let Some(icon) = &filter.icon {
            query.and_where(Expr::col(iam_account::Column::Icon).eq(icon.as_str()));
        }
        Ok(())
    }
}

impl IamAccountServ {
    /// if add_req.status is None.default is RbumCertStatusKind::Enabled
    pub async fn add_account_agg(add_req: &IamAccountAggAddReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<String> {
        let attrs = IamAttrServ::find_account_attrs(funs, ctx).await?;
        if attrs.iter().any(|i| i.required && !add_req.exts.contains_key(&i.name)) {
            return Err(funs.err().bad_request(&Self::get_obj_name(), "add", "missing required field", "400-iam-account-field-missing"));
        }
        let account_id = IamAccountServ::add_item(
            &mut IamAccountAddReq {
                id: add_req.id.clone(),
                name: add_req.name.clone(),
                scope_level: add_req.scope_level.clone(),
                disabled: add_req.disabled,
                icon: add_req.icon.clone(),
            },
            funs,
            ctx,
        )
        .await?;
        if let Some(cert_conf) = IamCertServ::get_cert_conf_id_and_ext_opt_by_kind(&IamCertKernelKind::UserPwd.to_string(), Some(ctx.own_paths.clone()), funs).await? {
            IamCertUserPwdServ::add_cert(
                &IamCertUserPwdAddReq {
                    ak: add_req.cert_user_name.clone(),
                    sk: add_req.cert_password.clone(),
                    status: add_req.status.clone(),
                },
                &account_id,
                Some(cert_conf.id),
                funs,
                ctx,
            )
            .await?;
        }
        if let Some(cert_phone) = &add_req.cert_phone {
            if let Some(cert_conf) = IamCertServ::get_cert_conf_id_and_ext_opt_by_kind(&IamCertKernelKind::PhoneVCode.to_string(), Some(ctx.own_paths.clone()), funs).await? {
                IamCertPhoneVCodeServ::add_cert(
                    &IamCertPhoneVCodeAddReq {
                        phone: TrimString(cert_phone.to_string()),
                    },
                    &account_id,
                    &cert_conf.id,
                    funs,
                    ctx,
                )
                .await?;
            }
        }
        if let Some(cert_mail) = &add_req.cert_mail {
            if let Some(cert_conf) = IamCertServ::get_cert_conf_id_and_ext_opt_by_kind(&IamCertKernelKind::MailVCode.to_string(), Some(ctx.own_paths.clone()), funs).await? {
                IamCertMailVCodeServ::add_cert(&IamCertMailVCodeAddReq { mail: cert_mail.to_string() }, &account_id, &cert_conf.id, funs, ctx).await?;
            }
        }
        if let Some(role_ids) = &add_req.role_ids {
            for role_id in role_ids {
                IamRoleServ::add_rel_account(role_id, &account_id, None, funs, ctx).await?;
            }
        }
        if let Some(org_cate_ids) = &add_req.org_node_ids {
            let set_id = IamSetServ::get_default_set_id_by_ctx(&IamSetKind::Org, funs, ctx).await?;
            for org_node_id in org_cate_ids {
                IamSetServ::add_set_item(
                    &IamSetItemAddReq {
                        set_id: set_id.clone(),
                        set_cate_id: org_node_id.to_string(),
                        sort: 0,
                        rel_rbum_item_id: account_id.to_string(),
                    },
                    funs,
                    ctx,
                )
                .await?;
            }
        }
        IamAttrServ::add_or_modify_account_attr_values(&account_id, add_req.exts.clone(), funs, ctx).await?;

        Self::add_or_modify_account_search(&account_id, false, funs, ctx).await.unwrap();
        Ok(account_id)
    }

    pub async fn modify_account_agg(id: &str, modify_req: &IamAccountAggModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        IamAccountServ::modify_item(
            id,
            &mut IamAccountModifyReq {
                name: modify_req.name.clone(),
                scope_level: modify_req.scope_level.clone(),
                disabled: modify_req.disabled,
                icon: modify_req.icon.clone(),
            },
            funs,
            ctx,
        )
        .await?;
        if let Some(input_role_ids) = &modify_req.role_ids {
            let stored_roles = Self::find_simple_rel_roles(id, true, None, None, funs, ctx).await?;
            let stored_role_ids: Vec<String> = stored_roles.into_iter().map(|r| r.rel_id).collect();
            for input_role_id in input_role_ids {
                if !stored_role_ids.contains(input_role_id) {
                    IamRoleServ::add_rel_account(input_role_id, id, None, funs, ctx).await?;
                }
            }
            for stored_role_id in stored_role_ids {
                if !input_role_ids.contains(&stored_role_id) {
                    IamRoleServ::delete_rel_account(&stored_role_id, id, None, funs, ctx).await?;
                }
            }
        }
        // TODO test
        if let Some(input_org_cate_ids) = &modify_req.org_cate_ids {
            let set_id = IamSetServ::get_default_set_id_by_ctx(&IamSetKind::Org, funs, ctx).await?;
            let stored_cates = IamSetServ::find_set_items(Some(set_id.clone()), None, Some(id.to_string()), false, funs, ctx).await?;
            let mut stored_cate_ids: Vec<String> = stored_cates.iter().map(|r| r.rel_rbum_set_cate_id.clone().unwrap_or_default()).collect();
            stored_cate_ids.dedup();
            for input_org_cate_id in input_org_cate_ids {
                if !stored_cate_ids.contains(input_org_cate_id) {
                    IamSetServ::add_set_item(
                        &IamSetItemAddReq {
                            set_id: set_id.clone(),
                            set_cate_id: input_org_cate_id.to_string(),
                            sort: 0,
                            rel_rbum_item_id: id.to_string(),
                        },
                        funs,
                        ctx,
                    )
                    .await?;
                }
            }
            let deleted_item_ids: Vec<String> =
                stored_cates.into_iter().filter(|r| !input_org_cate_ids.contains(&r.rel_rbum_set_cate_id.clone().unwrap_or_default())).map(|r| r.id).unique().collect();
            for deleted_item_id in deleted_item_ids {
                IamSetServ::delete_set_item(&deleted_item_id, funs, ctx).await?;
            }
        }
        if let Some(exts) = &modify_req.exts {
            IamAttrServ::add_or_modify_account_attr_values(id, exts.clone(), funs, ctx).await?;
        }
        Self::add_or_modify_account_search(id, false, funs, ctx).await.unwrap();
        Ok(())
    }

    pub async fn self_modify_account(modify_req: &mut IamAccountSelfModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let id = &ctx.owner;
        let account = IamAccountServ::peek_item(
            id,
            &IamAccountFilterReq {
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
        let mock_ctx = TardisContext {
            owner: account.id.clone(),
            own_paths: account.own_paths.clone(),
            ..ctx.clone()
        };
        IamAccountServ::modify_item(
            id,
            &mut IamAccountModifyReq {
                name: modify_req.name.clone(),
                icon: modify_req.icon.clone(),
                disabled: modify_req.disabled,
                scope_level: None,
            },
            funs,
            &mock_ctx,
        )
        .await?;
        IamAttrServ::add_or_modify_account_attr_values(id, modify_req.exts.clone(), funs, &mock_ctx).await?;
        Ok(())
    }

    // TODO To optimize
    pub async fn get_account_detail_aggs(
        account_id: &str,
        filter: &IamAccountFilterReq,
        use_sys_org: bool,
        use_sys_cert: bool,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<IamAccountDetailAggResp> {
        let account = IamAccountServ::get_item(account_id, filter, funs, ctx).await?;
        let set_id = if use_sys_org {
            IamSetServ::get_set_id_by_code(&IamSetServ::get_default_code(&IamSetKind::Org, ""), true, funs, ctx).await?
        } else {
            IamSetServ::get_default_set_id_by_ctx(&IamSetKind::Org, funs, ctx).await?
        };
        let raw_roles = Self::find_simple_rel_roles(&account.id, true, Some(true), None, funs, ctx).await?;
        let mut roles: Vec<RbumRelBoneResp> = vec![];
        for role in raw_roles {
            if !IamRoleServ::is_disabled(&role.rel_id, funs).await? {
                roles.push(role)
            }
        }

        let enabled_apps = IamAppServ::find_items(
            &IamAppFilterReq {
                basic: RbumBasicFilterReq {
                    ignore_scope: false,
                    rel_ctx_owner: false,
                    with_sub_own_paths: true,
                    enabled: Some(true),
                    ..Default::default()
                },
                rel: Some(RbumItemRelFilterReq {
                    rel_by_from: false,
                    is_left: false,
                    tag: Some(IamRelKind::IamAccountApp.to_string()),
                    from_rbum_kind: Some(RbumRelFromKind::Item),
                    rel_item_id: Some(account.id.clone()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            None,
            None,
            funs,
            ctx,
        )
        .await?;
        let mut apps: Vec<IamAccountAppInfoResp> = vec![];
        for app in enabled_apps {
            let mut mock_app_ctx = ctx.clone();
            mock_app_ctx.own_paths = app.own_paths.clone();
            let set_id = IamSetServ::get_set_id_by_code(&IamSetServ::get_default_code(&IamSetKind::Org, &app.own_paths), true, funs, &mock_app_ctx).await?;
            let groups = IamSetServ::find_flat_set_items(&set_id, account_id, true, funs, &mock_app_ctx).await?;
            apps.push(IamAccountAppInfoResp {
                app_id: app.id,
                app_name: app.name,
                app_icon: app.icon,
                roles: roles.iter().filter(|r| r.rel_own_paths == app.own_paths).map(|r| (r.rel_id.to_string(), r.rel_name.to_string())).collect(),
                groups,
            });
        }
        let account_attrs = IamAttrServ::find_account_attrs(funs, ctx).await?;
        let account_attr_values = IamAttrServ::find_account_attr_values(&account.id, funs, ctx).await?;

        let org_set_id = IamSetServ::get_set_id_by_code(&IamSetServ::get_default_code(&IamSetKind::Org, &ctx.own_paths), false, funs, ctx).await?;
        let groups = IamSetServ::find_flat_set_items(&org_set_id, &account.id, false, funs, ctx).await?;
        let account = IamAccountDetailAggResp {
            id: account.id.clone(),
            name: account.name,
            own_paths: account.own_paths,
            owner: account.owner,
            owner_name: account.owner_name,
            create_time: account.create_time,
            update_time: account.update_time,
            scope_level: account.scope_level,
            disabled: account.disabled,
            icon: account.icon,
            roles: roles.iter().filter(|r| r.rel_own_paths == ctx.own_paths).map(|r| (r.rel_id.to_string(), r.rel_name.to_string())).collect(),
            apps,
            groups,
            certs: IamCertServ::find_certs(
                &RbumCertFilterReq {
                    basic: RbumBasicFilterReq {
                        own_paths: Some(if use_sys_cert { "".to_string() } else { IamTenantServ::get_id_by_ctx(ctx, funs)? }),
                        with_sub_own_paths: true,
                        ..Default::default()
                    },
                    rel_rbum_id: Some(account.id.clone()),
                    ..Default::default()
                },
                None,
                None,
                funs,
                ctx,
            )
            .await?
            .into_iter()
            .map(|r| (r.rel_rbum_cert_conf_code.unwrap(), r.ak))
            .collect(),
            orgs: IamSetServ::find_set_paths(&account.id, &set_id, funs, ctx).await?.into_iter().map(|r| r.into_iter().map(|rr| rr.name).join("/")).collect(),
            exts: account_attrs
                .into_iter()
                .map(|r| IamAccountAttrResp {
                    name: r.name.clone(),
                    label: r.label,
                    value: account_attr_values.get(&r.name).unwrap_or(&("".to_string())).to_string(),
                })
                .collect(),
        };
        Ok(account)
    }

    pub async fn paginate_account_summary_aggs(
        filter: &IamAccountFilterReq,
        use_sys_org: bool,
        use_sys_cert: bool,
        page_number: u32,
        page_size: u32,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<TardisPage<IamAccountSummaryAggResp>> {
        let accounts = IamAccountServ::paginate_items(filter, page_number, page_size, desc_sort_by_create, desc_sort_by_update, funs, ctx).await?;
        let mut account_aggs = Vec::with_capacity(accounts.total_size as usize);
        let set_id = if use_sys_org {
            IamSetServ::get_set_id_by_code(&IamSetServ::get_default_code(&IamSetKind::Org, ""), true, funs, ctx).await?
        } else {
            IamSetServ::get_default_set_id_by_ctx(&IamSetKind::Org, funs, ctx).await?
        };
        for account in accounts.records {
            account_aggs.push(IamAccountSummaryAggResp {
                id: account.id.clone(),
                name: account.name,
                own_paths: account.own_paths,
                owner: account.owner,
                create_time: account.create_time,
                update_time: account.update_time,
                scope_level: account.scope_level,
                disabled: account.disabled,
                icon: account.icon,
                roles: Self::find_simple_rel_roles(&account.id, true, None, None, funs, ctx).await?.into_iter().map(|r| (r.rel_id, r.rel_name)).collect(),
                certs: IamCertServ::find_certs(
                    &RbumCertFilterReq {
                        basic: RbumBasicFilterReq {
                            own_paths: if use_sys_cert {
                                Some("".to_string())
                            } else {
                                Some(IamTenantServ::get_id_by_ctx(ctx, funs)?)
                            },
                            with_sub_own_paths: use_sys_cert,
                            ..Default::default()
                        },
                        rel_rbum_id: Some(account.id.clone()),
                        ..Default::default()
                    },
                    None,
                    None,
                    funs,
                    ctx,
                )
                .await?
                .into_iter()
                .map(|r| (r.rel_rbum_cert_conf_code.unwrap(), r.ak))
                .collect(),
                orgs: IamSetServ::find_set_paths(&account.id, &set_id, funs, ctx).await?.into_iter().map(|r| r.into_iter().map(|rr| rr.name).join("/")).collect(),
                is_locked: funs.cache().exists(&format!("{}{}", funs.rbum_conf_cache_key_cert_locked_(), &account.id)).await?,
            });
        }
        Ok(TardisPage {
            page_size: accounts.page_size,
            page_number: accounts.page_number,
            total_size: accounts.total_size,
            records: account_aggs,
        })
    }

    pub async fn get_account_tenant_info(id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<AccountTenantInfoResp> {
        let mut tenant_ids = Vec::new();
        let raw_roles = Self::find_simple_rel_roles(id, true, Some(true), None, funs, ctx).await?;
        let mut roles: Vec<RbumRelBoneResp> = vec![];
        for role in raw_roles {
            if !IamRoleServ::is_disabled(&role.rel_id, funs).await? {
                let rel_own_path = role.rel_own_paths.clone();
                let tenant_id = rel_own_path.split('/').collect::<Vec<_>>();
                if !tenant_ids.contains(&tenant_id[0].to_string()) && !tenant_id[0].is_empty() {
                    tenant_ids.push(tenant_id[0].to_string());
                }
                roles.push(role);
            }
        }

        let mut tenant_info = HashMap::new();
        for tenant_id in tenant_ids {
            let tenant_ctx = IamCertServ::use_tenant_ctx(ctx.clone(), &tenant_id)?;
            let tenant_result = IamAccountServ::get_account_detail_aggs(
                id,
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
                &tenant_ctx,
            )
            .await?;
            let account_tenant_info = AccountTenantInfo {
                roles: tenant_result.roles,
                orgs: tenant_result.orgs,
                groups: tenant_result.groups,
                apps: tenant_result.apps,
            };
            tenant_info.insert(
                IamTenantServ::peek_item(&tenant_id, &IamTenantFilterReq::default(), funs, ctx).await?.name,
                account_tenant_info,
            );
        }

        let tenant_info = AccountTenantInfoResp { tenant_info };
        Ok(tenant_info)
    }

    pub async fn find_name_by_ids(ids: Vec<String>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Vec<String>> {
        IamAccountServ::find_items(
            &IamAccountFilterReq {
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
        .map(|r| r.into_iter().map(|r| format!("{},{},{}", r.id, r.name, r.icon)).collect())
    }

    pub async fn find_simple_rel_roles(
        account_id: &str,
        with_sub: bool,
        desc_by_create: Option<bool>,
        desc_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<Vec<RbumRelBoneResp>> {
        IamRelServ::find_from_simple_rels(&IamRelKind::IamAccountRole, with_sub, account_id, desc_by_create, desc_by_update, funs, ctx).await
    }

    pub async fn delete_tokens(id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        RbumItemServ::check_ownership(id, funs, ctx).await?;
        IamIdentCacheServ::delete_tokens_and_contexts_by_account_id(id, funs).await
    }

    pub async fn unlock_account(id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Void> {
        RbumItemServ::check_ownership(id, funs, ctx).await?;
        funs.cache().del(&format!("{}{}", funs.rbum_conf_cache_key_cert_locked_(), id)).await?;
        Ok(Void {})
    }

    /// return true means account is global account
    pub async fn is_global_account(account_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<bool> {
        if let Some(bool_) = funs.cache().hget(format!("{}{}", funs.conf::<IamConfig>().cache_key_account_info_, account_id).as_str(), "is_global").await? {
            let bool_ = bool_.parse::<bool>();
            Ok(bool_.unwrap_or(false))
        } else {
            let account = IamAccountServ::get_item(
                account_id,
                &IamAccountFilterReq {
                    basic: RbumBasicFilterReq {
                        ignore_scope: true,
                        with_sub_own_paths: true,
                        own_paths: "".to_string().into(),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                funs,
                ctx,
            )
            .await?;
            let is_global = account.own_paths.is_empty();
            funs.cache()
                .hset(
                    format!("{}{}", funs.conf::<IamConfig>().cache_key_account_info_, account_id).as_str(),
                    "is_global",
                    &is_global.to_string(),
                )
                .await?;
            Ok(is_global)
        }
    }

    pub async fn new_context_if_account_is_global(ctx: &TardisContext, funs: &TardisFunsInst) -> TardisResult<TardisContext> {
        if Self::is_global_account(&ctx.owner, funs, ctx).await? {
            let mut result = ctx.clone();
            result.own_paths = "".to_string();
            Ok(result)
        } else {
            Ok(ctx.clone())
        }
    }

    // account 全局搜索埋点方法
    pub async fn add_or_modify_account_search(account_id: &str, is_modify: bool, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let account_resp = IamAccountServ::get_account_detail_aggs(
            account_id,
            &IamAccountFilterReq {
                basic: RbumBasicFilterReq {
                    ignore_scope: true,
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                ..Default::default()
            },
            true,
            true,
            funs,
            ctx,
        )
        .await?;
        let account_certs = account_resp.certs.iter().map(|m| m.1.clone()).collect::<Vec<String>>();
        let account_app_ids: Vec<String> = account_resp.apps.iter().map(|a| a.app_id.clone()).collect();
        let search_url = funs.conf::<IamConfig>().spi.search_url.clone();
        let kv_url = funs.conf::<IamConfig>().spi.kv_url.clone();
        let spi_ctx = TardisContext {
            owner: funs.conf::<IamConfig>().spi.owner.clone(),
            ..ctx.clone()
        };
        let headers = Some(vec![(
            "Tardis-Context".to_string(),
            TardisFuns::crypto.base64.encode(&TardisFuns::json.obj_to_string(&spi_ctx).unwrap()),
        )]);
        #[cfg(feature = "spi_kv_features")]
        {
            //add kv
            funs.web_client()
                .put_obj_to_str(
                    &format!("{kv_url}/ci/item"),
                    &HashMap::from([("key", account_id), ("value", &account_resp.name)]),
                    headers.clone(),
                )
                .await
                .unwrap();
        }
        #[cfg(feature = "spi_search_features")]
        {
            let utc_now = Utc::now().to_string();
            let mut search_body = json!({
                "tag": funs.conf::<IamConfig>().spi.search_tag.clone(),
                "key": account_id.to_string(),
                "title": account_resp.name.clone(),
                "kind": funs.conf::<IamConfig>().spi.search_tag.clone(),
                "content": format!("{},{:?}", account_resp.name, account_certs,),
                "owner": funs.conf::<IamConfig>().spi.owner.clone(),
                "update_time": utc_now.clone(),
                "ext":{
                    "status": account_resp.disabled.to_string(),
                    "create_time": account_resp.create_time.to_string()
                },
                "visit_keys":{
                    "apps": account_app_ids,
                    "groups": account_resp.orgs
                },
            });
            if !is_modify {
                search_body.as_object_mut().unwrap().insert("create_time".to_string(), serde_json::Value::from(utc_now));
            }
            if !ctx.own_paths.is_empty() {
                search_body.as_object_mut().unwrap().insert("own_paths".to_string(), serde_json::Value::from(ctx.own_paths.clone()));
            }
            //add search
            funs.web_client().put_obj_to_str(&format!("{search_url}/ci/item"), &search_body, headers.clone()).await.unwrap();
        }
        Ok(())
    }
}
