use std::env;

use bios_basic::process::task_processor::TaskProcessor;
use bios_basic::test::init_test_container;
use tardis::basic::result::TardisResult;
use tardis::{testcontainers, tokio, TardisFuns};

#[tokio::test]
async fn test_task_processor() -> TardisResult<()> {
    env::set_var("RUST_LOG", "debug,test_iam_serv=trace,sqlx::query=off,sqlparser=off");
    let _x = init_test_container::init(None).await?;

    let cache_client = TardisFuns::inst("".to_string(), None).cache();

    TaskProcessor::set_status("test1", 1, true, &cache_client).await?;
    assert!(TaskProcessor::check_status("test1", 1, &cache_client).await?);
    assert!(!TaskProcessor::check_status("test1", u32::MAX as u64, &cache_client).await?);
    assert!(!TaskProcessor::check_status("test1", u32::MAX as u64 + 1, &cache_client).await?);
    assert!(!TaskProcessor::check_status("test1", u64::MAX, &cache_client).await?);

    TaskProcessor::set_status("test1", u32::MAX as u64, true, &cache_client).await?;
    assert!(TaskProcessor::check_status("test1", 1, &cache_client).await?);
    assert!(TaskProcessor::check_status("test1", u32::MAX as u64, &cache_client).await?);
    assert!(!TaskProcessor::check_status("test1", u32::MAX as u64 + 1, &cache_client).await?);
    assert!(!TaskProcessor::check_status("test1", u64::MAX, &cache_client).await?);

    TaskProcessor::set_status("test1", u32::MAX as u64 + 1, true, &cache_client).await?;
    assert!(TaskProcessor::check_status("test1", 1, &cache_client).await?);
    assert!(TaskProcessor::check_status("test1", u32::MAX as u64, &cache_client).await?);
    assert!(TaskProcessor::check_status("test1", u32::MAX as u64 + 1, &cache_client).await?);
    assert!(!TaskProcessor::check_status("test1", u64::MAX, &cache_client).await?);

    TaskProcessor::set_status("test1", u64::MAX, true, &cache_client).await?;
    assert!(TaskProcessor::check_status("test1", 1, &cache_client).await?);
    assert!(TaskProcessor::check_status("test1", u32::MAX as u64, &cache_client).await?);
    assert!(TaskProcessor::check_status("test1", u32::MAX as u64 + 1, &cache_client).await?);
    assert!(TaskProcessor::check_status("test1", u64::MAX, &cache_client).await?);

    TaskProcessor::set_status("test1", 1, false, &cache_client).await?;
    assert!(!TaskProcessor::check_status("test1", 1, &cache_client).await?);
    assert!(TaskProcessor::check_status("test1", u32::MAX as u64, &cache_client).await?);
    assert!(TaskProcessor::check_status("test1", u32::MAX as u64 + 1, &cache_client).await?);
    assert!(TaskProcessor::check_status("test1", u64::MAX, &cache_client).await?);

    assert!(!TaskProcessor::check_status("test2", u32::MAX as u64 + 1, &cache_client).await?);

    Ok(())
}
