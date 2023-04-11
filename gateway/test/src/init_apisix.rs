use tardis::basic::result::TardisResult;
use tardis::serde_json::json;
use tardis::testcontainers::clients::Cli;
use tardis::testcontainers::core::WaitFor;
use tardis::testcontainers::images::generic::GenericImage;
use tardis::testcontainers::{images, Container, RunnableImage};
use tardis::TardisFuns;

pub(crate) async fn init(docker: &Cli) -> TardisResult<(String, Container<GenericImage>)> {
    // Build docker image
    let (_code, output, _error) = run_script::run_script!(
        r#"
         docker rm -f apisix_test
         docker build -f gateway/test/conf/apisix/Dockerfile_with_etcd -t apisix_test gateway/
         "#
    )
    .unwrap();
    // println!("Exit Code: {}", code);
    // println!("Error: {}", error);
    println!("Output: {}", output);

    // Start docker container
    let image = images::generic::GenericImage::new("apisix_test", "latest").with_wait_for(WaitFor::seconds(15));
    let mut runnable_image: RunnableImage<GenericImage> = image.into();
    runnable_image = runnable_image.with_network("host");
    runnable_image = runnable_image.with_container_name("apisix_test");
    let node = docker.run(runnable_image);
    let url = format!("http://127.0.0.1:9080");
    // Init routes
    TardisFuns::web_client()
        .put_obj_to_str(
            &format!("{url}/apisix/admin/routes/1"),
            &json!({
                "uri": "/*",
                "plugins": {
                    "auth-bios": {
                        "host": "http://127.0.0.1:8080/auth",
                        "timeout": 60000
                    }
                },
                "upstream": {
                    "type": "roundrobin",
                    "nodes": {
                        "127.0.0.1:8080": 1
                    }
                }
            }),
            Some(vec![("X-API-KEY".to_string(), "edd1c9f034335f136f87ad84b6acecs1".to_string())]),
        )
        .await?;
    Ok((url, node))
}
