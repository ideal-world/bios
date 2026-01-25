# LDAP Search Scope 实现方案

## 概述

根据 `SearchRequest` 中的 `base` 和 `scope` 参数来确定搜索的作用域，实现符合 LDAP 协议的搜索范围限制。

## LDAP Scope 类型

LDAP 定义了三种搜索范围（Scope）：

1. **Base**: 只搜索 base DN 本身
2. **OneLevel**: 只搜索 base DN 的直接子级
3. **Subtree**: 递归搜索 base DN 及其所有子级

## 当前实现状态

- `LdapSearchQuery` 结构体已经包含了 `base` 和 `scope` 字段
- `parse_search_request` 函数已经正确解析了这两个字段
- 但在 `build_and_execute_sql_query` 函数中，没有根据 `scope` 来限制搜索范围
- 目前只处理了 `scope == Base` 的特殊情况（在 `is_simple_present_query` 中）

## LDAP 层次结构

根据代码分析，LDAP 的层次结构如下：

```
(根节点，空 DN)
  └── DC={dc}
       ├── ou={ou_staff},DC={dc}
       │    └── cn={cn},ou={ou_staff},DC={dc}  (账号)
       └── ou={ou_organization},DC={dc}
            └── cn={cn},ou={ou_organization},DC={dc}  (组织)
```

## 实现方案

### 1. 在 `ldap_parser.rs` 中添加辅助函数

添加以下函数来解析 base DN 和判断层级关系：

```rust
/// 从 base DN 中提取所有组件
/// 返回 (cn, ou, dc) 的元组
pub fn parse_base_dn_components(base: &str, config: &IamLdapConfig) -> (Option<String>, Option<String>, Option<String>) {
    let mut cn = None;
    let mut ou = None;
    let mut dc = None;
    
    // 提取 CN
    if let Some(cn_val) = extract_cn_from_base(base) {
        cn = Some(cn_val);
    }
    
    // 提取 OU
    if let Some(ou_val) = extract_ou_from_base(base) {
        ou = Some(ou_val);
    }
    
    // 提取 DC（通过检查是否包含配置的 DC）
    if base.to_lowercase().contains(&format!("dc={}", config.dc).to_lowercase()) {
        dc = Some(config.dc.clone());
    }
    
    (cn, ou, dc)
}

/// 判断 base DN 的层级类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BaseDnLevel {
    /// 根节点（空 DN）
    Root,
    /// DC 级别：DC={dc}
    Dc,
    /// OU 级别：ou={ou},DC={dc}
    Ou,
    /// 账号级别：cn={cn},ou={ou},DC={dc}
    Account,
}

/// 判断 base DN 的层级
pub fn get_base_dn_level(base: &str, config: &IamLdapConfig) -> BaseDnLevel {
    if base.is_empty() {
        return BaseDnLevel::Root;
    }
    
    let (cn, ou, _) = parse_base_dn_components(base, config);
    
    if cn.is_some() {
        BaseDnLevel::Account
    } else if ou.is_some() {
        BaseDnLevel::Ou
    } else if base.to_lowercase().contains(&format!("dc={}", config.dc).to_lowercase()) {
        BaseDnLevel::Dc
    } else {
        BaseDnLevel::Root
    }
}
```

### 2. 在 `account_query.rs` 中修改 `build_and_execute_sql_query` 函数

根据 scope 和 base 添加额外的 SQL WHERE 条件：

```rust
/// 根据 scope 和 base 构建额外的 SQL WHERE 条件
fn build_scope_where_clause(
    query: &ldap_parser::LdapSearchQuery,
    config: &IamLdapConfig,
) -> TardisResult<Option<String>> {
    use ldap3_proto::simple::LdapSearchScope;
    use ldap_parser::{get_base_dn_level, parse_base_dn_components, BaseDnLevel};
    
    match query.scope {
        LdapSearchScope::Base => {
            // Base scope: 只搜索 base DN 本身
            // 如果 base DN 包含 CN，则只查询该 CN 对应的账户
            if let Some(cn) = ldap_parser::extract_cn_from_base(&query.base) {
                // 转义 CN 值以防止 SQL 注入
                let escaped_cn = cn.replace("'", "''");
                Ok(Some(format!("user_pwd_cert.ak = '{}'", escaped_cn)))
            } else {
                // 如果 base DN 不包含 CN（如 OU 或 DC），Base scope 不应该返回账户
                // 返回一个永远为假的条件
                Ok(Some("1=0".to_string()))
            }
        }
        LdapSearchScope::OneLevel => {
            // OneLevel scope: 只搜索 base DN 的直接子级
            let base_level = get_base_dn_level(&query.base, config);
            match base_level {
                BaseDnLevel::Root | BaseDnLevel::Dc => {
                    // 如果 base 是根节点或 DC，直接子级包括 OU 和账号
                    // 在当前实现中，所有账号都在 ou={ou_staff} 下
                    // 所以返回所有账号（不添加额外限制）
                    Ok(None)
                }
                BaseDnLevel::Ou => {
                    // 如果 base 是 OU，直接子级是该 OU 下的账号
                    // 检查 OU 是否匹配配置的 ou_staff
                    let (_, ou, _) = parse_base_dn_components(&query.base, config);
                    if let Some(ou_val) = ou {
                        if ou_val.to_lowercase() == config.ou_staff.to_lowercase() {
                            // 匹配 ou_staff，返回该 OU 下的所有账号（不添加额外限制）
                            Ok(None)
                        } else {
                            // 不匹配 ou_staff，不返回账号
                            Ok(Some("1=0".to_string()))
                        }
                    } else {
                        Ok(Some("1=0".to_string()))
                    }
                }
                BaseDnLevel::Account => {
                    // 如果 base 是账号级别，OneLevel scope 不应该返回任何账户
                    // （账号没有子级）
                    Ok(Some("1=0".to_string()))
                }
            }
        }
        LdapSearchScope::Subtree => {
            // Subtree scope: 递归搜索 base DN 及其所有子级
            // 这是当前的行为，不添加额外限制
            Ok(None)
        }
    }
}

/// 根据LDAP查询参数构建SQL并执行，返回符合条件的account id列表
async fn build_and_execute_sql_query(
    query: &ldap_parser::LdapSearchQuery,
    config: &IamLdapConfig,
    funs: &TardisFunsInst,
    _ctx: &TardisContext,
) -> TardisResult<Vec<String>> {
    // 构建过滤器相关的 SQL WHERE 条件
    let (join, filter_where_clause) = if ldap_parser::is_full_query(query) {
        ("", "".to_string())
    } else {
        ("AND", build_sql_where_clause(&query.query_type, config)?)
    };
    
    // 根据 scope 和 base 构建额外的 WHERE 条件
    let scope_where_clause = build_scope_where_clause(query, config)?;
    
    // 合并 WHERE 条件
    let mut where_parts = vec![
        "rbum_item.disabled = false".to_string(),
        "rbum_item.scope_level = 0".to_string(),
        "rbum_item.own_paths = ''".to_string(),
    ];
    
    if !filter_where_clause.is_empty() {
        where_parts.push(filter_where_clause);
    }
    
    if let Some(scope_clause) = scope_where_clause {
        where_parts.push(scope_clause);
    }
    
    let where_clause = where_parts.join(" AND ");
    
    // 构建完整的SQL查询语句
    let user_pwd_conf_id = IamCertServ::get_cert_conf_id_by_kind(
        &IamCertKernelKind::UserPwd.to_string(),
        Some("".to_string()),
        &funs,
    )
    .await?;
    let mail_vcode_conf_id = IamCertServ::get_cert_conf_id_by_kind(
        &IamCertKernelKind::MailVCode.to_string(),
        Some("".to_string()),
        &funs,
    )
    .await?;

    let sql = format!(
        r#"
        SELECT iam_account.id
        FROM iam_account
        LEFT JOIN rbum_cert AS user_pwd_cert ON user_pwd_cert.rel_rbum_id = iam_account.id 
            AND user_pwd_cert.rel_rbum_kind = 0 AND user_pwd_cert.rel_rbum_cert_conf_id = '{}'
        LEFT JOIN rbum_cert AS mail_vcode_cert ON mail_vcode_cert.rel_rbum_id = iam_account.id 
            AND mail_vcode_cert.rel_rbum_kind = 0 AND mail_vcode_cert.rel_rbum_cert_conf_id = '{}'
        LEFT JOIN rbum_item ON rbum_item.id = iam_account.id
        WHERE {}
        "#,
        user_pwd_conf_id, mail_vcode_conf_id, where_clause
    );

    let result = funs.db().query_all(&sql, vec![]).await?;

    // 提取account id列表
    let account_ids: Vec<String> = result
        .into_iter()
        .filter_map(|row| row.try_get::<String>("", "id").ok())
        .collect();

    Ok(account_ids)
}
```

### 3. 处理特殊情况

在 `execute_ldap_account_search` 函数中，需要确保 Base scope 的特殊处理逻辑与新的实现一致：

```rust
pub async fn execute_ldap_account_search(
    query: &ldap_parser::LdapSearchQuery,
    config: &IamLdapConfig,
) -> TardisResult<Vec<IamAccountDetailAggResp>> {
    let funs = iam_constants::get_tardis_inst();

    // 处理简单存在性查询（从base DN提取CN，检查账户是否存在）
    // 注意：这个逻辑已经处理了 Base scope 的情况
    if ldap_parser::is_simple_present_query(query) {
        if let Some(cn) = ldap_parser::extract_cn_from_base(&query.base) {
            match check_account_exists(&cn).await {
                Ok(true) => {
                    // 账户存在，返回账户详情
                    if let Ok(Some(account)) = get_account_by_cn(&cn).await {
                        return Ok(vec![account]);
                    }
                }
                Ok(false) => return Ok(vec![]),
                Err(e) => return Err(e),
            }
        }
        return Ok(vec![]);
    }

    // 使用原生SQL查询方式（现在会考虑 scope 限制）
    let ctx = TardisContext::default();

    // 根据query参数构建SQL查询，获取符合条件的account id列表
    let account_ids = build_and_execute_sql_query(query, config, &funs, &ctx).await?;

    // ... 其余代码保持不变
}
```

## 测试场景

实现后需要测试以下场景：

1. **Base scope**:
   - `base="cn=user1,ou=staff,dc=example"`, `scope=Base` → 只返回 user1
   - `base="ou=staff,dc=example"`, `scope=Base` → 不返回账户（OU 不是账户）

2. **OneLevel scope**:
   - `base="ou=staff,dc=example"`, `scope=OneLevel` → 返回该 OU 下的所有账户
   - `base="dc=example"`, `scope=OneLevel` → 返回所有账户（当前实现中所有账户都是 DC 的直接子级）
   - `base="cn=user1,ou=staff,dc=example"`, `scope=OneLevel` → 不返回账户（账号没有子级）

3. **Subtree scope**:
   - `base="dc=example"`, `scope=Subtree` → 返回所有账户
   - `base="ou=staff,dc=example"`, `scope=Subtree` → 返回该 OU 下的所有账户

## 注意事项

1. **SQL 注入防护**: 所有从 base DN 提取的值都需要进行 SQL 转义
2. **大小写敏感性**: LDAP DN 通常不区分大小写，但数据库查询可能需要考虑
3. **向后兼容**: 确保现有功能不受影响
4. **性能考虑**: 对于 Base scope，添加 CN 条件可以提高查询性能

## 实现步骤

1. 在 `ldap_parser.rs` 中添加辅助函数
2. 在 `account_query.rs` 中实现 `build_scope_where_clause` 函数
3. 修改 `build_and_execute_sql_query` 函数以使用 scope 限制
4. 运行现有测试确保向后兼容
5. 添加新的测试用例验证 scope 功能
