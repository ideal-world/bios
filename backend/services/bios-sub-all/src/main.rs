use std::env;
use tardis::basic::result::TardisResult;
use tardis::tokio;
use tardis::TardisFuns;

mod initializer;

///
/// Visit: http://127.0.0.1:8080/
///
#[tokio::main]
async fn main() -> TardisResult<()> {
    env::set_var("RUST_LOG", "debug,tardis=trace,sqlx=off,bios=bios-serv-all,hyper::proto=off,sqlparser::parser=off");
    #[cfg(feature = "analysis")]
    let hostname = std::fs::read_to_string("/etc/hostname")?;
    #[cfg(feature = "analysis")]
    std::thread::spawn(move || {
        let hostname = hostname.lines().next().unwrap_or("bios-event");
        // analysis task
        loop {
            let current_time = tardis::chrono::Utc::now().to_rfc3339();
            let _profiler = dhat::Profiler::builder().file_name(format!("/report/dhat-{hostname}-[{current_time}].json")).build();
            std::thread::sleep(tokio::time::Duration::from_secs(60));
            drop(_profiler);
        }
    });
    TardisFuns::init(Some("config")).await?;
    let web_server = TardisFuns::web_server();
    initializer::init(&web_server).await?;
    web_server.start().await?;
    web_server.await;
    Ok(())
}
