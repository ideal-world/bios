use std::env;
use std::time::Duration;

use bios_basic::rbum::rbum_enumeration::RbumCertStatusKind;
use bios_basic::rbum::rbum_enumeration::RbumScopeLevelKind;
use bios_basic::rbum::rbum_initializer::get_first_account_context;
use bios_basic::test::init_test_container;
use bios_iam::basic::dto::iam_account_dto::IamAccountAggAddReq;
use bios_iam::basic::serv::iam_account_serv::IamAccountServ;
use bios_iam::iam_constants;
use bios_iam::iam_initializer;
use bios_iam::integration::ldap::ldap_server;
use ldap3::{LdapConnAsync, LdapConnSettings, Scope, SearchEntry};
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
    
    // 创建多个测试账户
    let funs = iam_constants::get_tardis_inst();
    let test_accounts = vec![
        ("testuser1", "testpass123", "测试用户1", "testuser1@example.com", "EMP001"),
        ("testuser2", "testpass456", "测试用户2", "testuser2@example.com", "EMP002"),
        ("testuser3", "testpass789", "测试用户3", "testuser3@example.com", "EMP003"),
    ];
    
    let mut created_accounts = Vec::new();
    for (username, password, account_name, email, employee_code) in &test_accounts {
        info!("[Test] Creating test account: {} (email: {}, employee_code: {})", username, email, employee_code);
        let account_id = IamAccountServ::add_account_agg(
            &IamAccountAggAddReq {
                id: None,
                name: TrimString(account_name.to_string()),
                cert_user_name: TrimString(username.to_string()),
                cert_password: Some(TrimString(password.to_string())),
                cert_phone: None,
                cert_mail: Some(TrimString(email.to_string())),
                role_ids: None,
                org_node_ids: None,
                scope_level: Some(RbumScopeLevelKind::Root),
                disabled: None,
                icon: None,
                exts: None,
                status: Some(RbumCertStatusKind::Enabled),
                temporary: None,
                lock_status: None,
                logout_type: None,
                labor_type: None,
                id_card_no: None,
                employee_code: Some(employee_code.to_string()),
                others_id: None,
            },
            false,
            &funs,
            &system_admin_context,
        )
        .await?;
        
        created_accounts.push((username, password, account_id.clone()));
        info!("[Test] Test account created: {} with ID: {}", username, account_id);
    }
    
    // 使用第一个账号进行后续的单个用户测试
    let test_username = created_accounts[0].0;
    let test_password = created_accounts[0].1;
    
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
    let requested_attrs = vec!["sAMAccountName", "cn", "mail"];
    let (rs, _res) = ldap
        .search(
            &base_dn,
            Scope::Subtree,
            &format!("(sAMAccountName={})", test_username),
            requested_attrs.clone(),
        )
        .await
        .map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP search error: {e:?}"), "500-ldap-search-error"))?
        .success().map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP search error: {e:?}"), "500-ldap-search-error"))?;
    
    let entries: Vec<SearchEntry> = rs
        .into_iter()
        .map(SearchEntry::construct)
        .collect();
    
    assert!(!entries.is_empty(), "Should find at least one entry");
    info!("[Test] Found {} entries", entries.len());
    
    let found_entry = &entries[0];
    
    // 验证返回的属性与请求的属性一致
    let returned_attrs: Vec<String> = found_entry.attrs.keys().map(|k| k.to_lowercase()).collect();
    let requested_attrs_lower: Vec<String> = requested_attrs.iter().map(|a| a.to_lowercase()).collect();
    
    info!("[Test] Requested attributes: {:?}", requested_attrs);
    info!("[Test] Returned attributes: {:?}", found_entry.attrs.keys().collect::<Vec<_>>());
    
    // 验证所有请求的属性都存在
    for requested_attr in &requested_attrs_lower {
        assert!(
            returned_attrs.contains(requested_attr),
            "Requested attribute '{}' should be present in response, but got: {:?}",
            requested_attr,
            returned_attrs
        );
    }
    
    // 验证返回的属性数量不超过请求的数量（可能相等，因为可能包含一些必需属性）
    // 注意：LDAP可能返回一些额外的必需属性，所以这里只验证请求的属性都存在
    
    // 验证属性值
    let found_cn = found_entry.attrs.get("cn").and_then(|v| v.first()).unwrap();
    assert_eq!(found_cn, test_username, "CN should match username");
    
    let found_sam = found_entry.attrs.get("sAMAccountName").and_then(|v| v.first()).unwrap();
    assert_eq!(found_sam, test_username, "sAMAccountName should match username");
    
    let found_mail = found_entry.attrs.get("mail").and_then(|v| v.first());
    assert!(found_mail.is_some(), "Mail attribute should be present");
    let expected_mail = "testuser1@example.com";
    assert_eq!(found_mail.unwrap(), expected_mail, "Mail should match expected value");
    
    info!("[Test] Found user CN: {}, sAMAccountName: {}, mail: {}", found_cn, found_sam, found_mail.unwrap());
    
    // 验证不应该返回的属性确实没有被返回
    // 这些属性在 LDAP 服务器中可能存在，但不在请求列表中，所以不应该被返回
    let unrequested_attrs = vec!["uid", "employeeNumber", "displayName", "objectClass", "ou", "employeeType", "title", "businessCategory", "givenName", "sn"];
    for unrequested_attr in &unrequested_attrs {
        assert!(
            !found_entry.attrs.contains_key(*unrequested_attr),
            "Unrequested attribute '{}' should NOT be present in response, but it was found. Returned attributes: {:?}",
            unrequested_attr,
            found_entry.attrs.keys().collect::<Vec<_>>()
        );
    }
    info!("[Test] Verified that unrequested attributes are correctly filtered out: {:?}", unrequested_attrs);
    
    // 测试 3.5: 搜索所有测试用户（应该返回多个结果）
    info!("[Test] Test 3.5: Searching for all test users (should return multiple results)");
    let (rs_multi, _res_multi) = ldap
        .search(
            &base_dn,
            Scope::Subtree,
            "(|(sAMAccountName=testuser1)(sAMAccountName=testuser2)(sAMAccountName=testuser3))",
            vec!["sAMAccountName", "cn", "mail"],
        )
        .await
        .map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP search error: {e:?}"), "500-ldap-search-error"))?
        .success().map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP search error: {e:?}"), "500-ldap-search-error"))?;
    
    let entries_multi: Vec<SearchEntry> = rs_multi
        .into_iter()
        .map(SearchEntry::construct)
        .collect();
    
    info!("[Test] Found {} entries in multi-user search", entries_multi.len());
    assert!(entries_multi.len() >= 3, "Should find at least 3 test users, found {}", entries_multi.len());
    
    // 验证所有测试用户都被找到
    let mut found_usernames: Vec<String> = entries_multi
        .iter()
        .filter_map(|entry| {
            entry.attrs.get("sAMAccountName")
            .and_then(|v| v.first()).cloned()
        })
        .collect();
    
    found_usernames.sort();
    info!("[Test] Found usernames: {:?}", found_usernames);
    
    // 验证所有三个测试用户都被找到
    let expected_usernames: Vec<&str> = vec!["testuser1", "testuser2", "testuser3"];
    for expected_username in &expected_usernames {
        assert!(
            found_usernames.contains(&expected_username.to_string()),
            "Should find user: {}, found: {:?}",
            expected_username,
            found_usernames
        );
    }
    info!("[Test] All test users found successfully");
    
    // 测试 3.6: 使用 cn 字段进行等值匹配搜索
    info!("[Test] Test 3.6: Equality search using cn field");
    let (rs_cn, _res_cn) = ldap
        .search(
            &base_dn,
            Scope::Subtree,
            &format!("(cn={})", test_username),
            vec!["sAMAccountName", "cn"],
        )
        .await
        .map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP search error: {e:?}"), "500-ldap-search-error"))?
        .success().map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP search error: {e:?}"), "500-ldap-search-error"))?;
    
    let entries_cn: Vec<SearchEntry> = rs_cn
        .into_iter()
        .map(SearchEntry::construct)
        .collect();
    
    assert!(!entries_cn.is_empty(), "Should find at least one entry using cn");
    let found_cn_value = entries_cn[0].attrs.get("cn").and_then(|v| v.first()).unwrap();
    assert_eq!(found_cn_value, test_username, "CN should match username");
    info!("[Test] Found user using cn field: {}", found_cn_value);
    
    // 测试 3.7: 使用 uid 字段进行等值匹配搜索
    info!("[Test] Test 3.7: Equality search using uid field");
    let (rs_uid, _res_uid) = ldap
        .search(
            &base_dn,
            Scope::Subtree,
            &format!("(uid={})", test_username),
            vec!["sAMAccountName", "uid"],
        )
        .await
        .map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP search error: {e:?}"), "500-ldap-search-error"))?
        .success().map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP search error: {e:?}"), "500-ldap-search-error"))?;
    
    let entries_uid: Vec<SearchEntry> = rs_uid
        .into_iter()
        .map(SearchEntry::construct)
        .collect();
    
    assert!(!entries_uid.is_empty(), "Should find at least one entry using uid");
    let found_uid_value = entries_uid[0].attrs.get("sAMAccountName").and_then(|v| v.first()).unwrap();
    assert_eq!(found_uid_value, test_username, "UID should match username");
    info!("[Test] Found user using uid field: {}", found_uid_value);
    
    // 测试 3.8: 使用 mail 字段进行等值匹配搜索
    info!("[Test] Test 3.8: Equality search using mail field");
    let test_email = "testuser1@example.com";
    let (rs_mail, _res_mail) = ldap
        .search(
            &base_dn,
            Scope::Subtree,
            &format!("(mail={})", test_email),
            vec!["sAMAccountName", "mail"],
        )
        .await
        .map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP search error: {e:?}"), "500-ldap-search-error"))?
        .success().map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP search error: {e:?}"), "500-ldap-search-error"))?;
    
    let entries_mail: Vec<SearchEntry> = rs_mail
        .into_iter()
        .map(SearchEntry::construct)
        .collect();
    
    assert!(!entries_mail.is_empty(), "Should find at least one entry using mail");
    let found_mail_value = entries_mail[0].attrs.get("mail").and_then(|v| v.first()).unwrap();
    assert_eq!(found_mail_value, test_email, "Mail should match");
    let found_mail_username = entries_mail[0].attrs.get("sAMAccountName").and_then(|v| v.first()).unwrap();
    assert_eq!(found_mail_username, test_username, "Username should match");
    info!("[Test] Found user using mail field: {} (username: {})", found_mail_value, found_mail_username);
    
    // 测试 3.9: 使用 employeeNumber 字段进行等值匹配搜索
    info!("[Test] Test 3.9: Equality search using employeeNumber field");
    let test_employee_code = "EMP001";
    let (rs_emp, _res_emp) = ldap
        .search(
            &base_dn,
            Scope::Subtree,
            &format!("(employeeNumber={})", test_employee_code),
            vec!["sAMAccountName", "employeeNumber"],
        )
        .await
        .map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP search error: {e:?}"), "500-ldap-search-error"))?
        .success().map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP search error: {e:?}"), "500-ldap-search-error"))?;
    
    let entries_emp: Vec<SearchEntry> = rs_emp
        .into_iter()
        .map(SearchEntry::construct)
        .collect();
    
    assert!(!entries_emp.is_empty(), "Should find at least one entry using employeeNumber");
    let found_emp_username = entries_emp[0].attrs.get("sAMAccountName").and_then(|v| v.first()).unwrap();
    assert_eq!(found_emp_username, test_username, "Username should match");
    info!("[Test] Found user using employeeNumber field: {} (username: {})", test_employee_code, found_emp_username);
    
    // 测试 3.10: 使用 displayName 字段进行等值匹配搜索
    info!("[Test] Test 3.10: Equality search using displayName field");
    let test_displayName = "测试用户1";
    let (rs_display, _res_display) = ldap
        .search(
            &base_dn,
            Scope::Subtree,
            &format!("(displayName={})", test_displayName),
            vec!["sAMAccountName", "displayName"],
        )
        .await
        .map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP search error: {e:?}"), "500-ldap-search-error"))?
        .success().map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP search error: {e:?}"), "500-ldap-search-error"))?;
    
    let entries_display: Vec<SearchEntry> = rs_display
        .into_iter()
        .map(SearchEntry::construct)
        .collect();
    
    assert!(!entries_display.is_empty(), "Should find at least one entry using displayName");
    let found_display_username = entries_display[0].attrs.get("sAMAccountName").and_then(|v| v.first()).unwrap();
    assert_eq!(found_display_username, test_username, "Username should match");
    info!("[Test] Found user using displayName field: {} (username: {})", test_displayName, found_display_username);
    
    // 定义期望的用户名列表，用于后续的子串匹配测试
    let expected_usernames: Vec<&str> = vec!["testuser1", "testuser2", "testuser3"];
    
    // 测试 3.11: 子串匹配查询 - 前缀匹配（initial）
    info!("[Test] Test 3.11: Substring search - prefix match (initial)");
    let (rs_prefix, _res_prefix) = ldap
        .search(
            &base_dn,
            Scope::Subtree,
            "(sAMAccountName=test*)",
            vec!["sAMAccountName", "cn"],
        )
        .await
        .map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP search error: {e:?}"), "500-ldap-search-error"))?
        .success().map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP search error: {e:?}"), "500-ldap-search-error"))?;
    
    let entries_prefix: Vec<SearchEntry> = rs_prefix
        .into_iter()
        .map(SearchEntry::construct)
        .collect();
    
    assert!(entries_prefix.len() >= 3, "Prefix search should find at least 3 test users, found {}", entries_prefix.len());
    let mut found_prefix_usernames: Vec<String> = entries_prefix
        .iter()
        .filter_map(|entry| {
            entry.attrs.get("sAMAccountName")
                .and_then(|v| v.first()).cloned()
        })
        .collect();
    found_prefix_usernames.sort();
    info!("[Test] Prefix search found usernames: {:?}", found_prefix_usernames);
    for expected_username in &expected_usernames {
        assert!(
            found_prefix_usernames.contains(&expected_username.to_string()),
            "Prefix search should find user: {}, found: {:?}",
            expected_username,
            found_prefix_usernames
        );
    }
    info!("[Test] Prefix search completed successfully");
    
    // 测试 3.12: 子串匹配查询 - 后缀匹配（final）
    info!("[Test] Test 3.12: Substring search - suffix match (final)");
    let (rs_suffix, _res_suffix) = ldap
        .search(
            &base_dn,
            Scope::Subtree,
            "(sAMAccountName=*user1)",
            vec!["sAMAccountName", "cn"],
        )
        .await
        .map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP search error: {e:?}"), "500-ldap-search-error"))?
        .success().map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP search error: {e:?}"), "500-ldap-search-error"))?;
    
    let entries_suffix: Vec<SearchEntry> = rs_suffix
        .into_iter()
        .map(SearchEntry::construct)
        .collect();
    
    assert!(!entries_suffix.is_empty(), "Suffix search should find at least one entry");
    let found_suffix_username = entries_suffix[0].attrs.get("sAMAccountName").and_then(|v| v.first()).unwrap();
    assert_eq!(found_suffix_username, "testuser1", "Suffix search should find testuser1");
    info!("[Test] Suffix search found user: {}", found_suffix_username);
    
    // 测试 3.13: 子串匹配查询 - 任意位置匹配（any）
    info!("[Test] Test 3.13: Substring search - any position match (any)");
    let (rs_any, _res_any) = ldap
        .search(
            &base_dn,
            Scope::Subtree,
            "(sAMAccountName=*user*)",
            vec!["sAMAccountName", "cn"],
        )
        .await
        .map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP search error: {e:?}"), "500-ldap-search-error"))?
        .success().map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP search error: {e:?}"), "500-ldap-search-error"))?;
    
    let entries_any: Vec<SearchEntry> = rs_any
        .into_iter()
        .map(SearchEntry::construct)
        .collect();
    
    assert!(entries_any.len() >= 3, "Any position search should find at least 3 test users, found {}", entries_any.len());
    let mut found_any_usernames: Vec<String> = entries_any
        .iter()
        .filter_map(|entry| {
            entry.attrs.get("sAMAccountName")
                .and_then(|v| v.first()).cloned()
        })
        .collect();
    found_any_usernames.sort();
    info!("[Test] Any position search found usernames: {:?}", found_any_usernames);
    for expected_username in &expected_usernames {
        assert!(
            found_any_usernames.contains(&expected_username.to_string()),
            "Any position search should find user: {}, found: {:?}",
            expected_username,
            found_any_usernames
        );
    }
    info!("[Test] Any position search completed successfully");
    
    // 测试 3.14: 子串匹配查询 - 前缀 + 任意位置组合（initial + any）
    info!("[Test] Test 3.14: Substring search - prefix + any position combination (initial + any)");
    let (rs_prefix_any, _res_prefix_any) = ldap
        .search(
            &base_dn,
            Scope::Subtree,
            "(sAMAccountName=test*user*)",
            vec!["sAMAccountName", "cn"],
        )
        .await
        .map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP search error: {e:?}"), "500-ldap-search-error"))?
        .success().map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP search error: {e:?}"), "500-ldap-search-error"))?;
    
    let entries_prefix_any: Vec<SearchEntry> = rs_prefix_any
        .into_iter()
        .map(SearchEntry::construct)
        .collect();
    
    assert!(entries_prefix_any.len() >= 3, "Prefix + any search should find at least 3 test users, found {}", entries_prefix_any.len());
    let mut found_prefix_any_usernames: Vec<String> = entries_prefix_any
        .iter()
        .filter_map(|entry| {
            entry.attrs.get("sAMAccountName")
                .and_then(|v| v.first()).cloned()
        })
        .collect();
    found_prefix_any_usernames.sort();
    info!("[Test] Prefix + any search found usernames: {:?}", found_prefix_any_usernames);
    for expected_username in &expected_usernames {
        assert!(
            found_prefix_any_usernames.contains(&expected_username.to_string()),
            "Prefix + any search should find user: {}, found: {:?}",
            expected_username,
            found_prefix_any_usernames
        );
    }
    info!("[Test] Prefix + any search completed successfully");
    
    // 测试 3.15: 子串匹配查询 - 任意位置 + 后缀组合（any + final）
    info!("[Test] Test 3.15: Substring search - any position + suffix combination (any + final)");
    let (rs_any_suffix, _res_any_suffix) = ldap
        .search(
            &base_dn,
            Scope::Subtree,
            "(sAMAccountName=*user*1)",
            vec!["sAMAccountName", "cn"],
        )
        .await
        .map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP search error: {e:?}"), "500-ldap-search-error"))?
        .success().map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP search error: {e:?}"), "500-ldap-search-error"))?;
    
    let entries_any_suffix: Vec<SearchEntry> = rs_any_suffix
        .into_iter()
        .map(SearchEntry::construct)
        .collect();
    
    assert!(!entries_any_suffix.is_empty(), "Any + suffix search should find at least one entry");
    let found_any_suffix_username = entries_any_suffix[0].attrs.get("sAMAccountName").and_then(|v| v.first()).unwrap();
    assert_eq!(found_any_suffix_username, "testuser1", "Any + suffix search should find testuser1");
    info!("[Test] Any + suffix search found user: {}", found_any_suffix_username);
    
    // 测试 3.16: 子串匹配查询 - 前缀 + 后缀组合（initial + final）
    info!("[Test] Test 3.16: Substring search - prefix + suffix combination (initial + final)");
    let (rs_prefix_suffix, _res_prefix_suffix) = ldap
        .search(
            &base_dn,
            Scope::Subtree,
            "(sAMAccountName=test*1)",
            vec!["sAMAccountName", "cn"],
        )
        .await
        .map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP search error: {e:?}"), "500-ldap-search-error"))?
        .success().map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP search error: {e:?}"), "500-ldap-search-error"))?;
    
    let entries_prefix_suffix: Vec<SearchEntry> = rs_prefix_suffix
        .into_iter()
        .map(SearchEntry::construct)
        .collect();
    
    assert!(!entries_prefix_suffix.is_empty(), "Prefix + suffix search should find at least one entry");
    let found_prefix_suffix_username = entries_prefix_suffix[0].attrs.get("sAMAccountName").and_then(|v| v.first()).unwrap();
    assert_eq!(found_prefix_suffix_username, "testuser1", "Prefix + suffix search should find testuser1");
    info!("[Test] Prefix + suffix search found user: {}", found_prefix_suffix_username);
    
    // 测试 3.17: 子串匹配查询 - 完整组合（initial + any + final）
    info!("[Test] Test 3.17: Substring search - full combination (initial + any + final)");
    let (rs_full_combo, _res_full_combo) = ldap
        .search(
            &base_dn,
            Scope::Subtree,
            "(sAMAccountName=test*user*1)",
            vec!["sAMAccountName", "cn"],
        )
        .await
        .map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP search error: {e:?}"), "500-ldap-search-error"))?
        .success().map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP search error: {e:?}"), "500-ldap-search-error"))?;
    
    let entries_full_combo: Vec<SearchEntry> = rs_full_combo
        .into_iter()
        .map(SearchEntry::construct)
        .collect();
    
    assert!(!entries_full_combo.is_empty(), "Full combination search should find at least one entry");
    let found_full_combo_username = entries_full_combo[0].attrs.get("sAMAccountName").and_then(|v| v.first()).unwrap();
    assert_eq!(found_full_combo_username, "testuser1", "Full combination search should find testuser1");
    info!("[Test] Full combination search found user: {}", found_full_combo_username);
    
    // 测试 3.18: 子串匹配查询 - 多个任意位置匹配（multiple any）
    info!("[Test] Test 3.18: Substring search - multiple any positions");
    let (rs_multi_any, _res_multi_any) = ldap
        .search(
            &base_dn,
            Scope::Subtree,
            "(sAMAccountName=*user*test*)",
            vec!["sAMAccountName", "cn"],
        )
        .await
        .map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP search error: {e:?}"), "500-ldap-search-error"))?
        .success().map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP search error: {e:?}"), "500-ldap-search-error"))?;
    
    let entries_multi_any: Vec<SearchEntry> = rs_multi_any
        .into_iter()
        .map(SearchEntry::construct)
        .collect();
    
    assert!(entries_multi_any.len() >= 3, "Multiple any positions search should find at least 3 test users, found {}", entries_multi_any.len());
    let mut found_multi_any_usernames: Vec<String> = entries_multi_any
        .iter()
        .filter_map(|entry| {
            entry.attrs.get("sAMAccountName")
                .and_then(|v| v.first()).cloned()
        })
        .collect();
    found_multi_any_usernames.sort();
    info!("[Test] Multiple any positions search found usernames: {:?}", found_multi_any_usernames);
    for expected_username in &expected_usernames {
        assert!(
            found_multi_any_usernames.contains(&expected_username.to_string()),
            "Multiple any positions search should find user: {}, found: {:?}",
            expected_username,
            found_multi_any_usernames
        );
    }
    info!("[Test] Multiple any positions search completed successfully");
    
    // 测试 3.19: 子串匹配查询 - 在不同字段上测试（mail 字段）
    info!("[Test] Test 3.19: Substring search - on mail field");
    let (rs_mail_substr, _res_mail_substr) = ldap
        .search(
            &base_dn,
            Scope::Subtree,
            "(mail=*@example.com)",
            vec!["sAMAccountName", "mail"],
        )
        .await
        .map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP search error: {e:?}"), "500-ldap-search-error"))?
        .success().map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP search error: {e:?}"), "500-ldap-search-error"))?;
    
    let entries_mail_substr: Vec<SearchEntry> = rs_mail_substr
        .into_iter()
        .map(SearchEntry::construct)
        .collect();
    
    assert!(entries_mail_substr.len() >= 3, "Mail substring search should find at least 3 test users, found {}", entries_mail_substr.len());
    let mut found_mail_substr_usernames: Vec<String> = entries_mail_substr
        .iter()
        .filter_map(|entry| {
            entry.attrs.get("sAMAccountName")
                .and_then(|v| v.first()).cloned()
        })
        .collect();
    found_mail_substr_usernames.sort();
    info!("[Test] Mail substring search found usernames: {:?}", found_mail_substr_usernames);
    for expected_username in &expected_usernames {
        assert!(
            found_mail_substr_usernames.contains(&expected_username.to_string()),
            "Mail substring search should find user: {}, found: {:?}",
            expected_username,
            found_mail_substr_usernames
        );
    }
    info!("[Test] Mail substring search completed successfully");
    
    // 测试 3.20: 子串匹配查询 - 在 displayName 字段上测试（中文）
    info!("[Test] Test 3.20: Substring search - on displayName field (Chinese)");
    let (rs_display_substr, _res_display_substr) = ldap
        .search(
            &base_dn,
            Scope::Subtree,
            "(displayName=*用户*)",
            vec!["sAMAccountName", "displayName"],
        )
        .await
        .map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP search error: {e:?}"), "500-ldap-search-error"))?
        .success().map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP search error: {e:?}"), "500-ldap-search-error"))?;
    
    let entries_display_substr: Vec<SearchEntry> = rs_display_substr
        .into_iter()
        .map(SearchEntry::construct)
        .collect();
    
    assert!(entries_display_substr.len() >= 3, "Displayname substring search should find at least 3 test users, found {}", entries_display_substr.len());
    let mut found_display_substr_usernames: Vec<String> = entries_display_substr
        .iter()
        .filter_map(|entry| {
            entry.attrs.get("sAMAccountName")
                .and_then(|v| v.first()).cloned()
        })
        .collect();
    found_display_substr_usernames.sort();
    info!("[Test] Displayname substring search found usernames: {:?}", found_display_substr_usernames);
    for expected_username in &expected_usernames {
        assert!(
            found_display_substr_usernames.contains(&expected_username.to_string()),
            "Displayname substring search should find user: {}, found: {:?}",
            expected_username,
            found_display_substr_usernames
        );
    }
    info!("[Test] Displayname substring search completed successfully");
    
    // 测试 3.21: 子串匹配查询 - 组合查询（AND 组合）
    info!("[Test] Test 3.21: Substring search - combined with AND filter");
    let (rs_and_substr, _res_and_substr) = ldap
        .search(
            &base_dn,
            Scope::Subtree,
            "(&(sAMAccountName=test*)(mail=*@example.com))",
            vec!["sAMAccountName", "mail"],
        )
        .await
        .map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP search error: {e:?}"), "500-ldap-search-error"))?
        .success().map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP search error: {e:?}"), "500-ldap-search-error"))?;
    
    let entries_and_substr: Vec<SearchEntry> = rs_and_substr
        .into_iter()
        .map(SearchEntry::construct)
        .collect();
    
    assert!(entries_and_substr.len() >= 3, "AND combined substring search should find at least 3 test users, found {}", entries_and_substr.len());
    let mut found_and_substr_usernames: Vec<String> = entries_and_substr
        .iter()
        .filter_map(|entry| {
            entry.attrs.get("sAMAccountName")
                .and_then(|v| v.first()).cloned()
        })
        .collect();
    found_and_substr_usernames.sort();
    info!("[Test] AND combined substring search found usernames: {:?}", found_and_substr_usernames);
    for expected_username in &expected_usernames {
        assert!(
            found_and_substr_usernames.contains(&expected_username.to_string()),
            "AND combined substring search should find user: {}, found: {:?}",
            expected_username,
            found_and_substr_usernames
        );
    }
    info!("[Test] AND combined substring search completed successfully");
    
    // 测试 3.22: 子串匹配查询 - 组合查询（OR 组合）
    info!("[Test] Test 3.22: Substring search - combined with OR filter");
    let (rs_or_substr, _res_or_substr) = ldap
        .search(
            &base_dn,
            Scope::Subtree,
            "(|(sAMAccountName=*user1)(sAMAccountName=*user2))",
            vec!["sAMAccountName", "cn"],
        )
        .await
        .map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP search error: {e:?}"), "500-ldap-search-error"))?
        .success().map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP search error: {e:?}"), "500-ldap-search-error"))?;
    
    let entries_or_substr: Vec<SearchEntry> = rs_or_substr
        .into_iter()
        .map(SearchEntry::construct)
        .collect();
    
    assert!(entries_or_substr.len() >= 2, "OR combined substring search should find at least 2 test users, found {}", entries_or_substr.len());
    let mut found_or_substr_usernames: Vec<String> = entries_or_substr
        .iter()
        .filter_map(|entry| {
            entry.attrs.get("sAMAccountName")
                .and_then(|v| v.first()).cloned()
        })
        .collect();
    found_or_substr_usernames.sort();
    info!("[Test] OR combined substring search found usernames: {:?}", found_or_substr_usernames);
    assert!(
        found_or_substr_usernames.contains(&"testuser1".to_string()),
        "OR combined substring search should find testuser1, found: {:?}",
        found_or_substr_usernames
    );
    assert!(
        found_or_substr_usernames.contains(&"testuser2".to_string()),
        "OR combined substring search should find testuser2, found: {:?}",
        found_or_substr_usernames
    );
    info!("[Test] OR combined substring search completed successfully");
    
    // 测试 3.23: NOT 条件查询 - 排除特定用户（等值匹配）
    info!("[Test] Test 3.23: NOT condition - exclude specific user (equality match)");
    let (rs_not_eq, _res_not_eq) = ldap
        .search(
            &base_dn,
            Scope::Subtree,
            "(!(sAMAccountName=testuser1))",
            vec!["sAMAccountName", "cn"],
        )
        .await
        .map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP search error: {e:?}"), "500-ldap-search-error"))?
        .success().map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP search error: {e:?}"), "500-ldap-search-error"))?;
    
    let entries_not_eq: Vec<SearchEntry> = rs_not_eq
        .into_iter()
        .map(SearchEntry::construct)
        .collect();
    
    info!("[Test] NOT equality search found {} entries", entries_not_eq.len());
    let mut found_not_eq_usernames: Vec<String> = entries_not_eq
        .iter()
        .filter_map(|entry| {
            entry.attrs.get("sAMAccountName")
                .and_then(|v| v.first()).cloned()
        })
        .collect();
    found_not_eq_usernames.sort();
    info!("[Test] NOT equality search found usernames: {:?}", found_not_eq_usernames);
    // 验证 testuser1 不在结果中
    assert!(
        !found_not_eq_usernames.contains(&"testuser1".to_string()),
        "NOT equality search should NOT find testuser1, found: {:?}",
        found_not_eq_usernames
    );
    // 验证其他用户应该在结果中（如果有的话）
    if entries_not_eq.len() >= 2 {
        assert!(
            found_not_eq_usernames.contains(&"testuser2".to_string()) || found_not_eq_usernames.contains(&"testuser3".to_string()),
            "NOT equality search should find other test users, found: {:?}",
            found_not_eq_usernames
        );
    }
    info!("[Test] NOT equality search completed successfully");
    
    // 测试 3.24: NOT 条件查询 - 排除子串匹配
    info!("[Test] Test 3.24: NOT condition - exclude substring match");
    let (rs_not_substr, _res_not_substr) = ldap
        .search(
            &base_dn,
            Scope::Subtree,
            "(!(sAMAccountName=*user1))",
            vec!["sAMAccountName", "cn"],
        )
        .await
        .map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP search error: {e:?}"), "500-ldap-search-error"))?
        .success().map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP search error: {e:?}"), "500-ldap-search-error"))?;
    
    let entries_not_substr: Vec<SearchEntry> = rs_not_substr
        .into_iter()
        .map(SearchEntry::construct)
        .collect();
    
    info!("[Test] NOT substring search found {} entries", entries_not_substr.len());
    let mut found_not_substr_usernames: Vec<String> = entries_not_substr
        .iter()
        .filter_map(|entry| {
            entry.attrs.get("sAMAccountName")
                .and_then(|v| v.first()).cloned()
        })
        .collect();
    found_not_substr_usernames.sort();
    info!("[Test] NOT substring search found usernames: {:?}", found_not_substr_usernames);
    // 验证 testuser1 不在结果中
    assert!(
        !found_not_substr_usernames.contains(&"testuser1".to_string()),
        "NOT substring search should NOT find testuser1, found: {:?}",
        found_not_substr_usernames
    );
    info!("[Test] NOT substring search completed successfully");
    
    // 测试 3.25: NOT 条件查询 - 与 AND 组合
    info!("[Test] Test 3.25: NOT condition - combined with AND filter");
    let (rs_not_and, _res_not_and) = ldap
        .search(
            &base_dn,
            Scope::Subtree,
            "(&(sAMAccountName=test*)(!(sAMAccountName=testuser1)))",
            vec!["sAMAccountName", "cn"],
        )
        .await
        .map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP search error: {e:?}"), "500-ldap-search-error"))?
        .success().map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP search error: {e:?}"), "500-ldap-search-error"))?;
    
    let entries_not_and: Vec<SearchEntry> = rs_not_and
        .into_iter()
        .map(SearchEntry::construct)
        .collect();
    
    info!("[Test] NOT + AND search found {} entries", entries_not_and.len());
    let mut found_not_and_usernames: Vec<String> = entries_not_and
        .iter()
        .filter_map(|entry| {
            entry.attrs.get("sAMAccountName")
                .and_then(|v| v.first()).cloned()
        })
        .collect();
    found_not_and_usernames.sort();
    info!("[Test] NOT + AND search found usernames: {:?}", found_not_and_usernames);
    // 验证 testuser1 不在结果中
    assert!(
        !found_not_and_usernames.contains(&"testuser1".to_string()),
        "NOT + AND search should NOT find testuser1, found: {:?}",
        found_not_and_usernames
    );
    // 验证其他以 test 开头的用户应该在结果中
    if entries_not_and.len() >= 2 {
        assert!(
            found_not_and_usernames.contains(&"testuser2".to_string()) || found_not_and_usernames.contains(&"testuser3".to_string()),
            "NOT + AND search should find other test users, found: {:?}",
            found_not_and_usernames
        );
    }
    info!("[Test] NOT + AND search completed successfully");
    
    // 测试 3.26: NOT 条件查询 - 与 OR 组合
    info!("[Test] Test 3.26: NOT condition - combined with OR filter");
    let (rs_not_or, _res_not_or) = ldap
        .search(
            &base_dn,
            Scope::Subtree,
            "(|(!(sAMAccountName=testuser1))(!(sAMAccountName=testuser2)))",
            vec!["sAMAccountName", "cn"],
        )
        .await
        .map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP search error: {e:?}"), "500-ldap-search-error"))?
        .success().map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP search error: {e:?}"), "500-ldap-search-error"))?;
    
    let entries_not_or: Vec<SearchEntry> = rs_not_or
        .into_iter()
        .map(SearchEntry::construct)
        .collect();
    
    info!("[Test] NOT + OR search found {} entries", entries_not_or.len());
    let mut found_not_or_usernames: Vec<String> = entries_not_or
        .iter()
        .filter_map(|entry| {
            entry.attrs.get("sAMAccountName")
                .and_then(|v| v.first()).cloned()
        })
        .collect();
    found_not_or_usernames.sort();
    info!("[Test] NOT + OR search found usernames: {:?}", found_not_or_usernames);
    // OR 与 NOT 的组合：!(A) OR !(B) 意味着"不是A或不是B"，这应该返回所有用户（因为每个用户要么不是A要么不是B）
    // 但更合理的测试是：NOT (A OR B)，即排除A和B
    info!("[Test] NOT + OR search completed (note: this is a complex logical operation)");
    
    // 测试 3.27: NOT 条件查询 - 排除多个用户（NOT + OR）
    info!("[Test] Test 3.27: NOT condition - exclude multiple users (NOT of OR)");
    let (rs_not_or2, _res_not_or2) = ldap
        .search(
            &base_dn,
            Scope::Subtree,
            "(!(|(sAMAccountName=testuser1)(sAMAccountName=testuser2)))",
            vec!["sAMAccountName", "cn"],
        )
        .await
        .map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP search error: {e:?}"), "500-ldap-search-error"))?
        .success().map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP search error: {e:?}"), "500-ldap-search-error"))?;
    
    let entries_not_or2: Vec<SearchEntry> = rs_not_or2
        .into_iter()
        .map(SearchEntry::construct)
        .collect();
    
    info!("[Test] NOT(OR) search found {} entries", entries_not_or2.len());
    let mut found_not_or2_usernames: Vec<String> = entries_not_or2
        .iter()
        .filter_map(|entry| {
            entry.attrs.get("sAMAccountName")
                .and_then(|v| v.first()).cloned()
        })
        .collect();
    found_not_or2_usernames.sort();
    info!("[Test] NOT(OR) search found usernames: {:?}", found_not_or2_usernames);
    // 验证 testuser1 和 testuser2 都不在结果中
    assert!(
        !found_not_or2_usernames.contains(&"testuser1".to_string()),
        "NOT(OR) search should NOT find testuser1, found: {:?}",
        found_not_or2_usernames
    );
    assert!(
        !found_not_or2_usernames.contains(&"testuser2".to_string()),
        "NOT(OR) search should NOT find testuser2, found: {:?}",
        found_not_or2_usernames
    );
    // 验证 testuser3 应该在结果中（如果存在）
    if entries_not_or2.len() > 0 {
        assert!(
            found_not_or2_usernames.contains(&"testuser3".to_string()),
            "NOT(OR) search should find testuser3, found: {:?}",
            found_not_or2_usernames
        );
    }
    info!("[Test] NOT(OR) search completed successfully");
    
    // 测试 3.28: NOT 条件查询 - 排除特定邮件域名
    info!("[Test] Test 3.28: NOT condition - exclude specific mail domain");
    let (rs_not_mail, _res_not_mail) = ldap
        .search(
            &base_dn,
            Scope::Subtree,
            "(!(mail=*@example.com))",
            vec!["sAMAccountName", "mail"],
        )
        .await
        .map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP search error: {e:?}"), "500-ldap-search-error"))?
        .success().map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP search error: {e:?}"), "500-ldap-search-error"))?;
    
    let entries_not_mail: Vec<SearchEntry> = rs_not_mail
        .into_iter()
        .map(SearchEntry::construct)
        .collect();
    
    info!("[Test] NOT mail search found {} entries", entries_not_mail.len());
    let mut found_not_mail_usernames: Vec<String> = entries_not_mail
        .iter()
        .filter_map(|entry| {
            entry.attrs.get("sAMAccountName")
                .and_then(|v| v.first()).cloned()
        })
        .collect();
    found_not_mail_usernames.sort();
    info!("[Test] NOT mail search found usernames: {:?}", found_not_mail_usernames);
    // 验证所有测试用户都不在结果中（因为它们的邮件都是 @example.com）
    for expected_username in &expected_usernames {
        assert!(
            !found_not_mail_usernames.contains(&expected_username.to_string()),
            "NOT mail search should NOT find user with @example.com: {}, found: {:?}",
            expected_username,
            found_not_mail_usernames
        );
    }
    info!("[Test] NOT mail search completed successfully");
    
    // 测试 3.29: NOT 条件查询 - 嵌套 NOT（双重否定）
    info!("[Test] Test 3.29: NOT condition - nested NOT (double negation)");
    let (rs_not_not, _res_not_not) = ldap
        .search(
            &base_dn,
            Scope::Subtree,
            "(!(!(sAMAccountName=testuser1)))",
            vec!["sAMAccountName", "cn"],
        )
        .await
        .map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP search error: {e:?}"), "500-ldap-search-error"))?
        .success().map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP search error: {e:?}"), "500-ldap-search-error"))?;
    
    let entries_not_not: Vec<SearchEntry> = rs_not_not
        .into_iter()
        .map(SearchEntry::construct)
        .collect();
    
    info!("[Test] NOT(NOT) search found {} entries", entries_not_not.len());
    let mut found_not_not_usernames: Vec<String> = entries_not_not
        .iter()
        .filter_map(|entry| {
            entry.attrs.get("sAMAccountName")
                .and_then(|v| v.first()).cloned()
        })
        .collect();
    found_not_not_usernames.sort();
    info!("[Test] NOT(NOT) search found usernames: {:?}", found_not_not_usernames);
    // 双重否定应该等于原条件，所以应该找到 testuser1
    assert!(
        found_not_not_usernames.contains(&"testuser1".to_string()),
        "NOT(NOT) search should find testuser1 (double negation equals original), found: {:?}",
        found_not_not_usernames
    );
    info!("[Test] NOT(NOT) search completed successfully");
    
    // 测试 3.30: NOT 条件查询 - 复杂组合（AND + NOT + OR）
    info!("[Test] Test 3.30: NOT condition - complex combination (AND + NOT + OR)");
    let (rs_complex, _res_complex) = ldap
        .search(
            &base_dn,
            Scope::Subtree,
            "(&(sAMAccountName=test*)(!(|(sAMAccountName=testuser1)(sAMAccountName=testuser2))))",
            vec!["sAMAccountName", "cn"],
        )
        .await
        .map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP search error: {e:?}"), "500-ldap-search-error"))?
        .success().map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP search error: {e:?}"), "500-ldap-search-error"))?;
    
    let entries_complex: Vec<SearchEntry> = rs_complex
        .into_iter()
        .map(SearchEntry::construct)
        .collect();
    
    info!("[Test] Complex NOT search found {} entries", entries_complex.len());
    let mut found_complex_usernames: Vec<String> = entries_complex
        .iter()
        .filter_map(|entry| {
            entry.attrs.get("sAMAccountName")
                .and_then(|v| v.first()).cloned()
        })
        .collect();
    found_complex_usernames.sort();
    info!("[Test] Complex NOT search found usernames: {:?}", found_complex_usernames);
    // 验证 testuser1 和 testuser2 都不在结果中
    assert!(
        !found_complex_usernames.contains(&"testuser1".to_string()),
        "Complex NOT search should NOT find testuser1, found: {:?}",
        found_complex_usernames
    );
    assert!(
        !found_complex_usernames.contains(&"testuser2".to_string()),
        "Complex NOT search should NOT find testuser2, found: {:?}",
        found_complex_usernames
    );
    // 验证 testuser3 应该在结果中（如果存在）
    if entries_complex.len() > 0 {
        assert!(
            found_complex_usernames.contains(&"testuser3".to_string()),
            "Complex NOT search should find testuser3, found: {:?}",
            found_complex_usernames
        );
    }
    info!("[Test] Complex NOT search completed successfully");
    
    // 测试 3.31: Presence 匹配查询 - sAMAccountName 属性存在性检查
    info!("[Test] Test 3.31: Presence match - check if sAMAccountName attribute exists");
    let (rs_presence_sam, _res_presence_sam) = ldap
        .search(
            &base_dn,
            Scope::Subtree,
            "(sAMAccountName=*)",
            vec!["sAMAccountName", "cn"],
        )
        .await
        .map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP search error: {e:?}"), "500-ldap-search-error"))?
        .success().map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP search error: {e:?}"), "500-ldap-search-error"))?;
    
    let entries_presence_sam: Vec<SearchEntry> = rs_presence_sam
        .into_iter()
        .map(SearchEntry::construct)
        .collect();
    
    info!("[Test] Presence sAMAccountName search found {} entries", entries_presence_sam.len());
    assert!(entries_presence_sam.len() >= 3, "Presence sAMAccountName search should find at least 3 test users, found {}", entries_presence_sam.len());
    // 验证所有返回的条目都有 sAMAccountName 属性
    for entry in &entries_presence_sam {
        assert!(
            entry.attrs.contains_key("sAMAccountName"),
            "All entries should have sAMAccountName attribute"
        );
    }
    let mut found_presence_sam_usernames: Vec<String> = entries_presence_sam
        .iter()
        .filter_map(|entry| {
            entry.attrs.get("sAMAccountName")
                .and_then(|v| v.first()).cloned()
        })
        .collect();
    found_presence_sam_usernames.sort();
    info!("[Test] Presence sAMAccountName search found usernames: {:?}", found_presence_sam_usernames);
    for expected_username in &expected_usernames {
        assert!(
            found_presence_sam_usernames.contains(&expected_username.to_string()),
            "Presence sAMAccountName search should find user: {}, found: {:?}",
            expected_username,
            found_presence_sam_usernames
        );
    }
    info!("[Test] Presence sAMAccountName search completed successfully");
    
    // 测试 3.32: Presence 匹配查询 - mail 属性存在性检查
    info!("[Test] Test 3.32: Presence match - check if mail attribute exists");
    let (rs_presence_mail, _res_presence_mail) = ldap
        .search(
            &base_dn,
            Scope::Subtree,
            "(mail=*)",
            vec!["sAMAccountName", "mail"],
        )
        .await
        .map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP search error: {e:?}"), "500-ldap-search-error"))?
        .success().map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP search error: {e:?}"), "500-ldap-search-error"))?;
    
    let entries_presence_mail: Vec<SearchEntry> = rs_presence_mail
        .into_iter()
        .map(SearchEntry::construct)
        .collect();
    
    info!("[Test] Presence mail search found {} entries", entries_presence_mail.len());
    assert!(entries_presence_mail.len() >= 3, "Presence mail search should find at least 3 test users, found {}", entries_presence_mail.len());
    // 验证所有返回的条目都有 mail 属性
    for entry in &entries_presence_mail {
        assert!(
            entry.attrs.contains_key("mail"),
            "All entries should have mail attribute"
        );
    }
    let mut found_presence_mail_usernames: Vec<String> = entries_presence_mail
        .iter()
        .filter_map(|entry| {
            entry.attrs.get("sAMAccountName")
                .and_then(|v| v.first()).cloned()
        })
        .collect();
    found_presence_mail_usernames.sort();
    info!("[Test] Presence mail search found usernames: {:?}", found_presence_mail_usernames);
    for expected_username in &expected_usernames {
        assert!(
            found_presence_mail_usernames.contains(&expected_username.to_string()),
            "Presence mail search should find user: {}, found: {:?}",
            expected_username,
            found_presence_mail_usernames
        );
    }
    info!("[Test] Presence mail search completed successfully");
    
    // 测试 3.33: Presence 匹配查询 - cn 属性存在性检查
    info!("[Test] Test 3.33: Presence match - check if cn attribute exists");
    let (rs_presence_cn, _res_presence_cn) = ldap
        .search(
            &base_dn,
            Scope::Subtree,
            "(cn=*)",
            vec!["sAMAccountName", "cn"],
        )
        .await
        .map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP search error: {e:?}"), "500-ldap-search-error"))?
        .success().map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP search error: {e:?}"), "500-ldap-search-error"))?;
    
    let entries_presence_cn: Vec<SearchEntry> = rs_presence_cn
        .into_iter()
        .map(SearchEntry::construct)
        .collect();
    
    info!("[Test] Presence cn search found {} entries", entries_presence_cn.len());
    assert!(entries_presence_cn.len() >= 3, "Presence cn search should find at least 3 test users, found {}", entries_presence_cn.len());
    // 验证所有返回的条目都有 cn 属性
    for entry in &entries_presence_cn {
        assert!(
            entry.attrs.contains_key("cn"),
            "All entries should have cn attribute"
        );
    }
    info!("[Test] Presence cn search completed successfully");
    
    // 测试 3.34: Presence 匹配查询 - displayName 属性存在性检查
    info!("[Test] Test 3.34: Presence match - check if displayName attribute exists");
    let (rs_presence_display, _res_presence_display) = ldap
        .search(
            &base_dn,
            Scope::Subtree,
            "(displayName=*)",
            vec!["sAMAccountName", "displayName"],
        )
        .await
        .map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP search error: {e:?}"), "500-ldap-search-error"))?
        .success().map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP search error: {e:?}"), "500-ldap-search-error"))?;
    
    let entries_presence_display: Vec<SearchEntry> = rs_presence_display
        .into_iter()
        .map(SearchEntry::construct)
        .collect();
    
    info!("[Test] Presence displayName search found {} entries", entries_presence_display.len());
    assert!(entries_presence_display.len() >= 3, "Presence displayName search should find at least 3 test users, found {}", entries_presence_display.len());
    // 验证所有返回的条目都有 displayName 属性
    for entry in &entries_presence_display {
        assert!(
            entry.attrs.contains_key("displayName"),
            "All entries should have displayName attribute"
        );
    }
    info!("[Test] Presence displayName search completed successfully");
    
    // 测试 3.35: Presence 匹配查询 - employeeNumber 属性存在性检查
    info!("[Test] Test 3.35: Presence match - check if employeeNumber attribute exists");
    let (rs_presence_emp, _res_presence_emp) = ldap
        .search(
            &base_dn,
            Scope::Subtree,
            "(employeeNumber=*)",
            vec!["sAMAccountName", "employeeNumber"],
        )
        .await
        .map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP search error: {e:?}"), "500-ldap-search-error"))?
        .success().map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP search error: {e:?}"), "500-ldap-search-error"))?;
    
    let entries_presence_emp: Vec<SearchEntry> = rs_presence_emp
        .into_iter()
        .map(SearchEntry::construct)
        .collect();
    
    info!("[Test] Presence employeeNumber search found {} entries", entries_presence_emp.len());
    assert!(entries_presence_emp.len() >= 3, "Presence employeeNumber search should find at least 3 test users, found {}", entries_presence_emp.len());
    // 验证所有返回的条目都有 employeeNumber 属性
    for entry in &entries_presence_emp {
        assert!(
            entry.attrs.contains_key("employeeNumber"),
            "All entries should have employeeNumber attribute"
        );
    }
    info!("[Test] Presence employeeNumber search completed successfully");
    
    // 测试 3.36: Presence 匹配查询 - 与 AND 组合（多个属性都存在）
    info!("[Test] Test 3.36: Presence match - combined with AND (multiple attributes present)");
    let (rs_presence_and, _res_presence_and) = ldap
        .search(
            &base_dn,
            Scope::Subtree,
            "(&(sAMAccountName=*)(mail=*))",
            vec!["sAMAccountName", "mail", "cn"],
        )
        .await
        .map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP search error: {e:?}"), "500-ldap-search-error"))?
        .success().map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP search error: {e:?}"), "500-ldap-search-error"))?;
    
    let entries_presence_and: Vec<SearchEntry> = rs_presence_and
        .into_iter()
        .map(SearchEntry::construct)
        .collect();
    
    info!("[Test] Presence AND search found {} entries", entries_presence_and.len());
    assert!(entries_presence_and.len() >= 3, "Presence AND search should find at least 3 test users, found {}", entries_presence_and.len());
    // 验证所有返回的条目同时有 sAMAccountName 和 mail 属性
    for entry in &entries_presence_and {
        assert!(
            entry.attrs.contains_key("sAMAccountName") && entry.attrs.contains_key("mail"),
            "All entries should have both sAMAccountName and mail attributes"
        );
    }
    let mut found_presence_and_usernames: Vec<String> = entries_presence_and
        .iter()
        .filter_map(|entry| {
            entry.attrs.get("sAMAccountName")
                .and_then(|v| v.first()).cloned()
        })
        .collect();
    found_presence_and_usernames.sort();
    info!("[Test] Presence AND search found usernames: {:?}", found_presence_and_usernames);
    for expected_username in &expected_usernames {
        assert!(
            found_presence_and_usernames.contains(&expected_username.to_string()),
            "Presence AND search should find user: {}, found: {:?}",
            expected_username,
            found_presence_and_usernames
        );
    }
    info!("[Test] Presence AND search completed successfully");
    
    // 测试 3.37: Presence 匹配查询 - 与 OR 组合（任一属性存在）
    info!("[Test] Test 3.37: Presence match - combined with OR (any attribute present)");
    let (rs_presence_or, _res_presence_or) = ldap
        .search(
            &base_dn,
            Scope::Subtree,
            "(|(sAMAccountName=*)(mail=*))",
            vec!["sAMAccountName", "mail", "cn"],
        )
        .await
        .map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP search error: {e:?}"), "500-ldap-search-error"))?
        .success().map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP search error: {e:?}"), "500-ldap-search-error"))?;
    
    let entries_presence_or: Vec<SearchEntry> = rs_presence_or
        .into_iter()
        .map(SearchEntry::construct)
        .collect();
    
    info!("[Test] Presence OR search found {} entries", entries_presence_or.len());
    assert!(entries_presence_or.len() >= 3, "Presence OR search should find at least 3 test users, found {}", entries_presence_or.len());
    // 验证所有返回的条目至少有 sAMAccountName 或 mail 属性之一
    for entry in &entries_presence_or {
        assert!(
            entry.attrs.contains_key("sAMAccountName") || entry.attrs.contains_key("mail"),
            "All entries should have at least sAMAccountName or mail attribute"
        );
    }
    let mut found_presence_or_usernames: Vec<String> = entries_presence_or
        .iter()
        .filter_map(|entry| {
            entry.attrs.get("sAMAccountName")
                .and_then(|v| v.first()).cloned()
        })
        .collect();
    found_presence_or_usernames.sort();
    info!("[Test] Presence OR search found usernames: {:?}", found_presence_or_usernames);
    for expected_username in &expected_usernames {
        assert!(
            found_presence_or_usernames.contains(&expected_username.to_string()),
            "Presence OR search should find user: {}, found: {:?}",
            expected_username,
            found_presence_or_usernames
        );
    }
    info!("[Test] Presence OR search completed successfully");
    
    // 测试 3.39: Presence 匹配查询 - 复杂组合（AND + OR + Presence）
    info!("[Test] Test 3.39: Presence match - complex combination (AND + OR + Presence)");
    let (rs_presence_complex, _res_presence_complex) = ldap
        .search(
            &base_dn,
            Scope::Subtree,
            "(&(sAMAccountName=test*)(|(mail=*)(displayName=*)))",
            vec!["sAMAccountName", "mail", "displayName"],
        )
        .await
        .map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP search error: {e:?}"), "500-ldap-search-error"))?
        .success().map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP search error: {e:?}"), "500-ldap-search-error"))?;
    
    let entries_presence_complex: Vec<SearchEntry> = rs_presence_complex
        .into_iter()
        .map(SearchEntry::construct)
        .collect();
    
    info!("[Test] Complex presence search found {} entries", entries_presence_complex.len());
    assert!(entries_presence_complex.len() >= 3, "Complex presence search should find at least 3 test users, found {}", entries_presence_complex.len());
    // 验证所有返回的条目都有 sAMAccountName（以 test 开头）并且有 mail 或 displayName
    for entry in &entries_presence_complex {
        assert!(
            entry.attrs.contains_key("sAMAccountName"),
            "All entries should have sAMAccountName attribute"
        );
        assert!(
            entry.attrs.contains_key("mail") || entry.attrs.contains_key("displayName"),
            "All entries should have mail or displayName attribute"
        );
    }
    let mut found_presence_complex_usernames: Vec<String> = entries_presence_complex
        .iter()
        .filter_map(|entry| {
            entry.attrs.get("sAMAccountName")
                .and_then(|v| v.first()).cloned()
        })
        .collect();
    found_presence_complex_usernames.sort();
    info!("[Test] Complex presence search found usernames: {:?}", found_presence_complex_usernames);
    for expected_username in &expected_usernames {
        assert!(
            found_presence_complex_usernames.contains(&expected_username.to_string()),
            "Complex presence search should find user: {}, found: {:?}",
            expected_username,
            found_presence_complex_usernames
        );
    }
    info!("[Test] Complex presence search completed successfully");
    
    // 测试 3.40: Presence 匹配查询 - 多个属性的 AND 组合
    info!("[Test] Test 3.40: Presence match - multiple attributes AND combination");
    let (rs_presence_multi, _res_presence_multi) = ldap
        .search(
            &base_dn,
            Scope::Subtree,
            "(&(sAMAccountName=*)(cn=*)(mail=*)(displayName=*))",
            vec!["sAMAccountName", "cn", "mail", "displayName"],
        )
        .await
        .map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP search error: {e:?}"), "500-ldap-search-error"))?
        .success().map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP search error: {e:?}"), "500-ldap-search-error"))?;
    
    let entries_presence_multi: Vec<SearchEntry> = rs_presence_multi
        .into_iter()
        .map(SearchEntry::construct)
        .collect();
    
    info!("[Test] Multiple presence AND search found {} entries", entries_presence_multi.len());
    assert!(entries_presence_multi.len() >= 3, "Multiple presence AND search should find at least 3 test users, found {}", entries_presence_multi.len());
    // 验证所有返回的条目同时有所有四个属性
    for entry in &entries_presence_multi {
        assert!(
            entry.attrs.contains_key("sAMAccountName") && 
            entry.attrs.contains_key("cn") && 
            entry.attrs.contains_key("mail") && 
            entry.attrs.contains_key("displayName"),
            "All entries should have all four attributes: sAMAccountName, cn, mail, displayName"
        );
    }
    let mut found_presence_multi_usernames: Vec<String> = entries_presence_multi
        .iter()
        .filter_map(|entry| {
            entry.attrs.get("sAMAccountName")
                .and_then(|v| v.first()).cloned()
        })
        .collect();
    found_presence_multi_usernames.sort();
    info!("[Test] Multiple presence AND search found usernames: {:?}", found_presence_multi_usernames);
    for expected_username in &expected_usernames {
        assert!(
            found_presence_multi_usernames.contains(&expected_username.to_string()),
            "Multiple presence AND search should find user: {}, found: {:?}",
            expected_username,
            found_presence_multi_usernames
        );
    }
    info!("[Test] Multiple presence AND search completed successfully");
    
    // 测试 4: 使用 Present 过滤器搜索根 DSE（空 base + objectClass）
    // 注意：LDAP 服务器实现只支持空 base 的 objectClass 搜索，不支持在 DC 级别搜索所有用户
    info!("[Test] Test 4: Searching root DSE with empty base and objectClass filter");
    let (rs2, _res2) = ldap
        .search(
            "",  // 空 base 用于搜索根 DSE
            Scope::Base,  // Base scope 用于根 DSE
            "(objectClass=*)",
            vec!["*"],  // 请求所有属性
        )
        .await
        .map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP search error: {e:?}"), "500-ldap-search-error"))?
        .success().map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP search error: {e:?}"), "500-ldap-search-error"))?;
    
    let entries2: Vec<SearchEntry> = rs2
        .into_iter()
        .map(SearchEntry::construct)
        .collect();
    
    info!("[Test] Found {} entries in root DSE", entries2.len());
    // 根 DSE 应该至少返回 DC 条目
    assert!(!entries2.is_empty(), "Root DSE search should return at least one entry");
    
    let root_dse_entry = &entries2[0];
    info!("[Test] Root DSE DN: {}", root_dse_entry.dn);
    info!("[Test] Root DSE attributes: {:?}", root_dse_entry.attrs.keys().collect::<Vec<_>>());
    
    // 验证 RootDSE 包含必要的属性
    // 1. namingContexts - 命名上下文（base DN）
    assert!(
        root_dse_entry.attrs.contains_key("namingContexts"),
        "RootDSE should contain namingContexts attribute"
    );
    let naming_contexts = root_dse_entry.attrs.get("namingContexts").unwrap();
    assert!(!naming_contexts.is_empty(), "namingContexts should not be empty");
    assert_eq!(
        naming_contexts[0], base_dn,
        "namingContexts should match base DN: {}, got: {}",
        base_dn, naming_contexts[0]
    );
    info!("[Test] namingContexts: {:?}", naming_contexts);
    
    // 2. subschemaSubentry - Schema 子条目位置（Apache Directory Studio 需要此属性）
    assert!(
        root_dse_entry.attrs.contains_key("subschemaSubentry"),
        "RootDSE should contain subschemaSubentry attribute (required by Apache Directory Studio)"
    );
    let subschema_subentry = root_dse_entry.attrs.get("subschemaSubentry").unwrap();
    assert!(!subschema_subentry.is_empty(), "subschemaSubentry should not be empty");
    assert!(
        subschema_subentry[0].contains("cn=Subschema"),
        "subschemaSubentry should contain 'cn=Subschema', got: {}",
        subschema_subentry[0]
    );
    info!("[Test] subschemaSubentry: {:?}", subschema_subentry);
    
    // 3. supportedLDAPVersion - 支持的 LDAP 版本
    assert!(
        root_dse_entry.attrs.contains_key("supportedLDAPVersion"),
        "RootDSE should contain supportedLDAPVersion attribute"
    );
    let supported_version = root_dse_entry.attrs.get("supportedLDAPVersion").unwrap();
    assert!(!supported_version.is_empty(), "supportedLDAPVersion should not be empty");
    assert!(
        supported_version.contains(&"3".to_string()),
        "supportedLDAPVersion should include version 3, got: {:?}",
        supported_version
    );
    info!("[Test] supportedLDAPVersion: {:?}", supported_version);
    
    // 4. vendorName - 供应商名称
    assert!(
        root_dse_entry.attrs.contains_key("vendorName"),
        "RootDSE should contain vendorName attribute"
    );
    let vendor_name = root_dse_entry.attrs.get("vendorName").unwrap();
    assert!(!vendor_name.is_empty(), "vendorName should not be empty");
    info!("[Test] vendorName: {:?}", vendor_name);
    
    // 5. vendorVersion - 供应商版本
    assert!(
        root_dse_entry.attrs.contains_key("vendorVersion"),
        "RootDSE should contain vendorVersion attribute"
    );
    let vendor_version = root_dse_entry.attrs.get("vendorVersion").unwrap();
    assert!(!vendor_version.is_empty(), "vendorVersion should not be empty");
    info!("[Test] vendorVersion: {:?}", vendor_version);
    
    // 6. supportedSASLMechanisms - 支持的 SASL 机制
    assert!(
        root_dse_entry.attrs.contains_key("supportedSASLMechanisms"),
        "RootDSE should contain supportedSASLMechanisms attribute"
    );
    let sasl_mechanisms = root_dse_entry.attrs.get("supportedSASLMechanisms").unwrap();
    assert!(!sasl_mechanisms.is_empty(), "supportedSASLMechanisms should not be empty");
    info!("[Test] supportedSASLMechanisms: {:?}", sasl_mechanisms);
    
    info!("[Test] RootDSE test completed successfully - all required attributes present");
    
    // 测试 4.1: 测试 RootDSE 特定属性查询（只请求 subschemaSubentry）
    info!("[Test] Test 4.1: Querying RootDSE with specific attribute (subschemaSubentry)");
    let (rs_subschema, _res_subschema) = ldap
        .search(
            "",
            Scope::Base,
            "(objectClass=*)",
            vec!["subschemaSubentry"],
        )
        .await
        .map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP search error: {e:?}"), "500-ldap-search-error"))?
        .success().map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP search error: {e:?}"), "500-ldap-search-error"))?;
    
    let entries_subschema: Vec<SearchEntry> = rs_subschema
        .into_iter()
        .map(SearchEntry::construct)
        .collect();
    
    assert!(!entries_subschema.is_empty(), "RootDSE search should return at least one entry");
    let subschema_entry = &entries_subschema[0];
    
    // 验证只返回了请求的属性
    assert!(
        subschema_entry.attrs.contains_key("subschemaSubentry"),
        "RootDSE should contain subschemaSubentry when requested"
    );
    assert_eq!(
        subschema_entry.attrs.len(),
        1,
        "RootDSE should only return requested attribute, got: {:?}",
        subschema_entry.attrs.keys().collect::<Vec<_>>()
    );
    info!("[Test] RootDSE subschemaSubentry query test passed");
    
    // 测试 4.2: 测试 RootDSE 多个属性查询
    info!("[Test] Test 4.2: Querying RootDSE with multiple specific attributes");
    let (rs_multi_attr, _res_multi_attr) = ldap
        .search(
            "",
            Scope::Base,
            "(objectClass=*)",
            vec!["namingContexts", "subschemaSubentry", "supportedLDAPVersion"],
        )
        .await
        .map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP search error: {e:?}"), "500-ldap-search-error"))?
        .success().map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP search error: {e:?}"), "500-ldap-search-error"))?;
    
    let entries_multi_attr: Vec<SearchEntry> = rs_multi_attr
        .into_iter()
        .map(SearchEntry::construct)
        .collect();
    
    assert!(!entries_multi_attr.is_empty(), "RootDSE search should return at least one entry");
    let multi_attr_entry = &entries_multi_attr[0];
    
    // 验证返回了所有请求的属性
    assert!(
        multi_attr_entry.attrs.contains_key("namingContexts"),
        "RootDSE should contain namingContexts"
    );
    assert!(
        multi_attr_entry.attrs.contains_key("subschemaSubentry"),
        "RootDSE should contain subschemaSubentry"
    );
    assert!(
        multi_attr_entry.attrs.contains_key("supportedLDAPVersion"),
        "RootDSE should contain supportedLDAPVersion"
    );
    assert_eq!(
        multi_attr_entry.attrs.len(),
        3,
        "RootDSE should return exactly 3 requested attributes, got: {:?}",
        multi_attr_entry.attrs.keys().collect::<Vec<_>>()
    );
    info!("[Test] RootDSE multiple attributes query test passed");
    
    // 测试 4.3: Schema 查询（Apache Directory Studio 连接时需要的查询）
    info!("[Test] Test 4.3: Querying LDAP Schema (subschema entry)");
    let schema_dn = "cn=Subschema".to_string();
    info!("[Test] Schema DN: {}", schema_dn);
    
    // 测试 4.3.1: 基本 Schema 查询 - 查询所有 schema 属性
    info!("[Test] Test 4.3.1: Basic schema query with all attributes");
    let (rs_schema, _res_schema) = ldap
        .search(
            &schema_dn,
            Scope::Base,
            "(objectClass=subschema)",
            vec!["*"],  // 请求所有属性
        )
        .await
        .map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP schema search error: {e:?}"), "500-ldap-schema-search-error"))?
        .success().map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP schema search error: {e:?}"), "500-ldap-schema-search-error"))?;
    
    let entries_schema: Vec<SearchEntry> = rs_schema
        .into_iter()
        .map(SearchEntry::construct)
        .collect();
    
    info!("[Test] Found {} entries in schema query", entries_schema.len());
    assert_eq!(entries_schema.len(), 1, "Schema query should return exactly one entry");
    
    let schema_entry = &entries_schema[0];
    info!("[Test] Schema entry DN: {}", schema_entry.dn);
    assert_eq!(schema_entry.dn.to_lowercase(), schema_dn.to_lowercase(), "Schema entry DN should match schema DN");
    
    info!("[Test] Schema entry attributes: {:?}", schema_entry.attrs.keys().collect::<Vec<_>>());
    
    // 验证 Schema 条目包含必要的属性
    // 1. objectClass - 应该包含 subschema
    assert!(
        schema_entry.attrs.contains_key("objectClass"),
        "Schema entry should contain objectClass attribute"
    );
    let object_classes = schema_entry.attrs.get("objectClass").unwrap();
    assert!(
        object_classes.iter().any(|oc| oc.to_lowercase() == "subschema"),
        "Schema entry objectClass should contain 'subschema', got: {:?}",
        object_classes
    );
    info!("[Test] Schema objectClass: {:?}", object_classes);
    
    // 2. objectClasses - 对象类定义
    assert!(
        schema_entry.attrs.contains_key("objectClasses"),
        "Schema entry should contain objectClasses attribute"
    );
    let object_classes_def = schema_entry.attrs.get("objectClasses").unwrap();
    assert!(!object_classes_def.is_empty(), "objectClasses should not be empty");
    info!("[Test] Found {} object class definitions", object_classes_def.len());
    
    // 验证一些常见的对象类是否存在
    let object_classes_str = object_classes_def.join(" ");
    assert!(
        object_classes_str.to_lowercase().contains("person"),
        "objectClasses should contain 'person' definition"
    );
    assert!(
        object_classes_str.to_lowercase().contains("inetorgperson"),
        "objectClasses should contain 'inetOrgPerson' definition"
    );
    info!("[Test] objectClasses contains expected definitions");
    
    // 3. attributeTypes - 属性类型定义
    assert!(
        schema_entry.attrs.contains_key("attributeTypes"),
        "Schema entry should contain attributeTypes attribute"
    );
    let attribute_types = schema_entry.attrs.get("attributeTypes").unwrap();
    assert!(!attribute_types.is_empty(), "attributeTypes should not be empty");
    info!("[Test] Found {} attribute type definitions", attribute_types.len());
    
    // 验证一些常见的属性类型是否存在
    let attribute_types_str = attribute_types.join(" ");
    assert!(
        attribute_types_str.to_lowercase().contains("cn") || attribute_types_str.to_lowercase().contains("commonname"),
        "attributeTypes should contain 'cn' definition"
    );
    assert!(
        attribute_types_str.to_lowercase().contains("mail") || attribute_types_str.to_lowercase().contains("rfc822mailbox"),
        "attributeTypes should contain 'mail' definition"
    );
    info!("[Test] attributeTypes contains expected definitions");
    
    // 4. ldapSyntaxes - LDAP 语法定义
    assert!(
        schema_entry.attrs.contains_key("ldapSyntaxes"),
        "Schema entry should contain ldapSyntaxes attribute"
    );
    let ldap_syntaxes = schema_entry.attrs.get("ldapSyntaxes").unwrap();
    assert!(!ldap_syntaxes.is_empty(), "ldapSyntaxes should not be empty");
    info!("[Test] Found {} LDAP syntax definitions", ldap_syntaxes.len());
    
    // 5. matchingRules - 匹配规则定义
    assert!(
        schema_entry.attrs.contains_key("matchingRules"),
        "Schema entry should contain matchingRules attribute"
    );
    let matching_rules = schema_entry.attrs.get("matchingRules").unwrap();
    assert!(!matching_rules.is_empty(), "matchingRules should not be empty");
    info!("[Test] Found {} matching rule definitions", matching_rules.len());
    
    // 6. matchingRuleUse - 匹配规则使用定义
    assert!(
        schema_entry.attrs.contains_key("matchingRuleUse"),
        "Schema entry should contain matchingRuleUse attribute"
    );
    let matching_rule_use = schema_entry.attrs.get("matchingRuleUse").unwrap();
    assert!(!matching_rule_use.is_empty(), "matchingRuleUse should not be empty");
    info!("[Test] Found {} matching rule use definitions", matching_rule_use.len());
    
    // 7. createTimestamp 和 modifyTimestamp - 时间戳
    assert!(
        schema_entry.attrs.contains_key("createTimestamp"),
        "Schema entry should contain createTimestamp attribute"
    );
    let create_timestamp = schema_entry.attrs.get("createTimestamp").unwrap();
    assert_eq!(create_timestamp.len(), 1, "createTimestamp should have exactly one value");
    assert!(
        create_timestamp[0].ends_with('Z'),
        "createTimestamp should end with 'Z' (UTC), got: {}",
        create_timestamp[0]
    );
    info!("[Test] createTimestamp: {}", create_timestamp[0]);
    
    assert!(
        schema_entry.attrs.contains_key("modifyTimestamp"),
        "Schema entry should contain modifyTimestamp attribute"
    );
    let modify_timestamp = schema_entry.attrs.get("modifyTimestamp").unwrap();
    assert_eq!(modify_timestamp.len(), 1, "modifyTimestamp should have exactly one value");
    assert!(
        modify_timestamp[0].ends_with('Z'),
        "modifyTimestamp should end with 'Z' (UTC), got: {}",
        modify_timestamp[0]
    );
    info!("[Test] modifyTimestamp: {}", modify_timestamp[0]);
    
    info!("[Test] Basic schema query test completed successfully");
    
    // 测试 4.3.2: Schema 查询 - 只请求特定属性（Apache Directory Studio 的标准查询）
    info!("[Test] Test 4.3.2: Schema query with specific attributes (Apache Directory Studio style)");
    let (rs_schema_specific, _res_schema_specific) = ldap
        .search(
            &schema_dn,
            Scope::Base,
            "(objectClass=subschema)",
            vec!["objectClasses", "attributeTypes", "ldapSyntaxes", "matchingRules", "matchingRuleUse", "createTimestamp", "modifyTimestamp"],
        )
        .await
        .map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP schema search error: {e:?}"), "500-ldap-schema-search-error"))?
        .success().map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP schema search error: {e:?}"), "500-ldap-schema-search-error"))?;
    
    let entries_schema_specific: Vec<SearchEntry> = rs_schema_specific
        .into_iter()
        .map(SearchEntry::construct)
        .collect();
    
    assert_eq!(entries_schema_specific.len(), 1, "Schema query should return exactly one entry");
    let schema_entry_specific = &entries_schema_specific[0];
    
    // 验证返回的属性是请求的属性
    let requested_attrs = vec!["objectClasses", "attributeTypes", "ldapSyntaxes", "matchingRules", "matchingRuleUse", "createTimestamp", "modifyTimestamp"];
    for attr in &requested_attrs {
        assert!(
            schema_entry_specific.attrs.contains_key(*attr),
            "Schema entry should contain requested attribute: {}",
            attr
        );
    }
    
    // 验证返回的属性数量（应该包含请求的属性，可能还有 objectClass）
    assert!(
        schema_entry_specific.attrs.len() >= requested_attrs.len(),
        "Schema entry should contain at least {} attributes (requested), got: {}",
        requested_attrs.len(),
        schema_entry_specific.attrs.len()
    );
    
    info!("[Test] Schema query with specific attributes test passed");
    
    // 测试 4.3.3: Schema 查询 - 只请求 objectClasses
    info!("[Test] Test 4.3.3: Schema query requesting only objectClasses");
    let (rs_schema_obj, _res_schema_obj) = ldap
        .search(
            &schema_dn,
            Scope::Base,
            "(objectClass=subschema)",
            vec!["objectClasses"],
        )
        .await
        .map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP schema search error: {e:?}"), "500-ldap-schema-search-error"))?
        .success().map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP schema search error: {e:?}"), "500-ldap-schema-search-error"))?;
    
    let entries_schema_obj: Vec<SearchEntry> = rs_schema_obj
        .into_iter()
        .map(SearchEntry::construct)
        .collect();
    
    assert_eq!(entries_schema_obj.len(), 1, "Schema query should return exactly one entry");
    let schema_entry_obj = &entries_schema_obj[0];
    
    assert!(
        schema_entry_obj.attrs.contains_key("objectClasses"),
        "Schema entry should contain objectClasses when requested"
    );
    assert!(
        schema_entry_obj.attrs.len() <= 2,  // objectClasses + 可能的 objectClass
        "Schema entry should return only requested attribute (and possibly objectClass), got: {:?}",
        schema_entry_obj.attrs.keys().collect::<Vec<_>>()
    );
    
    info!("[Test] Schema query with only objectClasses test passed");
    
    // 测试 4.3.4: Schema 查询 - 只请求 attributeTypes
    info!("[Test] Test 4.3.4: Schema query requesting only attributeTypes");
    let (rs_schema_attr, _res_schema_attr) = ldap
        .search(
            &schema_dn,
            Scope::Base,
            "(objectClass=subschema)",
            vec!["attributeTypes"],
        )
        .await
        .map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP schema search error: {e:?}"), "500-ldap-schema-search-error"))?
        .success().map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP schema search error: {e:?}"), "500-ldap-schema-search-error"))?;
    
    let entries_schema_attr: Vec<SearchEntry> = rs_schema_attr
        .into_iter()
        .map(SearchEntry::construct)
        .collect();
    
    assert_eq!(entries_schema_attr.len(), 1, "Schema query should return exactly one entry");
    let schema_entry_attr = &entries_schema_attr[0];
    
    assert!(
        schema_entry_attr.attrs.contains_key("attributeTypes"),
        "Schema entry should contain attributeTypes when requested"
    );
    
    info!("[Test] Schema query with only attributeTypes test passed");
    
    info!("[Test] All schema query tests completed successfully");
    
    // 测试 4.5: 使用 objectClass=* 在 base_dn 下进行全量查询
    info!("[Test] Test 4.5: Full query with objectClass=* in base DN (should return all entries)");
    let (rs_full, _res_full) = ldap
        .search(
            &base_dn,
            Scope::Subtree,
            "(objectClass=*)",
            vec!["sAMAccountName", "cn", "objectClass"],
        )
        .await
        .map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP search error: {e:?}"), "500-ldap-search-error"))?
        .success().map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP search error: {e:?}"), "500-ldap-search-error"))?;
    
    let entries_full: Vec<SearchEntry> = rs_full
        .into_iter()
        .map(SearchEntry::construct)
        .collect();
    
    info!("[Test] Found {} entries in full query", entries_full.len());
    // 全量查询应该至少返回所有测试用户（3个）加上可能的其他条目（如 DC 条目）
    assert!(entries_full.len() >= 3, "Full query should return at least 3 test users, found {}", entries_full.len());
    
    // 验证所有测试用户都在全量查询结果中
    let mut found_usernames_full: Vec<String> = entries_full
        .iter()
        .filter_map(|entry| {
            entry.attrs.get("sAMAccountName")
                .and_then(|v| v.first()).cloned()
        })
        .collect();
    
    found_usernames_full.sort();
    info!("[Test] Found usernames in full query: {:?}", found_usernames_full);
    
    // 验证所有三个测试用户都被找到
    let expected_usernames_full: Vec<&str> = vec!["testuser1", "testuser2", "testuser3"];
    for expected_username in &expected_usernames_full {
        assert!(
            found_usernames_full.contains(&expected_username.to_string()),
            "Full query should find user: {}, found: {:?}",
            expected_username,
            found_usernames_full
        );
    }
    info!("[Test] Full query found all test users successfully");
    
    // 测试 4.5.1: objectClass 相等查询 - 使用固定列表中的值（inetOrgPerson）
    info!("[Test] Test 4.5.1: objectClass equality query - using inetOrgPerson (should return all users)");
    let (rs_obj_eq1, _res_obj_eq1) = ldap
        .search(
            &base_dn,
            Scope::Subtree,
            "(objectClass=inetOrgPerson)",
            vec!["sAMAccountName", "cn", "objectClass"],
        )
        .await
        .map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP search error: {e:?}"), "500-ldap-search-error"))?
        .success().map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP search error: {e:?}"), "500-ldap-search-error"))?;
    
    let entries_obj_eq1: Vec<SearchEntry> = rs_obj_eq1
        .into_iter()
        .map(SearchEntry::construct)
        .collect();
    
    info!("[Test] objectClass=inetOrgPerson found {} entries", entries_obj_eq1.len());
    assert!(entries_obj_eq1.len() >= 3, "objectClass=inetOrgPerson should return at least 3 test users, found {}", entries_obj_eq1.len());
    
    // 验证所有测试用户都在结果中
    let mut found_obj_eq1_usernames: Vec<String> = entries_obj_eq1
        .iter()
        .filter_map(|entry| {
            entry.attrs.get("sAMAccountName")
                .and_then(|v| v.first()).cloned()
        })
        .collect();
    found_obj_eq1_usernames.sort();
    info!("[Test] objectClass=inetOrgPerson found usernames: {:?}", found_obj_eq1_usernames);
    for expected_username in &expected_usernames_full {
        assert!(
            found_obj_eq1_usernames.contains(&expected_username.to_string()),
            "objectClass=inetOrgPerson should find user: {}, found: {:?}",
            expected_username,
            found_obj_eq1_usernames
        );
    }
    info!("[Test] objectClass=inetOrgPerson query completed successfully");
    
    // 测试 4.5.2: objectClass 相等查询 - 使用固定列表中的另一个值（uidObject）
    info!("[Test] Test 4.5.2: objectClass equality query - using uidObject (should return all users)");
    let (rs_obj_eq2, _res_obj_eq2) = ldap
        .search(
            &base_dn,
            Scope::Subtree,
            "(objectClass=uidObject)",
            vec!["sAMAccountName", "cn", "objectClass"],
        )
        .await
        .map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP search error: {e:?}"), "500-ldap-search-error"))?
        .success().map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP search error: {e:?}"), "500-ldap-search-error"))?;
    
    let entries_obj_eq2: Vec<SearchEntry> = rs_obj_eq2
        .into_iter()
        .map(SearchEntry::construct)
        .collect();
    
    info!("[Test] objectClass=uidObject found {} entries", entries_obj_eq2.len());
    assert!(entries_obj_eq2.len() >= 3, "objectClass=uidObject should return at least 3 test users, found {}", entries_obj_eq2.len());
    info!("[Test] objectClass=uidObject query completed successfully");
    
    // 测试 4.5.3: objectClass 相等查询 - 使用固定列表中的另一个值（top）
    info!("[Test] Test 4.5.3: objectClass equality query - using top (should return all users)");
    let (rs_obj_eq3, _res_obj_eq3) = ldap
        .search(
            &base_dn,
            Scope::Subtree,
            "(objectClass=top)",
            vec!["sAMAccountName", "cn", "objectClass"],
        )
        .await
        .map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP search error: {e:?}"), "500-ldap-search-error"))?
        .success().map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP search error: {e:?}"), "500-ldap-search-error"))?;
    
    let entries_obj_eq3: Vec<SearchEntry> = rs_obj_eq3
        .into_iter()
        .map(SearchEntry::construct)
        .collect();
    
    info!("[Test] objectClass=top found {} entries", entries_obj_eq3.len());
    assert!(entries_obj_eq3.len() >= 3, "objectClass=top should return at least 3 test users, found {}", entries_obj_eq3.len());
    info!("[Test] objectClass=top query completed successfully");
    
    // 测试 4.5.4: objectClass 相等查询 - 使用不在固定列表中的值（应该返回空结果）
    info!("[Test] Test 4.5.4: objectClass equality query - using value not in fixed list (should return empty)");
    let (rs_obj_eq4, _res_obj_eq4) = ldap
        .search(
            &base_dn,
            Scope::Subtree,
            "(objectClass=person)",
            vec!["sAMAccountName", "cn", "objectClass"],
        )
        .await
        .map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP search error: {e:?}"), "500-ldap-search-error"))?
        .success().map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP search error: {e:?}"), "500-ldap-search-error"))?;
    
    let entries_obj_eq4: Vec<SearchEntry> = rs_obj_eq4
        .into_iter()
        .map(SearchEntry::construct)
        .collect();
    
    info!("[Test] objectClass=person found {} entries", entries_obj_eq4.len());
    assert!(entries_obj_eq4.is_empty(), "objectClass=person should return empty result (not in fixed list), found {}", entries_obj_eq4.len());
    info!("[Test] objectClass=person query correctly returned empty result");
    
    // 测试 4.5.5: objectClass 子串匹配查询 - 前缀匹配（initial）
    info!("[Test] Test 4.5.5: objectClass substring query - prefix match (initial)");
    let (rs_obj_sub1, _res_obj_sub1) = ldap
        .search(
            &base_dn,
            Scope::Subtree,
            "(objectClass=inet*)",
            vec!["sAMAccountName", "cn", "objectClass"],
        )
        .await
        .map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP search error: {e:?}"), "500-ldap-search-error"))?
        .success().map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP search error: {e:?}"), "500-ldap-search-error"))?;
    
    let entries_obj_sub1: Vec<SearchEntry> = rs_obj_sub1
        .into_iter()
        .map(SearchEntry::construct)
        .collect();
    
    info!("[Test] objectClass=inet* found {} entries", entries_obj_sub1.len());
    assert!(entries_obj_sub1.len() >= 3, "objectClass=inet* should return at least 3 test users, found {}", entries_obj_sub1.len());
    info!("[Test] objectClass=inet* query completed successfully");
    
    // 测试 4.5.6: objectClass 子串匹配查询 - 后缀匹配（final）
    info!("[Test] Test 4.5.6: objectClass substring query - suffix match (final)");
    let (rs_obj_sub2, _res_obj_sub2) = ldap
        .search(
            &base_dn,
            Scope::Subtree,
            "(objectClass=*Person)",
            vec!["sAMAccountName", "cn", "objectClass"],
        )
        .await
        .map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP search error: {e:?}"), "500-ldap-search-error"))?
        .success().map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP search error: {e:?}"), "500-ldap-search-error"))?;
    
    let entries_obj_sub2: Vec<SearchEntry> = rs_obj_sub2
        .into_iter()
        .map(SearchEntry::construct)
        .collect();
    
    info!("[Test] objectClass=*Person found {} entries", entries_obj_sub2.len());
    assert!(entries_obj_sub2.len() >= 3, "objectClass=*Person should return at least 3 test users, found {}", entries_obj_sub2.len());
    info!("[Test] objectClass=*Person query completed successfully");
    
    // 测试 4.5.7: objectClass 子串匹配查询 - 任意位置匹配（any）
    info!("[Test] Test 4.5.7: objectClass substring query - any position match (any)");
    let (rs_obj_sub3, _res_obj_sub3) = ldap
        .search(
            &base_dn,
            Scope::Subtree,
            "(objectClass=*Object*)",
            vec!["sAMAccountName", "cn", "objectClass"],
        )
        .await
        .map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP search error: {e:?}"), "500-ldap-search-error"))?
        .success().map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP search error: {e:?}"), "500-ldap-search-error"))?;
    
    let entries_obj_sub3: Vec<SearchEntry> = rs_obj_sub3
        .into_iter()
        .map(SearchEntry::construct)
        .collect();
    
    info!("[Test] objectClass=*Object* found {} entries", entries_obj_sub3.len());
    assert!(entries_obj_sub3.len() >= 3, "objectClass=*Object* should return at least 3 test users, found {}", entries_obj_sub3.len());
    info!("[Test] objectClass=*Object* query completed successfully");
    
    // 测试 4.5.8: objectClass 子串匹配查询 - 不匹配的值（应该返回空结果）
    info!("[Test] Test 4.5.8: objectClass substring query - non-matching value (should return empty)");
    let (rs_obj_sub4, _res_obj_sub4) = ldap
        .search(
            &base_dn,
            Scope::Subtree,
            "(objectClass=*Group*)",
            vec!["sAMAccountName", "cn", "objectClass"],
        )
        .await
        .map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP search error: {e:?}"), "500-ldap-search-error"))?
        .success().map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP search error: {e:?}"), "500-ldap-search-error"))?;
    
    let entries_obj_sub4: Vec<SearchEntry> = rs_obj_sub4
        .into_iter()
        .map(SearchEntry::construct)
        .collect();
    
    info!("[Test] objectClass=*Group* found {} entries", entries_obj_sub4.len());
    assert!(entries_obj_sub4.is_empty(), "objectClass=*Group* should return empty result (no matching fixed value), found {}", entries_obj_sub4.len());
    info!("[Test] objectClass=*Group* query correctly returned empty result");
    
    // 测试 4.5.9: objectClass 与其他条件的 AND 组合
    info!("[Test] Test 4.5.9: objectClass combined with AND filter");
    let (rs_obj_and, _res_obj_and) = ldap
        .search(
            &base_dn,
            Scope::Subtree,
            "(&(objectClass=inetOrgPerson)(sAMAccountName=testuser1))",
            vec!["sAMAccountName", "cn", "objectClass"],
        )
        .await
        .map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP search error: {e:?}"), "500-ldap-search-error"))?
        .success().map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP search error: {e:?}"), "500-ldap-search-error"))?;
    
    let entries_obj_and: Vec<SearchEntry> = rs_obj_and
        .into_iter()
        .map(SearchEntry::construct)
        .collect();
    
    info!("[Test] objectClass AND sAMAccountName found {} entries", entries_obj_and.len());
    assert!(!entries_obj_and.is_empty(), "objectClass AND sAMAccountName should find at least one entry");
    let found_obj_and_username = entries_obj_and[0].attrs.get("sAMAccountName").and_then(|v| v.first()).unwrap();
    assert_eq!(found_obj_and_username, "testuser1", "Should find testuser1");
    info!("[Test] objectClass AND filter query completed successfully");
    
    // 测试 4.5.10: objectClass 与其他条件的 OR 组合
    info!("[Test] Test 4.5.10: objectClass combined with OR filter");
    let (rs_obj_or, _res_obj_or) = ldap
        .search(
            &base_dn,
            Scope::Subtree,
            "(|(objectClass=inetOrgPerson)(sAMAccountName=nonexistent))",
            vec!["sAMAccountName", "cn", "objectClass"],
        )
        .await
        .map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP search error: {e:?}"), "500-ldap-search-error"))?
        .success().map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP search error: {e:?}"), "500-ldap-search-error"))?;
    
    let entries_obj_or: Vec<SearchEntry> = rs_obj_or
        .into_iter()
        .map(SearchEntry::construct)
        .collect();
    
    info!("[Test] objectClass OR filter found {} entries", entries_obj_or.len());
    assert!(entries_obj_or.len() >= 3, "objectClass OR filter should return at least 3 test users, found {}", entries_obj_or.len());
    info!("[Test] objectClass OR filter query completed successfully");
    
    // 测试 4.5.11: objectClass 与 NOT 组合
    info!("[Test] Test 4.5.11: objectClass combined with NOT filter");
    let (rs_obj_not, _res_obj_not) = ldap
        .search(
            &base_dn,
            Scope::Subtree,
            "(!(objectClass=person))",
            vec!["sAMAccountName", "cn", "objectClass"],
        )
        .await
        .map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP search error: {e:?}"), "500-ldap-search-error"))?
        .success().map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP search error: {e:?}"), "500-ldap-search-error"))?;
    
    let entries_obj_not: Vec<SearchEntry> = rs_obj_not
        .into_iter()
        .map(SearchEntry::construct)
        .collect();
    
    info!("[Test] NOT(objectClass=person) found {} entries", entries_obj_not.len());
    // NOT(objectClass=person) 应该返回所有用户，因为 person 不在固定列表中，所以 objectClass=person 返回空，NOT 后返回所有
    assert!(entries_obj_not.len() >= 3, "NOT(objectClass=person) should return at least 3 test users, found {}", entries_obj_not.len());
    info!("[Test] objectClass NOT filter query completed successfully");
    
    // 定义期望的用户名列表，用于后续的 scope 测试
    let expected_usernames: Vec<&str> = vec!["testuser1", "testuser2", "testuser3"];
    
    // 测试 4.6: 测试不同的 Scope - Base scope（只搜索 base DN 本身）
    info!("[Test] Test 4.6: Testing Base scope - searching specific user DN");
    let specific_user_dn = format!("CN={},ou=staff,DC={}", test_username, ldap_dc);
    let (rs_base, _res_base) = ldap
        .search(
            &specific_user_dn,
            Scope::Base,
            "(objectClass=*)",
            vec!["sAMAccountName", "cn"],
        )
        .await
        .map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP search error: {e:?}"), "500-ldap-search-error"))?
        .success().map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP search error: {e:?}"), "500-ldap-search-error"))?;
    
    let entries_base: Vec<SearchEntry> = rs_base
        .into_iter()
        .map(SearchEntry::construct)
        .collect();
    
    info!("[Test] Base scope found {} entries", entries_base.len());
    // Base scope 应该只返回一个条目（base DN 本身）
    assert_eq!(entries_base.len(), 1, "Base scope should return exactly 1 entry, found {}", entries_base.len());
    let found_base_cn = entries_base[0].attrs.get("cn").and_then(|v| v.first()).unwrap();
    assert_eq!(found_base_cn, test_username, "Base scope should return the specific user");
    info!("[Test] Base scope correctly returned user: {}", found_base_cn);
    
    // 测试 4.7: 测试不同的 Scope - OneLevel scope（只搜索直接子级）
    info!("[Test] Test 4.7: Testing OneLevel scope - searching direct children of base DN");
    let (rs_onelevel, _res_onelevel) = ldap
        .search(
            &format!("ou=staff,{}", base_dn),
            Scope::OneLevel,
            "(objectClass=*)",
            vec!["sAMAccountName", "cn"],
        )
        .await
        .map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP search error: {e:?}"), "500-ldap-search-error"))?
        .success().map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP search error: {e:?}"), "500-ldap-search-error"))?;
    
    let entries_onelevel: Vec<SearchEntry> = rs_onelevel
        .into_iter()
        .map(SearchEntry::construct)
        .collect();
    
    info!("[Test] OneLevel scope found {} entries", entries_onelevel.len());
    // OneLevel scope 应该返回 base DN 的直接子级
    // 在当前实现中，所有用户都是 base DN 的直接子级，所以应该返回所有用户
    assert!(entries_onelevel.len() >= 3, "OneLevel scope should return at least 3 test users, found {}", entries_onelevel.len());
    
    let mut found_onelevel_usernames: Vec<String> = entries_onelevel
        .iter()
        .filter_map(|entry| {
            entry.attrs.get("sAMAccountName")
                .and_then(|v| v.first()).cloned()
        })
        .collect();
    
    found_onelevel_usernames.sort();
    info!("[Test] OneLevel scope found usernames: {:?}", found_onelevel_usernames);
    
    // 验证所有测试用户都在 OneLevel 结果中
    for expected_username in &expected_usernames {
        assert!(
            found_onelevel_usernames.contains(&expected_username.to_string()),
            "OneLevel scope should find user: {}, found: {:?}",
            expected_username,
            found_onelevel_usernames
        );
    }
    info!("[Test] OneLevel scope found all test users successfully");
    
    // 测试 4.8: 测试不同的 Scope - Subtree scope（递归搜索所有子级）
    info!("[Test] Test 4.8: Testing Subtree scope - searching all descendants of base DN");
    let (rs_subtree, _res_subtree) = ldap
        .search(
            &format!("ou=staff,{}", base_dn),
            Scope::Subtree,
            "(objectClass=*)",
            vec!["sAMAccountName", "cn"],
        )
        .await
        .map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP search error: {e:?}"), "500-ldap-search-error"))?
        .success().map_err(|e| tardis::basic::error::TardisError::internal_error(&format!("LDAP search error: {e:?}"), "500-ldap-search-error"))?;
    
    let entries_subtree: Vec<SearchEntry> = rs_subtree
        .into_iter()
        .map(SearchEntry::construct)
        .collect();
    
    info!("[Test] Subtree scope found {} entries", entries_subtree.len());
    // Subtree scope 应该返回 base DN 及其所有子级（递归）
    // 在当前实现中，应该返回所有用户
    assert!(entries_subtree.len() >= 3, "Subtree scope should return at least 3 test users, found {}", entries_subtree.len());
    
    let mut found_subtree_usernames: Vec<String> = entries_subtree
        .iter()
        .filter_map(|entry| {
            entry.attrs.get("sAMAccountName")
                .and_then(|v| v.first()).cloned()
        })
        .collect();
    
    found_subtree_usernames.sort();
    info!("[Test] Subtree scope found usernames: {:?}", found_subtree_usernames);
    
    // 验证所有测试用户都在 Subtree 结果中
    for expected_username in &expected_usernames {
        assert!(
            found_subtree_usernames.contains(&expected_username.to_string()),
            "Subtree scope should find user: {}, found: {:?}",
            expected_username,
            found_subtree_usernames
        );
    }
    info!("[Test] Subtree scope found all test users successfully");
    
    // 测试 4.9: 比较不同 Scope 的返回结果数量
    info!("[Test] Test 4.9: Comparing result counts across different scopes");
    info!("[Test] Base scope count: {}", entries_base.len());
    info!("[Test] OneLevel scope count: {}", entries_onelevel.len());
    info!("[Test] Subtree scope count: {}", entries_subtree.len());
    
    // Base scope 应该返回最少的结果（只有1个）
    assert!(entries_base.len() <= entries_onelevel.len(), "Base scope should return fewer or equal results than OneLevel");
    assert!(entries_base.len() <= entries_subtree.len(), "Base scope should return fewer or equal results than Subtree");
    
    // OneLevel 和 Subtree 在当前实现中应该返回相同数量的结果
    // （因为所有用户都在同一级别，没有嵌套结构）
    // 但在实际LDAP中，Subtree 可能返回更多结果（如果存在嵌套）
    info!("[Test] Scope comparison completed successfully");
    
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
