use std::collections::HashMap;

use bios_basic::test::test_http_client::TestHttpClient;
use bios_spi_plugin::dto::plugin_exec_dto::{PluginExecReq, PluginExecResp};
use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::TardisFuns;

pub async fn test(client: &mut TestHttpClient) -> TardisResult<()> {
    client.set_auth(&TardisContext {
        own_paths: "t1/app001".to_string(),
        ak: "".to_string(),
        roles: vec![],
        groups: vec![],
        owner: "app001".to_string(),
        ..Default::default()
    })?;

    let resp: PluginExecResp = client
        .post(
            &format!("/ci/spi/plugin/{}/api/{}/exec", "gitlib", "test-api"),
            &PluginExecReq {
                header: Some(HashMap::from([(
                    "Tardis-Context".to_string(),
                    TardisFuns::crypto.base64.encode(&TardisFuns::json.obj_to_string(&client.context()).unwrap()),
                )])),
                body: Some(HashMap::from([("msg".to_string(), "plugin exec!")])),
            },
        )
        .await;
    println!("resp: {},{}", resp.code, resp.body.unwrap());
    Ok(())
}
