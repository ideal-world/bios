use bios_basic::rbum::rbum_initializer::get_first_account_context;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;
use bios_basic::test::init_test_container;
use bios_iam::basic::dto::iam_account_dto::IamAccountAddReq;
use bios_iam::basic::dto::iam_app_dto::{IamAppAggAddReq, IamAppModifyReq};
use bios_iam::basic::dto::iam_tenant_dto::IamTenantAggAddReq;
use bios_iam::basic::serv::iam_account_serv::IamAccountServ;
use bios_iam::basic::serv::iam_app_serv::IamAppServ;
use bios_iam::basic::serv::iam_key_cache_serv::IamIdentCacheServ;
use bios_iam::basic::serv::iam_tenant_serv::IamTenantServ;
use bios_iam::console_interface::serv::iam_ci_temp_serv::IamCiTempServ;
use bios_iam::iam_config::IamConfig;
use bios_iam::iam_constants;
use tardis::basic::dto::TardisContext;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::cache::AsyncCommands;
use tardis::serde_json::json;
use tardis::tokio;
use tardis::TardisFuns;

#[tokio::test]
async fn temp_product_ci_grants_revokes_and_replaces_apis() -> TardisResult<()> {
    let _container = init_test_container::init(None).await?;
    let funs = iam_constants::get_tardis_inst();
    bios_iam::iam_initializer::init_db(iam_constants::get_tardis_inst()).await?;
    let system_ctx = get_first_account_context(iam_constants::RBUM_KIND_CODE_IAM_ACCOUNT, iam_constants::COMPONENT_CODE, &funs).await?.unwrap();
    let account_id = IamAccountServ::add_item(
        &mut TardisFuns::json.json_to_obj::<IamAccountAddReq>(json!({
            "name": "Temp product account",
            "scope_level": iam_constants::RBUM_SCOPE_LEVEL_GLOBAL,
            "status": bios_iam::iam_enumeration::IamAccountStatusKind::Active
        }))?,
        &funs,
        &system_ctx,
    )
    .await?;
    let (tenant_id, _, _) = IamTenantServ::add_tenant_agg(
        &IamTenantAggAddReq {
            name: TrimString::from("Temp product tenant"),
            icon: None,
            contact_phone: None,
            note: None,
            account_self_reg: None,
            disabled: None,
            admin_username: TrimString::from("temp-product-admin"),
            admin_password: None,
            admin_phone: None,
            admin_mail: None,
            admin_name: TrimString::from("Temp product admin"),
            audit_username: TrimString::from("temp-product-audit"),
            audit_password: None,
            audit_phone: None,
            audit_mail: None,
            audit_name: TrimString::from("Temp product auditor"),
            cert_conf_by_oauth2: None,
            cert_conf_by_ldap: None,
        },
        &funs,
        &system_ctx,
    )
    .await?;
    let tenant_ctx = TardisContext {
        own_paths: tenant_id.clone(),
        owner: account_id.clone(),
        ..Default::default()
    };
    let product_id = IamAppServ::add_app_agg(
        &IamAppAggAddReq {
            app_name: TrimString::from("Temp product"),
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
        &funs,
        &tenant_ctx,
    )
    .await?;
    IamIdentCacheServ::add_aksk("platform-ak", "platform-sk", "", None, 0, &funs).await?;
    IamIdentCacheServ::add_aksk("tenant-ak", "tenant-sk", &tenant_id, None, 0, &funs).await?;
    IamIdentCacheServ::add_aksk("other-ak", "other-sk", "other-tenant", None, 0, &funs).await?;
    let platform_ctx = ak_context("platform-ak");
    let tenant_ak_ctx = ak_context("tenant-ak");
    let other_ak_ctx = ak_context("other-ak");

    let config = funs.conf::<IamConfig>();
    let product_key = format!("{}{account_id}", config.cache_key_temp_product_);
    let expected_paths = format!("{tenant_id}/{product_id}");

    IamCiTempServ::grant_product(&account_id, &product_id, &tenant_id, 10, &funs, &platform_ctx).await?;
    IamCiTempServ::grant_product(&account_id, &product_id, &tenant_id, 20, &funs, &tenant_ak_ctx).await?;
    let stored = funs.cache().hget(&product_key, &product_id).await?.expect("grant is stored");
    assert!(stored.contains("\"expire_at\":20"), "granting the same product overwrites the previous record: {stored}");
    assert!(stored.contains(&expected_paths), "{stored}");

    let mismatched = IamCiTempServ::grant_product(&account_id, &product_id, "other-tenant", 20, &funs, &platform_ctx).await;
    assert!(mismatched.is_err(), "a tenant id that is not the product path is rejected");
    let stored = funs.cache().hget(&product_key, &product_id).await?.expect("rejected grant leaves the previous record");
    assert!(stored.contains("\"expire_at\":20"), "{stored}");

    let outside = IamCiTempServ::grant_product(&account_id, &product_id, &tenant_id, 30, &funs, &other_ak_ctx).await;
    assert!(outside.is_err(), "a tenant access key cannot grant another tenant's product");
    let stored = funs.cache().hget(&product_key, &product_id).await?.expect("rejected grant leaves the previous record");
    assert!(stored.contains("\"expire_at\":20"), "{stored}");

    IamCiTempServ::replace_apis(&["iam-res://cp/account/*##GET".to_string(), "iam-res://cp/**##*".to_string()], &funs).await?;
    let mut conn = funs.cache().cmd().await?;
    let mut members: Vec<String> = conn.smembers(&config.cache_key_temp_api).await?;
    members.sort();
    assert_eq!(members, vec!["iam-res://cp/**##*".to_string(), "iam-res://cp/account/*##get".to_string()]);
    assert_eq!(funs.cache().get(&config.cache_key_temp_api_version).await?.as_deref(), Some("1"));

    let rejected = IamCiTempServ::replace_apis(&["not-a-uri##get".to_string()], &funs).await;
    assert!(rejected.is_err(), "an identifier the resource parser cannot accept is rejected");
    let host_wildcard = IamCiTempServ::replace_apis(&["iam-res://**##*".to_string()], &funs).await;
    assert!(host_wildcard.is_err(), "a host wildcard is rejected");
    let cross_host = IamCiTempServ::replace_apis(&["iam-res://*/**##get".to_string()], &funs).await;
    assert!(cross_host.is_err(), "a wildcard host is rejected");
    let mut members: Vec<String> = conn.smembers(&config.cache_key_temp_api).await?;
    members.sort();
    assert_eq!(members, vec!["iam-res://cp/**##*".to_string(), "iam-res://cp/account/*##get".to_string()]);
    assert_eq!(funs.cache().get(&config.cache_key_temp_api_version).await?.as_deref(), Some("1"));

    IamCiTempServ::require_platform_ak("platform-ak", &funs).await?;
    assert!(IamCiTempServ::require_platform_ak("tenant-ak", &funs).await.is_err());
    assert!(IamCiTempServ::require_platform_ak("missing-ak", &funs).await.is_err());

    let foreign_revoke = IamCiTempServ::revoke_product(&account_id, &product_id, &funs, &other_ak_ctx).await;
    assert!(foreign_revoke.is_err(), "a tenant access key cannot revoke another tenant's product");
    assert!(funs.cache().hget(&product_key, &product_id).await?.is_some());

    IamAppServ::modify_item(
        &product_id,
        &mut IamAppModifyReq {
            name: None,
            icon: None,
            description: None,
            sort: None,
            contact_phone: None,
            disabled: Some(true),
            scope_level: None,
        },
        &funs,
        &tenant_ctx,
    )
    .await?;
    let disabled = IamCiTempServ::grant_product(&account_id, &product_id, &tenant_id, 40, &funs, &platform_ctx).await;
    assert!(disabled.is_err(), "a disabled product cannot be granted");
    let stored = funs.cache().hget(&product_key, &product_id).await?.expect("disabled grant leaves the previous record");
    assert!(stored.contains("\"expire_at\":20"), "{stored}");

    IamCiTempServ::revoke_product(&account_id, &product_id, &funs, &platform_ctx).await?;
    assert!(funs.cache().hget(&product_key, &product_id).await?.is_none());
    let mut members: Vec<String> = conn.smembers(&config.cache_key_temp_api).await?;
    members.sort();
    assert_eq!(members, vec!["iam-res://cp/**##*".to_string(), "iam-res://cp/account/*##get".to_string()]);

    IamCiTempServ::replace_apis(&[], &funs).await?;
    let members: Vec<String> = conn.smembers(&config.cache_key_temp_api).await?;
    assert!(members.is_empty());
    assert_eq!(funs.cache().get(&config.cache_key_temp_api_version).await?.as_deref(), Some("2"));

    Ok(())
}

fn ak_context(ak: &str) -> TardisContext {
    TardisContext {
        ak: ak.to_string(),
        ..Default::default()
    }
}
