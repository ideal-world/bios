use std::env;
use std::time::Duration;

use bios_basic::rbum::rbum_enumeration::RbumCertStatusKind;
use bios_basic::rbum::rbum_initializer::get_first_account_context;
use bios_basic::test::init_test_container;
use bios_iam::basic::dto::iam_account_dto::IamAccountAggAddReq;
use bios_iam::basic::serv::iam_account_serv::IamAccountServ;
use bios_iam::iam_constants;
use bios_iam::iam_initializer;
use bios_iam::integration::ldap::ldap_server;
use ldap3::{Ldap, LdapConnAsync, LdapConnSettings, Scope, SearchEntry};
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::log::info;
use tardis::tokio::time::sleep;
use tardis::{tokio, TardisFuns};

#[tokio::test]
async fn test_ldap_account() -> TardisResult<()> {
    env::set_var("RUST_LOG", "debug,test_ldap=trace,sqlx::query=off");
    
    // 初始化测试容器（数据库等）
    let _x = init_test_container::init(None).await?;
    
    // 初始化 Tardis
    TardisFuns::init(Some("tests/config")).await?;
    
    // 初始化 IAM 数据库
    let funs = iam_constants::get_tardis_inst();
    let (sysadmin_name, sysadmin_password) = iam_initializer::init_db(funs).await?.unwrap();
    
    info!("[Test] System admin created: {} / {}", sysadmin_name, sysadmin_password);
    
    // 获取系统管理员上下文
    let system_admin_context = get_first_account_context(
        iam_constants::RBUM_KIND_CODE_IAM_ACCOUNT,
        iam_constants::COMPONENT_CODE,
        &TardisFuns::inst_with_db_conn("".to_string(), None),
    )
    .await?
    .unwrap();
    
    // 创建测试账户
    let funs = iam_constants::get_tardis_inst();
    let test_username = "testuser";
    let test_password = "testpass123";
    let test_account_name = "测试用户";
    
    info!("[Test] Creating test account: {}", test_username);
    let account_id = IamAccountServ::add_account_agg(
        &IamAccountAggAddReq {
            id: None,
            name: TrimString(test_account_name.to_string()),
            cert_user_name: TrimString(test_username.to_string()),
            cert_password: Some(TrimString(test_password.to_string())),
            cert_phone: None,
            cert_mail: None,
            role_ids: None,
            org_node_ids: None,
            scope_level: None,
            disabled: None,
            icon: None,
            exts: None,
            status: Some(RbumCertStatusKind::Enabled),
            temporary: None,
            lock_status: None,
            logout_type: None,
            labor_type: None,
            id_card_no: None,
            employee_code: None,
            others_id: None,
        },
        false,
        &funs,
        &system_admin_context,
    )
    .await?;
    
    info!("[Test] Test account created with ID: {}", account_id);
    
    // 等待一下确保数据已写入
    sleep(Duration::from_millis(500)).await;
    
    // 启动 LDAP 服务器（在后台）
    info!("[Test] Starting LDAP server...");
    ldap_server::start().await?;
    
    // 等待 LDAP 服务器启动
    sleep(Duration::from_millis(1000)).await;
    
    // 获取 LDAP 配置
    let ldap_config = funs.conf::<bios_iam::iam_config::IamConfig>().ldap.clone();
    let ldap_port = ldap_config.port;
    let ldap_dc = ldap_config.dc.clone();
    let bind_dn = ldap_config.bind_dn.clone();
    let bind_password = ldap_config.bind_password.clone();
    
    info!("[Test] LDAP server running on port: {}", ldap_port);
    info!("[Test] LDAP DC: {}", ldap_dc);
    
    // 使用 LDAP 客户端连接
    let ldap_url = format!("ldap://127.0.0.1:{}", ldap_port);
    let base_dn = format!("DC={}", ldap_dc);
    
    info!("[Test] Connecting to LDAP server: {}", ldap_url);
    
    let settings = LdapConnSettings::new();
    let (conn, mut ldap) = LdapConnAsync::with_settings(settings, &ldap_url)
        .await
        .map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP connection error: {e:?}"), "500-ldap-connection-error"))?;
    
    ldap3::drive!(conn);
    
    // 测试 1: 使用管理员账户绑定
    info!("[Test] Test 1: Binding with admin DN: {}", bind_dn);
    let bind_result = ldap
        .simple_bind(&bind_dn, &bind_password)
        .await
        .map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP bind error: {e:?}"), "500-ldap-bind-error"))?
        .success();
    
    assert!(bind_result.is_ok(), "Admin bind should succeed");
    info!("[Test] Admin bind successful");
    
    // 测试 2: 使用测试用户账户绑定
    let test_user_dn = format!("CN={},DC={}", test_username, ldap_dc);
    info!("[Test] Test 2: Binding with test user DN: {}", test_user_dn);
    
    let user_bind_result = ldap
        .simple_bind(&test_user_dn, test_password)
        .await
        .map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP user bind error: {e:?}"), "500-ldap-bind-error"))?
        .success();
    
    assert!(user_bind_result.is_ok(), "User bind should succeed");
    info!("[Test] User bind successful");
    
    // 重新绑定管理员以进行搜索
    ldap.simple_bind(&bind_dn, &bind_password)
        .await
        .map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP rebind error: {e:?}"), "500-ldap-bind-error"))?
        .success().map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP rebind error: {e:?}"), "500-ldap-bind-error"))?;
    
    // 测试 3: 搜索用户
    info!("[Test] Test 3: Searching for user: {}", test_username);
    let (rs, _res) = ldap
        .search(
            &base_dn,
            Scope::Subtree,
            &format!("(sAMAccountName={})", test_username),
            vec!["sAMAccountName", "cn", "mail"],
        )
        .await
        .map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP search error: {e:?}"), "500-ldap-search-error"))?
        .success().map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP search error: {e:?}"), "500-ldap-search-error"))?;
    
    let entries: Vec<SearchEntry> = rs
        .into_iter()
        .map(|entry| SearchEntry::construct(entry))
        .collect();
    
    assert!(!entries.is_empty(), "Should find at least one entry");
    info!("[Test] Found {} entries", entries.len());
    
    let found_entry = &entries[0];
    let found_cn = found_entry.attrs.get("cn").and_then(|v| v.first()).unwrap();
    assert_eq!(found_cn, test_username, "CN should match username");
    info!("[Test] Found user CN: {}", found_cn);
    
    // 测试 4: 使用 Present 过滤器搜索根 DSE（空 base + objectClass）
    // 注意：LDAP 服务器实现只支持空 base 的 objectClass 搜索，不支持在 DC 级别搜索所有用户
    info!("[Test] Test 4: Searching root DSE with empty base and objectClass filter");
    let (rs2, _res2) = ldap
        .search(
            "",  // 空 base 用于搜索根 DSE
            Scope::Base,  // Base scope 用于根 DSE
            "(objectClass=*)",
            vec!["sAMAccountName", "cn"],
        )
        .await
        .map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP search error: {e:?}"), "500-ldap-search-error"))?
        .success().map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP search error: {e:?}"), "500-ldap-search-error"))?;
    
    let entries2: Vec<SearchEntry> = rs2
        .into_iter()
        .map(|entry| SearchEntry::construct(entry))
        .collect();
    
    info!("[Test] Found {} entries in root DSE", entries2.len());
    // 根 DSE 应该至少返回 DC 条目
    assert!(!entries2.is_empty(), "Root DSE search should return at least one entry");
    
    // 测试 5: 使用错误的密码绑定应该失败
    info!("[Test] Test 5: Testing bind with wrong password (should fail)");
    let wrong_password_result = ldap
        .simple_bind(&test_user_dn, "wrongpassword")
        .await
        .map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP bind error: {e:?}"), "500-ldap-bind-error"))?
        .success();
    
    assert!(wrong_password_result.is_err(), "Bind with wrong password should fail");
    info!("[Test] Wrong password bind correctly failed");
    
    // 重新绑定管理员以进行清理
    ldap.simple_bind(&bind_dn, &bind_password)
        .await
        .map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP rebind error: {e:?}"), "500-ldap-bind-error"))?
        .success().map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP rebind error: {e:?}"), "500-ldap-bind-error"))?;
    
    ldap.unbind().await.map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP unbind error: {e:?}"), "500-ldap-unbind-error"))?;
    
    info!("[Test] All LDAP tests passed!");
    Ok(())
}
