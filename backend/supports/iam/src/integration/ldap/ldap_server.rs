//! LDAP Service
//!
//! Support platform-level user and tenant-level user login.
//!
//! Note: Since the tenant Id is case-sensitive but the ldap is not, the login name format is: <tenant Id in hexadecimal>/<ak,username>
//!
//! ## Example(Using Gitlab)
//!
//! ### Configuration
//!
//! echo "
//! gitlab_rails['time_zone'] ='Asia/Shanghai'
//! gitlab_rails['gitlab_shell_ssh_port'] = 9922
//! gitlab_rails['ldap_enabled'] = true
//! gitlab_rails['prevent_ldap_sign_in'] = false
//! gitlab_rails['ldap_servers'] = {
//! 'main' => {
//!   'label' => 'LDAP',
//!   'host' =>  'x.x.x.x',
//!   'port' => x,
//!   'uid' => 'sAMAccountName',
//!   'encryption' => 'plain',
//!   'verify_certificates' => false,
//!   'bind_dn' => 'CN=ldapadmin,DC=bios',
//!   'password' => '24eFDK9242@',
//!   'timeout' => 10,
//!   'active_directory' => false,
//!   'allow_username_or_email_login' => false,
//!   'block_auto_created_users' => false,
//!   'base' => 'DC=bios',
//!   'user_filter' => '',
//!   'attributes' => {
//!     'username' => ['uid', 'userid', 'sAMAccountName'],
//!     'email' => ['mail', 'email', 'userPrincipalName'],
//!     'name' => 'cn',
//!     'first_name' => 'givenName',
//!     'last_name' => 'sn'
//!   },
//!   'lowercase_usernames' => false
//!   }
//! }
//! " >> /opt/volumes/gitlab/etc/gitlab/gitlab.rb
//!
//! ### Start
//!
//! docker run --name gitlab -p 9980:80 -p 9443:443 -p 9922:22 \
//!   -v /opt/volumes/gitlab/etc/gitlab:/etc/gitlab \
//!   -v /opt/volumes/gitlab/var/log/gitlab:/var/log/gitlab \
//!   -v /opt/volumes/gitlab/var/opt/gitlab:/var/opt/gitlab \
//!   -dit gitlab/gitlab-ce
use std::net;
use std::str::FromStr;
use std::sync::Arc;

use ldap3_proto::simple::*;
use ldap3_proto::LdapCodec;
use tardis::basic::error::TardisError;
use tardis::basic::result::TardisResult;
use tardis::futures::SinkExt;
use tardis::futures::StreamExt;
use tardis::log::{error, info, trace};

use tardis::tokio::net::{TcpListener, TcpStream};
use tardis::{tokio, TardisFuns};
use tokio_util::codec::{FramedRead, FramedWrite};

use crate::iam_config::{IamConfig, IamLdapConfig};
use crate::iam_constants;
use crate::integration::ldap::ldap_auth;
use crate::integration::ldap::ldap_parser;
use crate::integration::ldap::account::{account_query, account_result};
use crate::integration::ldap::organization::{org_query, org_result};
use crate::integration::ldap::system::system_result;

/// LDAP会话管理
struct LdapSession {
    dn: String,
}

impl LdapSession {
    /// 处理LDAP绑定请求（认证）
    pub async fn do_bind(&mut self, req: &SimpleBindRequest, config: &IamLdapConfig) -> LdapMsg {
        // 管理员绑定
        if req.dn == config.bind_dn && req.pw == config.bind_password {
            self.dn = req.dn.to_string();
            return req.gen_success();
        }

        // 匿名绑定
        if req.dn.is_empty() && req.pw.is_empty() {
            self.dn = "Anonymous".to_string();
            return req.gen_invalid_cred();
        }

        // 验证DN格式
        if !req.dn.to_lowercase().contains(&format!("DC={}", config.dc).to_lowercase()) {
            return req.gen_invalid_cred();
        }

        // 用户绑定：从DN提取CN并验证凭证
        self.dn = req.dn.to_string();
        match ldap_parser::extract_cn_from_dn(&req.dn) {
            None => req.gen_invalid_cred(),
            Some(cn) => match ldap_auth::check_cert(&cn, &req.pw).await {
                Ok(true) => req.gen_success(),
                Ok(false) => req.gen_invalid_cred(),
                Err(_) => req.gen_error(LdapResultCode::Unavailable, "Service internal error".to_string()),
            },
        }
    }

    /// 处理LDAP搜索请求
    pub async fn do_search(&mut self, req: &SearchRequest, config: &IamLdapConfig) -> Vec<LdapMsg> {
        // 解析搜索请求
        let query = match ldap_parser::parse_search_request(req, config) {
            Ok(q) => q,
            Err(_) => return build_error_response(
                req,
                LdapResultCode::Other,
                "Invalid search request".to_string(),
            ),
        };

        // 识别查询类型（根查询、schema查询、账号或组织）
        let entity_type = ldap_parser::identify_entity_type(&query);

        // 根据查询类型路由到相应的处理逻辑
        match entity_type {
            ldap_parser::LdapEntityType::RootDse => {
                // 处理根DSE查询
                system_result::build_system_search_response(req, &query, config)
            }
            ldap_parser::LdapEntityType::Subschema => {
                // 处理Schema查询
                system_result::build_system_search_response(req, &query, config)
            }
            ldap_parser::LdapEntityType::Account => {
                // 执行账号查询
                let accounts = match account_query::execute_ldap_account_search(&query, config).await {
                    Ok(accounts) => accounts,
                    Err(_) => {
                        return build_error_response(
                            req,
                            LdapResultCode::Unavailable,
                            "Service internal error".to_string(),
                        );
                    }
                };

                // 组装并返回LDAP响应
                account_result::build_account_search_response(req, &query, accounts, config)
            }
            ldap_parser::LdapEntityType::Organization => {
                // 执行组织查询（保留代码逻辑，但不返回结果）
                let _orgs = match org_query::execute_ldap_org_search(&query, config).await {
                    Ok(orgs) => orgs,
                    Err(_) => {
                        return build_error_response(
                            req,
                            LdapResultCode::Unavailable,
                            "Service internal error".to_string(),
                        );
                    }
                };

                // 不返回组织结果，返回空结果
                vec![req.gen_success()]
            }
            ldap_parser::LdapEntityType::Unknown => {
                // 如果无法确定类型，默认尝试账号查询
                // 这通常发生在查询根DN或没有明确OU的情况下
                let accounts = match account_query::execute_ldap_account_search(&query, config).await {
                    Ok(accounts) => accounts,
                    Err(_) => {
                        return build_error_response(
                            req,
                            LdapResultCode::Unavailable,
                            "Service internal error".to_string(),
                        );
                    }
                };

                // 只返回账号查询结果（即使为空，也不再尝试组织查询）
                account_result::build_account_search_response(req, &query, accounts, config)
            }
        }
    }

    /// 处理Whoami请求
    pub fn do_whoami(&mut self, req: &WhoamiRequest) -> LdapMsg {
        req.gen_success(format!("DN: {}", self.dn).as_str())
    }
}

/// 处理客户端连接
#[allow(clippy::blocks_in_conditions)]
async fn handle_client(socket: TcpStream, _addr: net::SocketAddr, config: Arc<IamConfig>) {
    let config = &config.ldap;
    let (r, w) = tokio::io::split(socket);
    let mut reqs = FramedRead::new(r, LdapCodec);
    let mut resp = FramedWrite::new(w, LdapCodec);

    let mut session = LdapSession { dn: "Anonymous".to_string() };

    while let Some(msg) = reqs.next().await {
        let server_op = match msg.map_err(|_e| ()).and_then(|msg| {
            trace!("[TardisLdapServer] Received message:{:?}", msg);
            ServerOps::try_from(msg)
        }) {
            Ok(v) => v,
            Err(_) => {
                let _err = resp.send(DisconnectionNotice::gen(LdapResultCode::Other, "Internal Server Error")).await;
                let _err = resp.flush().await;
                return;
            }
        };

        let result = match server_op {
            ServerOps::SimpleBind(req) => vec![session.do_bind(&req, config).await],
            ServerOps::Search(req) => session.do_search(&req, config).await,
            ServerOps::Unbind(_) => {
                // No need to notify on unbind (per rfc4511)
                return;
            }
            ServerOps::Whoami(req) => vec![session.do_whoami(&req)],
            ServerOps::Compare(_) => {
                // No need to notify on Compare (per rfc4511)
                return;
            }
        };

        for rmsg in result.into_iter() {
            if resp.send(rmsg).await.is_err() {
                return;
            }
        }
        if resp.flush().await.is_err() {
            return;
        }
    }
}

/// 启动LDAP服务器
pub async fn start() -> TardisResult<()> {
    let config = TardisFuns::cs_config::<IamConfig>(iam_constants::COMPONENT_CODE);
    let config = &config.ldap;
    let addr_str = format!("0.0.0.0:{}", config.port);
    let addr = net::SocketAddr::from_str(&addr_str).map_err(|e| TardisError::format_error(&format!("[TardisLdapServer] Address error: {e:?}"), "406-iam-ldap-addr-error"))?;
    let listener = Box::new(TcpListener::bind(&addr).await?);
    tokio::spawn(async move {
        loop {
            match listener.accept().await {
                Ok((socket, addr)) => {
                    let config = TardisFuns::cs_config::<IamConfig>(iam_constants::COMPONENT_CODE);
                    tokio::spawn(handle_client(socket, addr, config));
                }
                Err(e) => {
                    error!("[TardisLdapServer] Received error: {}", e.to_string())
                }
            }
        }
    });
    info!("[TardisLdapServer] Started ldap://{}", addr_str);
    Ok(())
}

/// 构建错误响应
fn build_error_response(req: &SearchRequest, code: LdapResultCode, message: String) -> Vec<LdapMsg> {
    vec![req.gen_error(code, message)]
}
