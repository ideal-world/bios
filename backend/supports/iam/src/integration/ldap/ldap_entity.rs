//! LDAP Entity
//!
//! 负责定义和构建 LDAP 实体，包含 LDAP 搜索结果条目及其父 DN 信息

use ldap3_proto::simple::*;

use crate::{iam_config::IamLdapConfig};

/// LDAP 实体数据
/// 存储 LDAP 搜索结果条目及其父 DN
#[derive(Debug, Clone)]
pub struct LdapEntity {
    /// LDAP 搜索结果条目
    pub entry: LdapSearchResultEntry,
    /// 父 DN
    pub parent_dn: String,
}

impl LdapEntity {
    /// 构建域节点
    /// DC 节点的 DN 格式为 DC={dc}，父 DN 为 空字符串（根节点）
    pub fn build_dc_node(config: &IamLdapConfig) -> Self {
        let dn = config.base_dn.clone();
        let attributes = vec![
            // dc: 域组件
            LdapPartialAttribute {
                atype: "dc".to_string(),
                vals: vec![config.dc.clone().into()],
            },
            // objectClass: 对象类
            LdapPartialAttribute {
                atype: "objectClass".to_string(),
                vals: vec!["top".to_string().into(), "domain".to_string().into()],
            },
        ];
        Self {
            entry: LdapSearchResultEntry {
                dn,
                attributes,
            },
            parent_dn: "".to_string(),
        }
    }

    /// 构建 OU 节点
    /// OU 节点的 DN 格式为 ou={ou_name},dc={dc}，父 DN 为 DC={dc}
    pub fn build_ou_node(ou_name: &str, config: &IamLdapConfig) -> Self {
        let parent_dn = config.base_dn.clone();
        let dn = format!("ou={},{}", ou_name, parent_dn);
        let attributes = vec![
            // ou: 组织单位
            LdapPartialAttribute {
                atype: "ou".to_string(),
                vals: vec![ou_name.to_string().into()],
            },
            // objectClass: 对象类
            LdapPartialAttribute {
                atype: "objectClass".to_string(),
                vals: vec!["top".to_string().into(), "organizationalUnit".to_string().into()],
            },
        ];
        Self {
            entry: LdapSearchResultEntry {
                dn,
                attributes,
            },
            parent_dn,
        }
    }
}