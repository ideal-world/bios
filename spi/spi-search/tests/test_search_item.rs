use bios_basic::test::test_http_client::TestHttpClient;
use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;

pub async fn test(client: &mut TestHttpClient) -> TardisResult<()> {
    client.set_auth(&TardisContext {
        own_paths: "t1/a1".to_string(),
        ak: "".to_string(),
        roles: vec![],
        groups: vec![],
        owner: "app001".to_string(),
        ..Default::default()
    })?;

    Ok(())
}
