use bios_basic::test::init_test_container;
use bios_iam::console_interface::serv::iam_ci_temp_serv::IamCiTempServ;
use bios_iam::iam_config::IamConfig;
use bios_iam::iam_constants;
use tardis::basic::result::TardisResult;
use tardis::cache::AsyncCommands;
use tardis::tokio;

#[tokio::test]
async fn temp_product_ci_grants_revokes_and_replaces_apis() -> TardisResult<()> {
    let _container = init_test_container::init(None).await?;
    let funs = iam_constants::get_tardis_inst();
    let config = funs.conf::<IamConfig>();
    let account_id = "account-1";
    let product_id = "product-1";
    let product_key = format!("{}{account_id}", config.cache_key_temp_product_);

    IamCiTempServ::grant_product(account_id, product_id, "tenant-1", 10, &funs).await?;
    IamCiTempServ::grant_product(account_id, product_id, "tenant-1", 20, &funs).await?;
    let stored = funs.cache().hget(&product_key, product_id).await?.expect("grant is stored");
    assert!(stored.contains("\"expire_at\":20"), "granting the same product overwrites the previous record: {stored}");
    assert!(stored.contains("tenant-1/product-1"));

    IamCiTempServ::replace_apis(&["iam-res://cp/account/*##GET".to_string(), "iam-res://cp/**##*".to_string()], &funs).await?;
    let mut conn = funs.cache().cmd().await?;
    let mut members: Vec<String> = conn.smembers(&config.cache_key_temp_api).await?;
    members.sort();
    assert_eq!(members, vec!["iam-res://cp/**##*".to_string(), "iam-res://cp/account/*##get".to_string()]);
    assert_eq!(funs.cache().get(&config.cache_key_temp_api_version).await?.as_deref(), Some("1"));

    let rejected = IamCiTempServ::replace_apis(&["not-a-uri##get".to_string()], &funs).await;
    assert!(rejected.is_err(), "an identifier the resource parser cannot accept is rejected");
    let mut members: Vec<String> = conn.smembers(&config.cache_key_temp_api).await?;
    members.sort();
    assert_eq!(members, vec!["iam-res://cp/**##*".to_string(), "iam-res://cp/account/*##get".to_string()]);

    IamCiTempServ::revoke_product(account_id, product_id, &funs).await?;
    assert!(funs.cache().hget(&product_key, product_id).await?.is_none());
    let mut members: Vec<String> = conn.smembers(&config.cache_key_temp_api).await?;
    members.sort();
    assert_eq!(members, vec!["iam-res://cp/**##*".to_string(), "iam-res://cp/account/*##get".to_string()]);

    IamCiTempServ::replace_apis(&[], &funs).await?;
    let members: Vec<String> = conn.smembers(&config.cache_key_temp_api).await?;
    assert!(members.is_empty());
    assert_eq!(funs.cache().get(&config.cache_key_temp_api_version).await?.as_deref(), Some("2"));

    Ok(())
}
