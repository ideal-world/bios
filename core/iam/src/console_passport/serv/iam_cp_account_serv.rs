use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::TardisFunsInst;

use bios_basic::rbum::dto::rbum_filer_dto::RbumBasicFilterReq;
use bios_basic::rbum::dto::rbum_rel_dto::RbumRelBoneResp;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;

use crate::basic::dto::iam_filer_dto::{IamAccountFilterReq, IamAppFilterReq, IamTenantFilterReq};
use crate::basic::serv::iam_account_serv::IamAccountServ;
use crate::basic::serv::iam_app_serv::IamAppServ;
use crate::basic::serv::iam_cert_serv::IamCertServ;
use crate::basic::serv::iam_role_serv::IamRoleServ;
use crate::basic::serv::iam_set_serv::IamSetServ;
use crate::basic::serv::iam_tenant_serv::IamTenantServ;
use crate::console_passport::dto::iam_cp_account_dto::{IamCpAccountAppInfoResp, IamCpAccountInfoResp};

pub struct IamCpAccountServ;

impl<'a> IamCpAccountServ {
    pub async fn get_current_account_info(funs: &TardisFunsInst<'a>, ctx: &TardisContext) -> TardisResult<IamCpAccountInfoResp> {
        let account = IamAccountServ::get_item(ctx.owner.as_str(), &IamAccountFilterReq::default(), funs, ctx).await?;
        let raw_roles = IamAccountServ::find_simple_rel_roles(&account.id, true, Some(true), None, funs, ctx).await?;
        let mut roles: Vec<RbumRelBoneResp> = vec![];
        for role in raw_roles {
            if !IamRoleServ::is_disabled(&role.rel_id, funs).await? {
                roles.push(role)
            }
        }
        let apps = if !account.own_paths.is_empty() {
            let enabled_apps = IamAppServ::find_items(
                &IamAppFilterReq {
                    basic: RbumBasicFilterReq {
                        ignore_scope: false,
                        rel_ctx_owner: false,
                        with_sub_own_paths: true,
                        enabled: Some(true),
                        ..Default::default()
                    },
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
            apps
        } else {
            vec![]
        };

        let tenant_name = if account.own_paths.is_empty() {
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
            IamSetServ::find_set_paths(&account.id, &set_id, funs, &tenant_ctx).await?
        };
        let account_info = IamCpAccountInfoResp {
            account_id: account.id.clone(),
            account_name: account.name.clone(),
            tenant_name,
            roles: roles.iter().filter(|r| r.rel_own_paths == ctx.own_paths).map(|r| (r.rel_id.to_string(), r.rel_name.to_string())).collect(),
            org,
            apps,
        };
        Ok(account_info)
    }
}
