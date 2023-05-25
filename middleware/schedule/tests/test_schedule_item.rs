use bios_basic::test::test_http_client::TestHttpClient;

use tardis::basic::result::TardisResult;
use tardis::serde_json::json;
use tardis::web::web_resp::Void;

use tardis::TardisFunsInst;
pub async fn test(client: &mut TestHttpClient, _funs: &TardisFunsInst) -> TardisResult<()> {
    let req = json!({
        "code": "test",
        "cron": "0/5 * * * * ?",
        "callback_url": "https://localhost:8080/schedule/ci/schedule/test/exec/123",
    });
    let _resp = client.put::<_, Void>("/ci/schedule/jobs", &req).await;
    let _resp = client.put::<_, Void>("/ci/schedule/jobs", &req).await;
    // tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
    client.delete("/ci/schedule/jobs/test").await;
    Ok(())
}
