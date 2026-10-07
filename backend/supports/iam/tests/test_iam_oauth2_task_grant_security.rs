use std::sync::Arc;

use bios_basic::rbum::rbum_initializer::get_first_account_context;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;
use bios_basic::test::init_test_container;
use bios_iam::basic::domain::iam_oauth2_task_grant;
use bios_iam::basic::dto::iam_account_dto::IamAccountAddReq;
use bios_iam::basic::dto::iam_app_dto::IamAppAggAddReq;
use bios_iam::basic::dto::iam_oauth2_task_grant_dto::{IamOAuth2TaskGrantCreateReq, IamOAuth2TaskGrantRef};
use bios_iam::basic::dto::iam_tenant_dto::IamTenantAggAddReq;
use bios_iam::basic::serv::iam_account_serv::IamAccountServ;
use bios_iam::basic::serv::iam_app_serv::IamAppServ;
use bios_iam::basic::serv::iam_oauth2_task_grant_serv::IamOAuth2TaskGrantServ;
use bios_iam::basic::serv::iam_tenant_serv::IamTenantServ;
use bios_iam::iam_constants;
use tardis::basic::{dto::TardisContext, field::TrimString, result::TardisResult};
use tardis::db::sea_orm::{EntityTrait, Value};
use tardis::serde_json::json;
use tardis::tokio;
use tardis::TardisFuns;

fn task_context(owner: &str, own_paths: &str) -> TardisContext {
    TardisContext {
        own_paths: own_paths.to_string(),
        owner: owner.to_string(),
        ..Default::default()
    }
}

fn create_req(task_id: &str, expected_grant_id: Option<&str>) -> IamOAuth2TaskGrantCreateReq {
    IamOAuth2TaskGrantCreateReq {
        task_id: task_id.to_string(),
        requires_external_oauth: false,
        expected_grant_id: expected_grant_id.map(str::to_string),
        takeover: false,
    }
}

async fn create_grant(req: IamOAuth2TaskGrantCreateReq, ctx: TardisContext) -> TardisResult<IamOAuth2TaskGrantRef> {
    let mut tx_funs = iam_constants::get_tardis_inst();
    tx_funs.begin().await?;
    match IamOAuth2TaskGrantServ::create(&req, None, &tx_funs, &ctx).await {
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

#[tokio::test]
#[ignore = "requires Docker for isolated PostgreSQL, Redis and RabbitMQ"]
async fn task_grant_concurrent_replacement_and_initializer_index_are_stable() -> TardisResult<()> {
    std::env::set_var("TARDIS_CSM.IAM.OAUTH2_TASK_EXECUTOR_ACCOUNT_ID", "scheduler");
    std::env::set_var("TARDIS_CSM.IAM.OAUTH2_PROVIDER_GRANT_KEY", "09".repeat(32));
    let _containers = init_test_container::init(None).await?;
    let funs = iam_constants::get_tardis_inst();
    bios_iam::iam_initializer::init_db(iam_constants::get_tardis_inst()).await?;

    let system_ctx = get_first_account_context(iam_constants::RBUM_KIND_CODE_IAM_ACCOUNT, iam_constants::COMPONENT_CODE, &funs).await?.unwrap();
    let account_id = IamAccountServ::add_item(
        &mut TardisFuns::json.json_to_obj::<IamAccountAddReq>(json!({
            "name": "Concurrent replacement owner",
            "scope_level": iam_constants::RBUM_SCOPE_LEVEL_GLOBAL,
            "status": bios_iam::iam_enumeration::IamAccountStatusKind::Active
        }))?,
        &funs,
        &system_ctx,
    )
    .await?;
    let (tenant_id, _, _) = IamTenantServ::add_tenant_agg(
        &IamTenantAggAddReq {
            name: TrimString::from("Task grant security tenant"),
            icon: None,
            contact_phone: None,
            note: None,
            account_self_reg: None,
            disabled: None,
            admin_username: TrimString::from("task-grant-security-admin"),
            admin_password: None,
            admin_phone: None,
            admin_mail: None,
            admin_name: TrimString::from("Task grant security admin"),
            audit_username: TrimString::from("task-grant-security-audit"),
            audit_password: None,
            audit_phone: None,
            audit_mail: None,
            audit_name: TrimString::from("Task grant security auditor"),
            cert_conf_by_oauth2: None,
            cert_conf_by_ldap: None,
        },
        &funs,
        &system_ctx,
    )
    .await?;
    let tenant_ctx = task_context(&account_id, &tenant_id);
    let app_id = IamAppServ::add_app_agg(
        &IamAppAggAddReq {
            app_name: TrimString::from("Concurrent replacement app"),
            app_description: None,
            app_icon: None,
            app_sort: None,
            app_contact_phone: None,
            admin_ids: Some(vec![account_id.clone()]),
            publish_system_ids: None,
            disabled: None,
            set_cate_id: None,
            kind: None,
            sync_apps_group: Some(false),
        },
        &funs,
        &tenant_ctx,
    )
    .await?;
    let user_ctx = task_context(&account_id, &format!("{}/{app_id}", tenant_ctx.own_paths));

    let initial = create_grant(create_req("binding/replacement-race", None), user_ctx.clone()).await?;

    // Re-running the real startup initializer must leave the existing table and
    // its nullable current-slot unique index intact.
    bios_iam::iam_initializer::init_db(iam_constants::get_tardis_inst()).await?;
    let index_rows = funs
        .db()
        .query_all(
            "SELECT indexname, indexdef FROM pg_indexes WHERE schemaname = current_schema() AND tablename = $1 AND indexname = $2",
            vec![Value::from("iam_oauth2_task_grant"), Value::from("ux_iam_oauth2_task_grant_current_slot")],
        )
        .await?;
    assert_eq!(index_rows.len(), 1, "initializer must not duplicate or omit the current-slot index");
    let index_def: String = index_rows[0].try_get("", "indexdef")?;
    assert!(index_def.starts_with("CREATE UNIQUE INDEX"), "current-slot index must remain unique: {index_def}");
    assert!(
        index_def.contains("current_own_paths") && index_def.contains("current_task_id"),
        "index must cover both current-slot columns: {index_def}"
    );

    let barrier = Arc::new(tokio::sync::Barrier::new(2));
    let barrier_one = barrier.clone();
    let barrier_two = barrier.clone();
    let ctx_one = task_context(&account_id, &user_ctx.own_paths);
    let ctx_two = task_context(&account_id, &user_ctx.own_paths);
    let expected_one = initial.grant_id.clone();
    let expected_two = initial.grant_id.clone();
    let replace_one = async move {
        barrier_one.wait().await;
        create_grant(create_req("binding/replacement-race", Some(&expected_one)), ctx_one).await
    };
    let replace_two = async move {
        barrier_two.wait().await;
        create_grant(create_req("binding/replacement-race", Some(&expected_two)), ctx_two).await
    };
    let (result_one, result_two) = tokio::join!(replace_one, replace_two);
    assert_eq!(result_one.is_ok() as usize + result_two.is_ok() as usize, 1, "one CAS replacement must win");
    let winner = match (result_one, result_two) {
        (Ok(winner), Err(error)) | (Err(error), Ok(winner)) => {
            assert_eq!(error.code, "409-iam-oauth-task-grant-conflict", "the losing CAS must return the stable conflict code");
            winner
        }
        _ => unreachable!("exactly one concurrent replacement must succeed"),
    };

    let current = IamOAuth2TaskGrantServ::current("binding/replacement-race", &funs, &user_ctx).await?.unwrap();
    assert_eq!(current.grant_id, winner.grant_id);
    let original = iam_oauth2_task_grant::Entity::find_by_id(&initial.grant_id).one(funs.db().raw_conn()).await?.unwrap();
    assert!(original.revoked);
    assert_eq!(original.revoked_by.as_deref(), Some(account_id.as_str()));
    assert!(original.revoked_time.is_some());
    assert_eq!(original.replaced_by_grant_id.as_deref(), Some(winner.grant_id.as_str()));
    let current_rows = funs
        .db()
        .query_all(
            "SELECT id FROM iam_oauth2_task_grant WHERE current_own_paths = $1 AND current_task_id = $2",
            vec![Value::from(user_ctx.own_paths.clone()), Value::from("binding/replacement-race")],
        )
        .await?;
    assert_eq!(current_rows.len(), 1, "exactly one current grant may remain after competing replacements");
    Ok(())
}
