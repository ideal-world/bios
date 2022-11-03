use bios_basic::rbum::rbum_enumeration::RbumRelFromKind;
use itertools::Itertools;
use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::TardisFunsInst;

use crate::basic::dto::iam_account_dto::IamAccountAttrResp;
use crate::iam_enumeration::IamRelKind;
use bios_basic::rbum::dto::rbum_filer_dto::{RbumBasicFilterReq, RbumCertFilterReq, RbumItemRelFilterReq};
use bios_basic::rbum::dto::rbum_rel_dto::RbumRelBoneResp;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;

use crate::basic::dto::iam_filer_dto::{IamAccountFilterReq, IamAppFilterReq, IamTenantFilterReq};
use crate::basic::serv::iam_account_serv::IamAccountServ;
use crate::basic::serv::iam_app_serv::IamAppServ;
use crate::basic::serv::iam_attr_serv::IamAttrServ;
use crate::basic::serv::iam_cert_serv::IamCertServ;
use crate::basic::serv::iam_role_serv::IamRoleServ;
use crate::basic::serv::iam_set_serv::IamSetServ;
use crate::basic::serv::iam_tenant_serv::IamTenantServ;
use crate::console_passport::dto::iam_cp_account_dto::{IamCpAccountAppInfoResp, IamCpAccountInfoResp};

pub struct IamCpAccountServ;

impl IamCpAccountServ {
    // TODO To optimize
    pub async fn get_current_account_info(use_sys_cert: bool, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<IamCpAccountInfoResp> {
        let account = IamAccountServ::get_item(ctx.owner.as_str(), &IamAccountFilterReq::default(), funs, ctx).await?;
        let raw_roles = IamAccountServ::find_simple_rel_roles(&account.id, true, Some(true), None, funs, ctx).await?;
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
        let mut apps: Vec<IamCpAccountAppInfoResp> = vec![];
        for app in enabled_apps {
            apps.push(IamCpAccountAppInfoResp {
                app_id: app.id,
                app_name: app.name,
                roles: roles.iter().filter(|r| r.rel_own_paths == app.own_paths).map(|r| (r.rel_id.to_string(), r.rel_name.to_string())).collect(),
            });
        }

        let tenant_name = if ctx.own_paths.is_empty() {
            None
        } else {
            Some(IamTenantServ::peek_item(&IamTenantServ::get_id_by_ctx(ctx, funs)?, &IamTenantFilterReq::default(), funs, ctx).await?.name)
        };

        let org = if ctx.own_paths.is_empty() {
            vec![]
        } else {
            // Get an organizational tree that is open to tenants and applications
            let set_id = IamSetServ::get_set_id_by_code(&IamSetServ::get_default_org_code_by_tenant(funs, ctx)?, true, funs, ctx).await?;
            // Get all tree nodes using tenant contexts
            let tenant_ctx = IamCertServ::use_sys_or_tenant_ctx_unsafe(ctx.clone())?;
            IamSetServ::find_set_paths(&account.id, &set_id, funs, &tenant_ctx).await?.into_iter().map(|r| r.into_iter().map(|rr| rr.name).join("/")).collect()
        };
        let account_attrs = IamAttrServ::find_account_attrs(funs, ctx).await?;
        let account_attr_values = IamAttrServ::find_account_attr_values(&account.id, funs, ctx).await?;
        let account_info = IamCpAccountInfoResp {
            account_id: account.id.clone(),
            account_name: account.name.clone(),
            account_icon: account.icon.clone(),
            tenant_id: Some(IamTenantServ::get_id_by_ctx(ctx, funs)?),
            tenant_name,
            roles: roles.iter().filter(|r| r.rel_own_paths == ctx.own_paths).map(|r| (r.rel_id.to_string(), r.rel_name.to_string())).collect(),
            org,
            apps,
            certs: IamCertServ::find_certs(
                &RbumCertFilterReq {
                    basic: RbumBasicFilterReq {
                        own_paths: if use_sys_cert {
                            Some("".to_string())
                        } else {
                            Some(IamTenantServ::get_id_by_ctx(ctx, funs)?)
                        },
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
            exts: account_attrs
                .into_iter()
                .map(|r| IamAccountAttrResp {
                    name: r.name.clone(),
                    label: r.label,
                    value: account_attr_values.get(&r.name).unwrap_or(&("".to_string())).to_string(),
                })
                .collect(),
            scope_level: account.scope_level,
        };
        Ok(account_info)
    }
}
