use tardis::basic::result::TardisResult;
use tardis::log::info;

use bios_iam::iam_constants;
use bios_iam::iam_initializer;
use bios_iam::integration::ldap::ldap_server;

pub async fn init() -> TardisResult<()> {
    info!("[BiosLdap] Initializing LDAP server...");
    
    // 初始化 IAM 数据库（不包含 web server API）
    let funs = iam_constants::get_tardis_inst();
    iam_initializer::init_db(funs).await?;
    
    // 启动 LDAP 服务器
    info!("[BiosLdap] Starting LDAP server...");
    ldap_server::start().await?;
    
    info!("[BiosLdap] LDAP server started successfully");
    Ok(())
}
