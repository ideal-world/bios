use bios_basic::test::test_http_client::TestHttpClient;
use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;

pub async fn test(client: &mut TestHttpClient) -> TardisResult<()> {
    client.set_auth(&TardisContext {
        own_paths: "t1/app001".to_string(),
        ak: "".to_string(),
        roles: vec![],
        groups: vec![],
        owner: "app001".to_string(),
        ..Default::default()
    })?;

    let upload_url: String = client.get(&format!("/ci/obj/presign/put?object_path={}&exp_secs=300&private=true", "a/001.txt")).await;
    println!("upload_url: {}\r\nE.g.\r\n curl -X PUT -d @CNAME \"{}\"", upload_url, upload_url);

    let view_url: String = client.get(&format!("/ci/obj/presign/view?object_path={}&exp_secs=300&private=true", "a/001.txt")).await;
    println!("view_url: {}\r\nE.g.\r\nwget -O ./a.txt \"{}\"", view_url, view_url);

    let delete_url: String = client.get(&format!("/ci/obj/presign/delete?object_path={}&exp_secs=300&private=true", "a/001.txt")).await;
    println!("delete_url: {}\r\nE.g.\r\ncurl -X DELETE \"{}\"", delete_url, delete_url);

    // tardis::tokio::time::sleep(std::time::Duration::from_secs(300)).await;
    Ok(())
}
