use std::env;
use std::rc::Rc;
use std::time::Duration;

use bios_basic::rbum::rbum_config::RbumConfig;
use bios_basic::test::init_test_container;
use bios_basic::test::test_http_client::TestHttpClient;
use bios_mw_event::event_constants::DOMAIN_CODE;
use bios_mw_event::event_initializer;
use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::cluster::cluster_processor::set_local_node_id;
use tardis::config::config_dto::TardisConfig;
use tardis::test::test_container::TardisTestContainer;
use tardis::tokio::time::sleep;
use tardis::{testcontainers, tokio, TardisFuns};

use tokio::io::AsyncReadExt;
use tokio::process::Command;

mod test_event_inmails;
mod test_event_with_event_code;
mod test_event_with_im;
mod test_event_without_mgr;
#[tokio::test(flavor = "multi_thread")]
async fn test_event() -> TardisResult<()> {
    if let Ok(kind) = env::var("KIND") {
        let db_url = env::var("DB").expect("DB is not set");
        let cache_url = env::var("CACHE").expect("CACHE is not set");
        let port = env::var("PORT").expect("PORT is not set").parse::<u16>().expect("invalid port");
        env::set_var("TARDIS_FW.DB.URL", &db_url);
        env::set_var("TARDIS_FW.CACHE.URL", &cache_url);
        env::set_var("PROFILE", port.to_string());
        server_side().await?;
    } else {
        env::set_var("RUST_LOG", "debug,tardis=trace,bios_mw_event=trace,test_event=trace,sqlx::query=off");
        let docker = testcontainers::clients::Cli::default();
        let reldb_container = TardisTestContainer::postgres_custom(None, &docker);
        let port = reldb_container.get_host_port_ipv4(5432);
        let db_url = format!("postgres://postgres:123456@127.0.0.1:{port}/test");
        env::set_var("TARDIS_FW.DB.URL", &db_url);

        let redis_container = TardisTestContainer::redis_custom(&docker);
        let port = redis_container.get_host_port_ipv4(6379);
        let redis_url = format!("redis://127.0.0.1:{port}/0");
        env::set_var("TARDIS_FW.CACHE.URL", &redis_url);
        let program = env::current_exe()?;
        for node_idx in 0..3 {
            let program = program.clone();
            let db_url = db_url.clone();
            let redis_url = redis_url.clone();
            let port = 8080 + node_idx;
            tokio::spawn(async move {
                let mut child = if cfg!(target_os = "windows") {
                    Command::new("cmd")
                        .env("DB", &db_url)
                        .env("CACHE", &redis_url)
                        .env("PORT", port.to_string())
                        .env("LS_COLORS", "rs=0:di=38;5;27:mh=44;38;5;15")
                        .env("KIND", "server")
                        .arg("/C")
                        .arg(program)
                        .stdout(std::process::Stdio::piped())
                        .stderr(std::process::Stdio::piped())
                        .spawn()?
                } else {
                    Command::new("sh")
                        .env("DB", &db_url)
                        .env("CACHE", &redis_url)
                        .env("PORT", port.to_string())
                        .env("LS_COLORS", "rs=0:di=38;5;27:mh=44;38;5;15")
                        .env("KIND", "server")
                        .arg("-c")
                        .arg(program)
                        .stdout(std::process::Stdio::piped())
                        .stderr(std::process::Stdio::piped())
                        .spawn()?
                };
                let mut buf = [0; 1024];
                let mut err_buf = [0; 1024];
                let mut stdout = child.stdout.take().unwrap();
                let mut stderr = child.stderr.take().unwrap();
                loop {
                    tokio::select! {
                        result = stdout.read(&mut buf) => {
                            let size = result?;
                            if size != 0 {
                                println!("node[{node_idx}]/stdout:");
                                println!("{}", String::from_utf8_lossy(&buf[..size]));
                            }
                        }
                        result = stderr.read(&mut err_buf) => {
                            let size = result?;
                            if size != 0 {
                                println!("node[{node_idx}]/stdout:");
                                println!("{}", String::from_utf8_lossy(&err_buf[..size]));
                            }
                        }
                        exit_code = child.wait() => {
                            if let Ok(exit_code) = exit_code {
                                return TardisResult::Ok(exit_code.success())
                            } else {
                                return TardisResult::Ok(false)
                            }
                        }

                    };
                }
            });
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
        tokio::time::sleep(Duration::from_secs(15)).await;
        client_side().await?;
    }

    Ok(())
}

async fn server_side() -> TardisResult<()> {
    TardisFuns::init(Some("tests/config")).await?;
    set_local_node_id(TardisFuns::field.nanoid());
    // Initialize RBUM
    let rbum_init_result = bios_basic::rbum::rbum_initializer::init(DOMAIN_CODE, RbumConfig::default()).await;

    let web_server = TardisFuns::web_server();
    // Initialize Event
    let _event_init_result = event_initializer::init(web_server.as_ref()).await.expect("fail to initialize");
    web_server.start().await?;
    web_server.await;
    Ok(())
}

async fn client_side() -> TardisResult<()> {
    TardisFuns::init(Some("tests/config")).await?;

    let ctx = TardisContext {
        own_paths: "".to_string(),
        ak: "".to_string(),
        roles: vec![],
        groups: vec![],
        owner: "".to_string(),
        ..Default::default()
    };

    // let mut client = TestHttpClient::new(format!("http://127.0.0.1:{}/{}",,  DOMAIN_CODE));
    let client_set = [8080, 8081, 8082]
        .into_iter()
        .map(|port| {
            let mut client = TestHttpClient::new(format!("http://127.0.0.1:{}/{}", port, DOMAIN_CODE));
            client.set_auth(&ctx).unwrap();
            client
        })
        .collect::<Vec<_>>();

    // test_event_without_mgr::test(&client_set.iter().collect::<Vec<_>>()).await?;
    // test_event_with_event_code::test(&client_set.iter().collect::<Vec<_>>()).await?;
    // test_event_with_im::test(&client_set.iter().collect::<Vec<_>>()).await?;
    test_event_inmails::test(&client_set.iter().collect::<Vec<_>>()).await?;
    Ok(())
}
