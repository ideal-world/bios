use std::env;
use tardis::basic::result::TardisResult;
use tardis::tokio;
use tardis::TardisFuns;

mod initializer;

///
/// LDAP Server for IAM Account
///
/// 通过 LDAP 协议暴露 IAM 的 account
///
#[tokio::main]
async fn main() -> TardisResult<()> {
    env::set_var("RUST_LOG", "debug,tardis=trace,sqlx=off,bios=bios-serv-ldap,hyper::proto=off,sqlparser::parser=off");
    TardisFuns::init(Some("config")).await?;
    initializer::init().await?;

    // 保持程序运行
    tokio::signal::ctrl_c().await.map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("Failed to listen for ctrl_c: {e}"), ""))?;
    Ok(())
}
