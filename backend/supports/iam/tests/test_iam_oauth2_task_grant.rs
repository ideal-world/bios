use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use bios_basic::rbum::helper::rbum_event_helper;
use bios_basic::rbum::rbum_initializer::get_first_account_context;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;
use bios_basic::test::init_test_container;
use bios_iam::basic::domain::iam_oauth2_task_grant;
use bios_iam::basic::dto::iam_account_dto::IamAccountAddReq;
use bios_iam::basic::dto::iam_app_dto::IamAppAggAddReq;
use bios_iam::basic::dto::iam_cert_conf_dto::IamCertConfOAuth2AddOrModifyReq;
use bios_iam::basic::dto::iam_oauth2_task_grant_dto::{IamOAuth2TaskGrantCreateReq, IamOAuth2TaskGrantExchangeReq, IamOAuth2TaskGrantToken};
use bios_iam::basic::dto::iam_role_dto::{IamRoleAddReq, IamRoleAggAddReq};
use bios_iam::basic::dto::iam_tenant_dto::IamTenantAggAddReq;
use bios_iam::basic::serv::iam_account_serv::IamAccountServ;
use bios_iam::basic::serv::iam_app_serv::IamAppServ;
use bios_iam::basic::serv::iam_cert_oauth2_serv::{IamCertOAuth2Serv, IamCertOAuth2TokenInfo};
use bios_iam::basic::serv::iam_oauth2_task_grant_serv::IamOAuth2TaskGrantServ;
use bios_iam::basic::serv::iam_rel_serv::IamRelServ;
use bios_iam::basic::serv::iam_role_serv::IamRoleServ;
use bios_iam::basic::serv::iam_tenant_serv::IamTenantServ;
use bios_iam::{
    iam_config::IamBasicConfigApi,
    iam_constants,
    iam_enumeration::{IamAccountLockStateKind, IamCertOAuth2Supplier, IamRelKind},
};
use tardis::basic::{dto::TardisContext, field::TrimString, result::TardisResult};
use tardis::chrono::Utc;
use tardis::db::sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use tardis::serde_json::json;
use tardis::tokio;
use tardis::web::poem::{endpoint::make, http::StatusCode, listener::TcpAcceptor, Body, Request, Response, Route, Server};
use tardis::{TardisFuns, TardisFunsInst};

fn task_grant_req(task_id: &str, requires_external_oauth: bool, expected_grant_id: Option<&str>, takeover: bool) -> IamOAuth2TaskGrantCreateReq {
    IamOAuth2TaskGrantCreateReq {
        task_id: task_id.to_string(),
        requires_external_oauth,
        expected_grant_id: expected_grant_id.map(str::to_string),
        takeover,
    }
}

fn task_context(owner: &str, own_paths: &str) -> TardisContext {
    TardisContext {
        own_paths: own_paths.to_string(),
        owner: owner.to_string(),
        ..Default::default()
    }
}

async fn create_task_grant(
    req: IamOAuth2TaskGrantCreateReq,
    provider_grant_id: Option<String>,
    ctx: TardisContext,
) -> TardisResult<bios_iam::basic::dto::iam_oauth2_task_grant_dto::IamOAuth2TaskGrantRef> {
    let mut tx_funs = iam_constants::get_tardis_inst();
    tx_funs.begin().await?;
    match Box::pin(IamOAuth2TaskGrantServ::create(&req, provider_grant_id.as_deref(), &tx_funs, &ctx)).await {
        Ok(grant) => {
            tx_funs.commit().await?;
            ctx.execute_task().await?;
            Ok(grant)
        }
        Err(error) => {
            tx_funs.rollback().await?;
            Err(error)
        }
    }
}

async fn validate_task_grant_create(req: &IamOAuth2TaskGrantCreateReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    Box::pin(IamOAuth2TaskGrantServ::validate_create(req, funs, ctx)).await
}

async fn current_task_grant(
    task_id: &str,
    funs: &TardisFunsInst,
    ctx: &TardisContext,
) -> TardisResult<Option<bios_iam::basic::dto::iam_oauth2_task_grant_dto::IamOAuth2TaskGrantRef>> {
    Box::pin(IamOAuth2TaskGrantServ::current(task_id, funs, ctx)).await
}

async fn exchange_task_grant(req: &IamOAuth2TaskGrantExchangeReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<IamOAuth2TaskGrantToken> {
    Box::pin(IamOAuth2TaskGrantServ::exchange(req, funs, ctx)).await
}

async fn revoke_task_grant(grant_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    Box::pin(IamOAuth2TaskGrantServ::revoke(grant_id, funs, ctx)).await
}

struct TaskGrantFixture {
    system_ctx: TardisContext,
    tenant_ctx: TardisContext,
    user_ctx: TardisContext,
    scheduler_ctx: TardisContext,
    app_id: String,
    role_id: String,
}

#[tokio::test]
#[ignore = "requires Docker for isolated PostgreSQL, Redis and RabbitMQ"]
async fn task_grant_checks_persisted_authorization_cas_and_transactions() -> TardisResult<()> {
    Box::pin(task_grant_checks_persisted_authorization_cas_and_transactions_fixture()).await
}

async fn task_grant_checks_persisted_authorization_cas_and_transactions_fixture() -> TardisResult<()> {
    std::env::set_var("TARDIS_CSM.IAM.OAUTH2_TASK_EXECUTOR_ACCOUNT_ID", "scheduler");
    std::env::set_var("TARDIS_CSM.IAM.OAUTH2_PROVIDER_GRANT_KEY", "07".repeat(32));
    let _containers = Box::pin(init_test_container::init(None)).await?;
    let funs = iam_constants::get_tardis_inst();
    Box::pin(bios_iam::iam_initializer::init_db(iam_constants::get_tardis_inst())).await?;
    assert_eq!(funs.conf::<bios_iam::iam_config::IamConfig>().oauth2_task_executor_account_id, "scheduler");
    let fixture = Box::pin(setup_task_grant_fixture(&funs)).await?;
    let internal_replacement_id = Box::pin(verify_authorization_and_rollback(&funs, &fixture)).await?;
    Box::pin(verify_takeover_authorization(&funs, &fixture)).await?;
    Box::pin(verify_concurrent_first_create(&funs, &fixture)).await?;
    Box::pin(verify_provider_delivery(&funs, &fixture, &internal_replacement_id)).await?;
    Ok(())
}

async fn setup_task_grant_fixture(funs: &TardisFunsInst) -> TardisResult<TaskGrantFixture> {
    let system_ctx = Box::pin(get_first_account_context(iam_constants::RBUM_KIND_CODE_IAM_ACCOUNT, iam_constants::COMPONENT_CODE, funs)).await?.unwrap();
    let account_id = Box::pin(IamAccountServ::add_item(
        &mut TardisFuns::json.json_to_obj::<IamAccountAddReq>(json!({
            "name": "Scheduled starter", "scope_level": iam_constants::RBUM_SCOPE_LEVEL_GLOBAL,
            "status": bios_iam::iam_enumeration::IamAccountStatusKind::Active
        }))?,
        funs,
        &system_ctx,
    ))
    .await?;
    let (tenant_id, _, _) = Box::pin(IamTenantServ::add_tenant_agg(
        &IamTenantAggAddReq {
            name: TrimString::from("Task grant fixture tenant"),
            icon: None,
            contact_phone: None,
            note: None,
            account_self_reg: None,
            disabled: None,
            admin_username: TrimString::from("task-grant-admin"),
            admin_password: None,
            admin_phone: None,
            admin_mail: None,
            admin_name: TrimString::from("Task grant fixture admin"),
            audit_username: TrimString::from("task-grant-audit"),
            audit_password: None,
            audit_phone: None,
            audit_mail: None,
            audit_name: TrimString::from("Task grant fixture auditor"),
            cert_conf_by_oauth2: None,
            cert_conf_by_ldap: None,
        },
        funs,
        &system_ctx,
    ))
    .await?;
    let tenant_ctx = TardisContext {
        own_paths: tenant_id,
        owner: account_id.clone(),
        ..Default::default()
    };
    let app_id = Box::pin(IamAppServ::add_app_agg(
        &IamAppAggAddReq {
            app_name: TrimString::from("Scheduled workflow"),
            app_description: None,
            app_icon: None,
            app_sort: None,
            app_contact_phone: None,
            admin_ids: None,
            publish_system_ids: None,
            disabled: None,
            set_cate_id: None,
            kind: None,
            sync_apps_group: Some(false),
        },
        funs,
        &tenant_ctx,
    ))
    .await?;
    let user_ctx = TardisContext {
        own_paths: format!("{}/{app_id}", tenant_ctx.own_paths),
        owner: account_id,
        ..Default::default()
    };
    let tenant_role_id = Box::pin(IamRoleServ::tenant_add_app_role_agg(
        &mut IamRoleAggAddReq {
            role: IamRoleAddReq {
                id: None,
                code: None,
                name: TrimString::from("Workflow user"),
                icon: None,
                sort: None,
                kind: Some(bios_iam::iam_enumeration::IamRoleKind::App),
                scope_level: Some(iam_constants::RBUM_SCOPE_LEVEL_APP),
                in_embed: None,
                in_base: None,
                extend_role_id: None,
                disabled: None,
                deletable: None,
            },
            res_ids: None,
        },
        funs,
        &tenant_ctx,
    ))
    .await?;
    let role_id = Box::pin(IamRoleServ::get_embed_sub_role_id(&tenant_role_id, funs, &user_ctx)).await?;
    IamAppServ::add_rel_account(&app_id, &user_ctx.owner, false, funs, &user_ctx).await?;
    IamRelServ::add_simple_rel(&IamRelKind::IamAccountRole, &user_ctx.owner, &role_id, None, None, true, false, funs, &user_ctx).await?;
    let scheduler_ctx = TardisContext {
        owner: "scheduler".into(),
        own_paths: user_ctx.own_paths.clone(),
        ..Default::default()
    };
    Ok(TaskGrantFixture {
        system_ctx,
        tenant_ctx,
        user_ctx,
        scheduler_ctx,
        app_id,
        role_id,
    })
}

async fn verify_authorization_and_rollback(funs: &TardisFunsInst, fixture: &TaskGrantFixture) -> TardisResult<String> {
    let user_ctx = &fixture.user_ctx;
    let scheduler_ctx = &fixture.scheduler_ctx;
    let app_id = &fixture.app_id;
    let role_id = &fixture.role_id;
    let internal = create_task_grant(task_grant_req("binding/internal", false, None, false), None, (*user_ctx).clone()).await?;
    let initial_events = rbum_event_helper::get_notify_event_with_ctx(user_ctx).await?.unwrap();
    assert!(initial_events.iter().any(|event| event.table_name == "iam_oauth2_task_grant" && event.operate == "c" && event.record_id == internal.grant_id));
    let internal_req = IamOAuth2TaskGrantExchangeReq {
        grant_id: internal.grant_id,
        task_id: "binding/internal".into(),
    };
    assert!(exchange_task_grant(&internal_req, funs, scheduler_ctx).await?.access_token.is_none());

    let rollback_ctx = task_context(&user_ctx.owner, &user_ctx.own_paths);
    let mut rollback_funs = iam_constants::get_tardis_inst();
    rollback_funs.begin().await?;
    let rolled_back = Box::pin(IamOAuth2TaskGrantServ::create(
        &task_grant_req("binding/internal", false, Some(&internal_req.grant_id), false),
        None,
        &rollback_funs,
        &rollback_ctx,
    ))
    .await?;
    rollback_funs.rollback().await?;
    assert_ne!(rolled_back.grant_id, internal_req.grant_id);
    let current_internal = current_task_grant("binding/internal", funs, user_ctx).await?.unwrap();
    assert_eq!(current_internal.grant_id, internal_req.grant_id);
    assert!(exchange_task_grant(&internal_req, funs, scheduler_ctx).await?.access_token.is_none());

    let stale_req = task_grant_req("binding/internal", false, Some("stale-grant-id"), false);
    assert_eq!(
        validate_task_grant_create(&stale_req, funs, user_ctx).await.unwrap_err().code,
        "409-iam-oauth-task-grant-conflict"
    );

    funs.db()
        .execute_one(
            "UPDATE rbum_rel SET disabled = $1 WHERE tag = $2 AND from_rbum_id = $3 AND to_rbum_item_id = $4",
            vec![
                true.into(),
                IamRelKind::IamAccountApp.to_string().into(),
                user_ctx.owner.clone().into(),
                app_id.clone().into(),
            ],
        )
        .await?;
    let disabled_app_relation_rejected = exchange_task_grant(&internal_req, funs, scheduler_ctx).await.is_err();
    assert!(validate_task_grant_create(&task_grant_req("binding/disabled-app", false, None, false), funs, user_ctx).await.is_err());
    funs.db()
        .execute_one(
            "UPDATE rbum_rel SET disabled = $1 WHERE tag = $2 AND from_rbum_id = $3 AND to_rbum_item_id = $4",
            vec![
                false.into(),
                IamRelKind::IamAccountApp.to_string().into(),
                user_ctx.owner.clone().into(),
                app_id.clone().into(),
            ],
        )
        .await?;
    assert!(disabled_app_relation_rejected, "a disabled app membership relation must invalidate task grants");

    funs.db()
        .execute_one(
            "UPDATE rbum_rel SET disabled = $1 WHERE tag = $2 AND from_rbum_id = $3 AND to_rbum_item_id = $4",
            vec![
                true.into(),
                IamRelKind::IamAccountRole.to_string().into(),
                user_ctx.owner.clone().into(),
                role_id.clone().into(),
            ],
        )
        .await?;
    assert!(
        exchange_task_grant(&internal_req, funs, scheduler_ctx).await.is_err(),
        "a disabled account-role relation must invalidate task grants"
    );
    assert!(validate_task_grant_create(&task_grant_req("binding/disabled-role", false, None, false), funs, user_ctx).await.is_err());
    funs.db()
        .execute_one(
            "UPDATE rbum_rel SET disabled = $1 WHERE tag = $2 AND from_rbum_id = $3 AND to_rbum_item_id = $4",
            vec![
                false.into(),
                IamRelKind::IamAccountRole.to_string().into(),
                user_ctx.owner.clone().into(),
                role_id.clone().into(),
            ],
        )
        .await?;

    funs.db()
        .execute_one(
            "UPDATE iam_account SET lock_status = $1 WHERE id = $2",
            vec![IamAccountLockStateKind::ManualLocked.to_int().into(), user_ctx.owner.clone().into()],
        )
        .await?;
    assert!(validate_task_grant_create(&task_grant_req("binding/locked-account", false, None, false), funs, user_ctx).await.is_err());
    assert!(
        exchange_task_grant(&internal_req, funs, scheduler_ctx).await.is_err(),
        "a locked account must invalidate existing grants"
    );
    funs.db()
        .execute_one(
            "UPDATE iam_account SET lock_status = $1 WHERE id = $2",
            vec![IamAccountLockStateKind::Unlocked.to_int().into(), user_ctx.owner.clone().into()],
        )
        .await?;

    let mut revoke_funs = iam_constants::get_tardis_inst();
    revoke_funs.begin().await?;
    revoke_task_grant(&internal_req.grant_id, &revoke_funs, user_ctx).await?;
    revoke_funs.commit().await?;
    user_ctx.execute_task().await?;
    let revoke_events = rbum_event_helper::get_notify_event_with_ctx(user_ctx).await?.unwrap();
    assert!(revoke_events.iter().any(|event| event.table_name == "iam_oauth2_task_grant" && event.operate == "u" && event.record_id == internal_req.grant_id));
    let revoked_before_replace = iam_oauth2_task_grant::Entity::find_by_id(&internal_req.grant_id).one(funs.db().raw_conn()).await?.unwrap();
    assert!(revoked_before_replace.revoked);
    assert_eq!(revoked_before_replace.revoked_by.as_deref(), Some(user_ctx.owner.as_str()));
    assert!(revoked_before_replace.revoked_time.is_some());
    assert_eq!(current_task_grant("binding/internal", funs, user_ctx).await?.unwrap().grant_id, internal_req.grant_id);
    assert!(exchange_task_grant(&internal_req, funs, scheduler_ctx).await.is_err());

    let internal_replacement = create_task_grant(task_grant_req("binding/internal", false, Some(&internal_req.grant_id), false), None, (*user_ctx).clone()).await?;
    let replacement_events = rbum_event_helper::get_notify_event_with_ctx(user_ctx).await?.unwrap();
    assert!(replacement_events.iter().any(|event| event.table_name == "iam_oauth2_task_grant" && event.operate == "c" && event.record_id == internal_replacement.grant_id));
    assert!(replacement_events.iter().any(|event| event.table_name == "iam_oauth2_task_grant" && event.operate == "u" && event.record_id == internal_req.grant_id));
    let revoked_after_replace = iam_oauth2_task_grant::Entity::find_by_id(&internal_req.grant_id).one(funs.db().raw_conn()).await?.unwrap();
    assert_eq!(revoked_after_replace.revoked_by, revoked_before_replace.revoked_by);
    assert_eq!(revoked_after_replace.revoked_time, revoked_before_replace.revoked_time);
    assert_eq!(revoked_after_replace.replaced_by_grant_id.as_deref(), Some(internal_replacement.grant_id.as_str()));
    assert!(exchange_task_grant(&internal_req, funs, scheduler_ctx).await.is_err());
    Ok(internal_replacement.grant_id)
}

async fn verify_takeover_authorization(funs: &TardisFunsInst, fixture: &TaskGrantFixture) -> TardisResult<()> {
    let user_ctx = &fixture.user_ctx;
    let scheduler_ctx = &fixture.scheduler_ctx;
    let system_ctx = &fixture.system_ctx;
    let app_id = &fixture.app_id;
    let role_id = &fixture.role_id;
    let takeover_grant = create_task_grant(task_grant_req("binding/takeover", false, None, false), None, (*user_ctx).clone()).await?;
    let other_account_id = IamAccountServ::add_item(
        &mut TardisFuns::json.json_to_obj::<IamAccountAddReq>(json!({
            "name": "Scheduled second owner", "scope_level": iam_constants::RBUM_SCOPE_LEVEL_GLOBAL,
            "status": bios_iam::iam_enumeration::IamAccountStatusKind::Active
        }))?,
        &funs,
        &system_ctx,
    )
    .await?;
    let mut other_user_ctx = task_context(&other_account_id, &user_ctx.own_paths);
    other_user_ctx.roles = vec![funs.iam_basic_role_app_admin_id()];
    IamAppServ::add_rel_account(&app_id, &other_account_id, false, &funs, &other_user_ctx).await?;
    IamRelServ::add_simple_rel(&IamRelKind::IamAccountRole, &other_account_id, &role_id, None, None, true, false, &funs, &other_user_ctx).await?;

    assert_eq!(
        current_task_grant("binding/takeover", funs, &other_user_ctx).await.unwrap_err().code,
        "403-iam-oauth-task-current-forbidden"
    );
    assert_eq!(
        validate_task_grant_create(&task_grant_req("binding/takeover", false, Some(&takeover_grant.grant_id), false), funs, &other_user_ctx).await.unwrap_err().code,
        "403-iam-oauth-task-takeover-forbidden"
    );
    assert_eq!(
        validate_task_grant_create(&task_grant_req("binding/takeover", false, Some(&takeover_grant.grant_id), true), funs, &other_user_ctx).await.unwrap_err().code,
        "403-iam-oauth-task-takeover-forbidden",
        "ctx.roles must not grant app-admin authority"
    );
    assert!(exchange_task_grant(
        &IamOAuth2TaskGrantExchangeReq {
            grant_id: takeover_grant.grant_id.clone(),
            task_id: "binding/takeover".into(),
        },
        funs,
        scheduler_ctx,
    )
    .await?
    .access_token
    .is_none());

    let admin_role_id = IamRoleServ::get_embed_sub_role_id(&funs.iam_basic_role_app_admin_id(), &funs, &other_user_ctx).await?;
    IamRelServ::add_simple_rel(
        &IamRelKind::IamAccountRole,
        &other_account_id,
        &admin_role_id,
        None,
        None,
        true,
        false,
        &funs,
        &other_user_ctx,
    )
    .await?;
    funs.db()
        .execute_one(
            "UPDATE rbum_rel SET disabled = $1 WHERE tag = $2 AND from_rbum_id = $3 AND to_rbum_item_id = $4",
            vec![
                true.into(),
                IamRelKind::IamAccountRole.to_string().into(),
                other_account_id.clone().into(),
                admin_role_id.clone().into(),
            ],
        )
        .await?;
    assert_eq!(
        validate_task_grant_create(&task_grant_req("binding/takeover", false, Some(&takeover_grant.grant_id), true), funs, &other_user_ctx).await.unwrap_err().code,
        "403-iam-oauth-task-takeover-forbidden",
        "a disabled persisted app-admin relation must not authorize takeover"
    );
    assert!(
        exchange_task_grant(
            &IamOAuth2TaskGrantExchangeReq {
                grant_id: takeover_grant.grant_id.clone(),
                task_id: "binding/takeover".into(),
            },
            funs,
            scheduler_ctx,
        )
        .await?
        .access_token
        .is_none(),
        "failed takeover must leave the prior grant usable"
    );
    funs.db()
        .execute_one(
            "UPDATE rbum_rel SET disabled = $1 WHERE tag = $2 AND from_rbum_id = $3 AND to_rbum_item_id = $4",
            vec![
                false.into(),
                IamRelKind::IamAccountRole.to_string().into(),
                other_account_id.clone().into(),
                admin_role_id.clone().into(),
            ],
        )
        .await?;
    assert_eq!(
        current_task_grant("binding/takeover", funs, &other_user_ctx).await?.unwrap().grant_id,
        takeover_grant.grant_id
    );
    let takeover_replacement = create_task_grant(
        task_grant_req("binding/takeover", false, Some(&takeover_grant.grant_id), true),
        None,
        other_user_ctx.clone(),
    )
    .await?;
    assert_eq!(
        current_task_grant("binding/takeover", funs, &other_user_ctx).await?.unwrap().grant_id,
        takeover_replacement.grant_id
    );
    assert_eq!(
        current_task_grant("binding/takeover", funs, user_ctx).await.unwrap_err().code,
        "403-iam-oauth-task-current-forbidden"
    );
    Ok(())
}

async fn verify_concurrent_first_create(funs: &TardisFunsInst, fixture: &TaskGrantFixture) -> TardisResult<()> {
    let user_ctx = &fixture.user_ctx;
    let race_barrier = Arc::new(tokio::sync::Barrier::new(2));
    let race_barrier_one = race_barrier.clone();
    let race_barrier_two = race_barrier.clone();
    let race_ctx_one = task_context(&user_ctx.owner, &user_ctx.own_paths);
    let race_ctx_two = task_context(&user_ctx.owner, &user_ctx.own_paths);
    let race_one = async move {
        race_barrier_one.wait().await;
        create_task_grant(task_grant_req("binding/concurrent", false, None, false), None, race_ctx_one).await
    };
    let race_two = async move {
        race_barrier_two.wait().await;
        create_task_grant(task_grant_req("binding/concurrent", false, None, false), None, race_ctx_two).await
    };
    let (race_result_one, race_result_two) = tokio::join!(race_one, race_two);
    assert_eq!(race_result_one.is_ok() as usize + race_result_two.is_ok() as usize, 1);
    for result in [race_result_one, race_result_two] {
        if let Err(error) = result {
            assert_eq!(error.code, "409-iam-oauth-task-grant-conflict");
        }
    }
    let concurrent_slots = iam_oauth2_task_grant::Entity::find()
        .filter(iam_oauth2_task_grant::Column::CurrentOwnPaths.eq(&user_ctx.own_paths))
        .filter(iam_oauth2_task_grant::Column::CurrentTaskId.eq("binding/concurrent"))
        .all(funs.db().raw_conn())
        .await?;
    assert_eq!(concurrent_slots.len(), 1);
    Ok(())
}

async fn verify_provider_delivery(funs: &TardisFunsInst, fixture: &TaskGrantFixture, internal_replacement_id: &str) -> TardisResult<()> {
    let system_ctx = &fixture.system_ctx;
    let tenant_ctx = &fixture.tenant_ctx;
    let user_ctx = &fixture.user_ctx;
    let scheduler_ctx = &fixture.scheduler_ctx;
    let app_id = &fixture.app_id;
    let revoked = Arc::new(AtomicBool::new(false));
    let delay_userinfo = Arc::new(AtomicBool::new(false));
    let token_endpoint = make(move |request: Request| async move {
        let _body = request.into_body().into_string().await.unwrap();
        Response::builder().status(StatusCode::OK).content_type("application/json").body(Body::from_string(json!({
                "code": "200", "msg": "", "data": {"access_token": "rotated-access", "token_type": "Bearer", "expires_in": 3600, "refresh_token": "rotated-refresh", "scope": "iam.app.read"}
            }).to_string()))
    });
    let revoked_for_provider = revoked.clone();
    let delay_userinfo_for_provider = delay_userinfo.clone();
    let userinfo_endpoint = make(move |request: Request| {
        let revoked = revoked_for_provider.clone();
        let delay_userinfo = delay_userinfo_for_provider.clone();
        async move {
            assert_eq!(request.headers().get("Bios-Token").unwrap().to_str().unwrap(), "rotated-access");
            if delay_userinfo.load(Ordering::SeqCst) {
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
            let status = if revoked.load(Ordering::SeqCst) { StatusCode::UNAUTHORIZED } else { StatusCode::OK };
            Response::builder().status(status).content_type("application/json").body(Body::from_string(json!({
                "code": status.as_u16().to_string(), "msg": "", "data": {"provider": "bios", "sub": "external-user", "tenant_id": "external-tenant", "name": "External user", "disabled": false}
            }).to_string()))
        }
    });
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
    let address = listener.local_addr().unwrap();
    let acceptor = TcpAcceptor::from_tokio(listener).unwrap();
    let (stop_tx, stop_rx) = tokio::sync::oneshot::channel();
    let provider_task = tokio::spawn(async move {
        Server::new_with_acceptor(acceptor)
            .run_with_graceful_shutdown(
                Route::new().at("/cp/oauth2/token", token_endpoint).at("/cp/oauth2/userinfo", userinfo_endpoint),
                async move {
                    let _ = stop_rx.await;
                },
                Some(Duration::from_secs(1)),
            )
            .await
            .unwrap();
    });
    let cert_conf_id = IamCertOAuth2Serv::add_cert_conf(
        IamCertOAuth2Supplier::BiosIam,
        &IamCertConfOAuth2AddOrModifyReq {
            supplier: TrimString("BiosIam".to_string()),
            ak: TrimString("mock-client".to_string()),
            sk: TrimString("mock-secret".to_string()),
            base_url: Some(format!("http://{address}")),
        },
        "",
        &funs,
        &system_ctx,
    )
    .await?;
    IamCertOAuth2Serv::bind_cert_account(IamCertOAuth2Supplier::BiosIam, "mock-code", &tenant_ctx.own_paths, &user_ctx.owner, &funs, &system_ctx).await?;
    let cache_key = format!(
        "{}BiosIam:{}",
        funs.conf::<bios_iam::iam_config::IamConfig>().cache_key_oauth2_provider_token_,
        user_ctx.owner
    );
    let token = IamCertOAuth2TokenInfo {
        open_id: "external-user".into(),
        access_token: "rotated-access".into(),
        refresh_token: Some("initial-refresh".into()),
        token_expires_ms: Some(3600000),
        expires_at_ms: Some(Utc::now().timestamp_millis() + 3600000),
        scope: Some("iam.app.read".into()),
        provider_cert_conf_id: Some(cert_conf_id),
        union_id: None,
    };
    funs.cache().set_ex(&cache_key, &TardisFuns::json.obj_to_string(&token)?, 300).await?;
    let provider_grant_id = IamCertOAuth2Serv::ensure_provider_grant(&user_ctx.owner, &user_ctx.own_paths, &funs).await?;
    let grant = create_task_grant(task_grant_req("binding/external", true, None, false), Some(provider_grant_id), (*user_ctx).clone()).await?;
    let req = IamOAuth2TaskGrantExchangeReq {
        grant_id: grant.grant_id.clone(),
        task_id: "binding/external".into(),
    };
    delay_userinfo.store(true, Ordering::SeqCst);
    let (exchange_one, exchange_two) = tokio::join!(
        Box::pin(exchange_task_grant(&req, funs, scheduler_ctx)),
        Box::pin(exchange_task_grant(&req, funs, scheduler_ctx)),
    );
    delay_userinfo.store(false, Ordering::SeqCst);
    assert_eq!(exchange_one?.access_token.as_deref(), Some("rotated-access"));
    assert_eq!(exchange_two?.access_token.as_deref(), Some("rotated-access"));
    let wrong_user = TardisContext {
        owner: user_ctx.owner.clone(),
        own_paths: user_ctx.own_paths.clone(),
        ..Default::default()
    };
    assert!(exchange_task_grant(&req, funs, &wrong_user).await.is_err());
    assert!(exchange_task_grant(
        &IamOAuth2TaskGrantExchangeReq {
            grant_id: grant.grant_id.clone(),
            task_id: "another-task".into()
        },
        funs,
        scheduler_ctx
    )
    .await
    .is_err());

    revoked.store(true, Ordering::SeqCst);
    assert_eq!(
        exchange_task_grant(&req, funs, scheduler_ctx).await.unwrap_err().code,
        "409-iam-oauth-task-authorization-required"
    );
    revoked.store(false, Ordering::SeqCst);
    let mut external_revoke_funs = iam_constants::get_tardis_inst();
    external_revoke_funs.begin().await?;
    revoke_task_grant(&grant.grant_id, &external_revoke_funs, user_ctx).await?;
    external_revoke_funs.commit().await?;
    user_ctx.execute_task().await?;
    assert!(exchange_task_grant(&req, funs, scheduler_ctx).await.is_err());
    IamRelServ::delete_simple_rel(&IamRelKind::IamAccountApp, &user_ctx.owner, &app_id, &funs, &user_ctx).await?;
    assert!(exchange_task_grant(
        &IamOAuth2TaskGrantExchangeReq {
            grant_id: internal_replacement_id.to_string(),
            task_id: "binding/internal".into(),
        },
        funs,
        scheduler_ctx,
    )
    .await
    .is_err());
    let _ = stop_tx.send(());
    provider_task.await.unwrap();
    Ok(())
}
