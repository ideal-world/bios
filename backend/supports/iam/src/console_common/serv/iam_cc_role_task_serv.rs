use crate::{
    basic::{
        dto::iam_filer_dto::{IamAccountFilterReq, IamAppFilterReq, IamRoleFilterReq, IamTenantFilterReq},
        serv::{iam_account_serv::IamAccountServ, iam_app_serv::IamAppServ, iam_rel_serv::IamRelServ, iam_role_serv::IamRoleServ, iam_tenant_serv::IamTenantServ},
    },
    iam_config::IamConfig,
    iam_constants,
    iam_enumeration::{IamRelKind, IamRoleKind},
    iam_initializer::{default_iam_send_avatar, ws_iam_send_client},
};
use bios_basic::{
    process::task_processor::TaskProcessor,
    rbum::{dto::rbum_filer_dto::RbumBasicFilterReq, serv::rbum_item_serv::RbumItemCrudOperation},
};

use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    log::info,
    TardisFunsInst,
};

pub struct IamCcRoleTaskServ;

impl IamCcRoleTaskServ {
    pub async fn execute_role_task(funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Option<String>> {
        let task_ctx = ctx.clone();
        TaskProcessor::execute_task_with_ctx(
            &funs.conf::<IamConfig>().cache_key_async_task_status,
            move |_task_id| async move {
                let mut funs = iam_constants::get_tardis_inst();
                funs.begin().await?;
                let base_tenant_role_ids = IamRoleServ::find_id_items(
                    &IamRoleFilterReq {
                        basic: RbumBasicFilterReq {
                            own_paths: Some("".to_string()),
                            with_sub_own_paths: true,
                            ..Default::default()
                        },
                        kind: Some(IamRoleKind::Tenant),
                        in_base: Some(true),
                        in_embed: Some(true),
                        ..Default::default()
                    },
                    None,
                    None,
                    &funs,
                    &task_ctx,
                )
                .await?;
                let base_app_role_ids = IamRoleServ::find_id_items(
                    &IamRoleFilterReq {
                        basic: RbumBasicFilterReq {
                            own_paths: Some("".to_string()),
                            with_sub_own_paths: true,
                            ..Default::default()
                        },
                        kind: Some(IamRoleKind::App),
                        in_base: Some(true),
                        in_embed: Some(true),
                        ..Default::default()
                    },
                    None,
                    None,
                    &funs,
                    &task_ctx,
                )
                .await?;
                let tenants = IamTenantServ::find_items(
                    &IamTenantFilterReq {
                        basic: RbumBasicFilterReq {
                            own_paths: Some("".to_string()),
                            with_sub_own_paths: true,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    None,
                    None,
                    &funs,
                    &task_ctx,
                )
                .await?;
                for tenant in tenants {
                    let tenant_ctx = TardisContext {
                        own_paths: tenant.own_paths.clone(),
                        ..task_ctx.clone()
                    };
                    if IamRoleServ::count_items(
                        &IamRoleFilterReq {
                            basic: RbumBasicFilterReq {
                                with_sub_own_paths: true,
                                ..Default::default()
                            },
                            kind: Some(IamRoleKind::Tenant),
                            in_base: Some(false),
                            in_embed: Some(true),
                            ..Default::default()
                        },
                        &funs,
                        &tenant_ctx,
                    )
                    .await?
                        > 0
                    {
                        continue;
                    }
                    info!("execute_role_task: tenant_id: {}, tenant_name: {}", tenant.id, tenant.name);
                    IamRoleServ::copy_role_agg(&tenant.id, None, &IamRoleKind::Tenant, &funs, &tenant_ctx).await?;
                    for base_tenant_role_id in &base_tenant_role_ids {
                        let rel_account_roles = IamRelServ::find_to_simple_rels(&IamRelKind::IamAccountRole, &base_tenant_role_id, None, None, &funs, &tenant_ctx).await?;
                        for rel_account_role in rel_account_roles {
                            if IamAccountServ::count_items(
                                &IamAccountFilterReq {
                                    basic: RbumBasicFilterReq {
                                        with_sub_own_paths: true,
                                        ids: Some(vec![rel_account_role.rel_id.clone()]),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                },
                                &funs,
                                &tenant_ctx,
                            )
                            .await?
                                > 0
                            {
                                info!("execute_role_task: base_tenant_role_id: {}, rel_account_role: {:?}", base_tenant_role_id, rel_account_role);
                                let _ = IamRoleServ::add_rel_account(base_tenant_role_id, &rel_account_role.rel_id, None, &funs, &tenant_ctx).await;
                                let _ = IamRelServ::delete_simple_rel(&IamRelKind::IamAccountRole, &rel_account_role.rel_id, base_tenant_role_id, &funs, &tenant_ctx).await;
                            }
                        }
                    }
                    // tenant_ctx.execute_task().await?;
                }
                let apps = IamAppServ::find_items(
                    &IamAppFilterReq {
                        basic: RbumBasicFilterReq {
                            own_paths: Some("".to_string()),
                            with_sub_own_paths: true,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    None,
                    None,
                    &funs,
                    &task_ctx,
                )
                .await?;
                for app in apps {
                    let app_ctx = TardisContext {
                        own_paths: app.own_paths.clone(),
                        ..task_ctx.clone()
                    };
                    if IamRoleServ::count_items(
                        &IamRoleFilterReq {
                            basic: RbumBasicFilterReq {
                                with_sub_own_paths: true,
                                ..Default::default()
                            },
                            kind: Some(IamRoleKind::App),
                            in_base: Some(false),
                            in_embed: Some(true),
                            ..Default::default()
                        },
                        &funs,
                        &app_ctx,
                    )
                    .await?
                        > 0
                    {
                        continue;
                    }
                    info!("execute_role_task: app_id: {}, app_name: {}", app.id, app.name);
                    IamRoleServ::copy_role_agg(&app.id, None, &IamRoleKind::App, &funs, &app_ctx).await?;
                    for base_app_role_id in &base_app_role_ids {
                        let rel_account_roles = IamRelServ::find_to_simple_rels(&IamRelKind::IamAccountRole, base_app_role_id, None, None, &funs, &app_ctx).await?;
                        for rel_account_role in rel_account_roles {
                            if IamAccountServ::count_items(
                                &IamAccountFilterReq {
                                    basic: RbumBasicFilterReq {
                                        with_sub_own_paths: true,
                                        ids: Some(vec![rel_account_role.rel_id.clone()]),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                },
                                &funs,
                                &app_ctx,
                            )
                            .await?
                                > 0
                            {
                                info!("execute_role_task: base_app_role_id: {}, rel_account_role: {:?}", base_app_role_id, rel_account_role);
                                let _ = IamRoleServ::add_rel_account(base_app_role_id, &rel_account_role.rel_id, None, &funs, &app_ctx).await;
                                let _ = IamRelServ::delete_simple_rel(&IamRelKind::IamAccountRole, &rel_account_role.rel_id, base_app_role_id, &funs, &app_ctx).await;
                            }
                        }
                    }
                    // app_ctx.execute_task().await?;
                }
                funs.commit().await?;
                task_ctx.execute_task().await?;
                Ok(())
            },
            &funs.cache(),
            ws_iam_send_client().await.clone(),
            default_iam_send_avatar().await.clone(),
            Some(vec![format!("account/{}", ctx.owner)]),
            ctx,
        )
        .await?;
        Ok(None)
    }
}
