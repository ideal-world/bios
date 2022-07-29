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
//!
use std::convert::TryFrom;
use std::net;
use std::str::FromStr;

use ldap3_proto::simple::*;
use ldap3_proto::LdapCodec;
use tardis::basic::error::TardisError;
use tardis::basic::result::TardisResult;
use tardis::futures::SinkExt;
use tardis::futures::StreamExt;
use tardis::log::{error, info, trace};

use tardis::regex::Regex;
use tardis::tokio::net::{TcpListener, TcpStream};
use tardis::{tokio, TardisFuns};
use tokio_util::codec::{FramedRead, FramedWrite};

use crate::iam_config::{IamConfig, IamLdapConfig};
use crate::iam_constants;
use crate::integration::ldap::ldap_processor;

lazy_static! {
    static ref CN_R: Regex = Regex::new(r"(,|^)[cC][nN]=(.+?)(,|$)").expect("Regular parsing error");
}

struct LdapSession {
    dn: String,
}

impl LdapSession {
    pub async fn do_bind(&mut self, req: &SimpleBindRequest, config: &IamLdapConfig) -> LdapMsg {
        if req.dn == config.bind_dn && req.pw == config.bind_password {
            self.dn = req.dn.to_string();
            req.gen_success()
        } else if req.dn.is_empty() && req.pw.is_empty() {
            self.dn = "Anonymous".to_string();
            req.gen_invalid_cred()
        } else if !req.dn.to_lowercase().contains(&format!("DC={}", config.dc).to_lowercase()) {
            req.gen_invalid_cred()
        } else {
            self.dn = req.dn.to_string();
            match extract_cn(&req.dn) {
                None => req.gen_invalid_cred(),
                Some(cn) => match ldap_processor::check_cert(&cn, &req.pw).await {
                    Ok(true) => req.gen_success(),
                    Ok(false) => req.gen_invalid_cred(),
                    Err(_) => req.gen_error(LdapResultCode::Unavailable, "Service internal error".to_string()),
                },
            }
        }
    }

    pub async fn do_search(&mut self, req: &SearchRequest, config: &IamLdapConfig) -> Vec<LdapMsg> {
        match &req.filter {
            LdapFilter::And(_) | LdapFilter::Or(_) | LdapFilter::Not(_) | LdapFilter::Substring(_, _) => {
                vec![req.gen_error(LdapResultCode::Other, "This operation is not currently supported".to_string())]
            }
            LdapFilter::Equality(_, cn) => {
                if !req.base.to_lowercase().contains(&format!("DC={}", config.dc).to_lowercase()) {
                    return vec![req.gen_error(LdapResultCode::NoSuchObject, "DN is invalid".to_string())];
                }
                match ldap_processor::check_exist(cn).await {
                    Ok(true) => vec![
                        req.gen_result_entry(LdapSearchResultEntry {
                            dn: format!("CN={},DC={}", cn, config.dc),
                            attributes: vec![
                                LdapPartialAttribute {
                                    atype: "sAMAccountName".to_string(),
                                    vals: vec![cn.to_string()],
                                },
                                // TODO
                                LdapPartialAttribute {
                                    atype: "mail".to_string(),
                                    vals: vec![format!("{}@example.com", cn)],
                                },
                                // TODO
                                LdapPartialAttribute {
                                    atype: "cn".to_string(),
                                    vals: vec![cn.to_string()],
                                },
                                // TODO
                                LdapPartialAttribute {
                                    atype: "givenName".to_string(),
                                    vals: vec!["".to_string()],
                                },
                                // TODO
                                LdapPartialAttribute {
                                    atype: "sn".to_string(),
                                    vals: vec![cn.to_string()],
                                },
                            ],
                        }),
                        req.gen_success(),
                    ],
                    Ok(false) => vec![req.gen_error(LdapResultCode::NoSuchObject, "CN not exist".to_string())],
                    Err(_) => vec![req.gen_error(LdapResultCode::Unavailable, "Service internal error".to_string())],
                }
            }
            LdapFilter::Present(k) => {
                if k == "objectClass" && req.base.is_empty() {
                    // https://ldap.com/dit-and-the-ldap-root-dse/
                    // https://docs.oracle.com/cd/E19957-01/817-6707/srvrinfo.html
                    return vec![
                        req.gen_result_entry(LdapSearchResultEntry {
                            dn: format!("DC={}", config.dc),
                            attributes: vec![],
                        }),
                        req.gen_success(),
                    ];
                }
                if !req.base.to_lowercase().contains(&format!("DC={}", config.dc).to_lowercase()) {
                    return vec![req.gen_error(LdapResultCode::NoSuchObject, "DN is invalid".to_string())];
                }
                match extract_cn(&req.base) {
                    None => vec![req.gen_error(LdapResultCode::NoSuchObject, "CN is invalid".to_string())],
                    Some(cn) => match ldap_processor::check_exist(&cn).await {
                        Ok(true) => vec![
                            req.gen_result_entry(LdapSearchResultEntry {
                                dn: format!("CN={},DC={}", cn, config.dc),
                                attributes: vec![
                                    LdapPartialAttribute {
                                        atype: "sAMAccountName".to_string(),
                                        vals: vec![cn.clone()],
                                    },
                                    // TODO
                                    LdapPartialAttribute {
                                        atype: "mail".to_string(),
                                        vals: vec![format!("{}@example.com", cn.clone())],
                                    },
                                    // TODO
                                    LdapPartialAttribute {
                                        atype: "cn".to_string(),
                                        vals: vec![cn.clone()],
                                    },
                                    // TODO
                                    LdapPartialAttribute {
                                        atype: "givenName".to_string(),
                                        vals: vec!["".to_string()],
                                    },
                                    // TODO
                                    LdapPartialAttribute {
                                        atype: "sn".to_string(),
                                        vals: vec![cn.clone()],
                                    },
                                ],
                            }),
                            req.gen_success(),
                        ],
                        Ok(false) => vec![req.gen_error(LdapResultCode::NoSuchObject, "CN not exist".to_string())],
                        Err(_) => vec![req.gen_error(LdapResultCode::Unavailable, "Service internal error".to_string())],
                    },
                }
            }
        }
    }

    pub fn do_whoami(&mut self, req: &WhoamiRequest) -> LdapMsg {
        req.gen_success(format!("DN: {}", self.dn).as_str())
    }
}

fn extract_cn(dn: &str) -> Option<String> {
    match CN_R.captures(dn) {
        None => None,
        Some(cap) => {
            let cn = cap.get(2).unwrap().as_str();
            Some(cn.to_string())
        }
    }
}

async fn handle_client(socket: TcpStream, _addr: net::SocketAddr, config: &IamLdapConfig) {
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

pub async fn start() -> TardisResult<()> {
    let config = TardisFuns::cs_config::<IamConfig>(iam_constants::COMPONENT_CODE);
    let config = &config.ldap;
    let addr_str = format!("0.0.0.0:{}", config.port);
    let addr = net::SocketAddr::from_str(&addr_str).map_err(|e| TardisError::format_error(&format!("[TardisLdapServer] Address error: {:?}", e), "406-iam-ldap-addr-error"))?;
    let listener = Box::new(TcpListener::bind(&addr).await.unwrap());
    tokio::spawn(async move {
        loop {
            match listener.accept().await {
                Ok((socket, addr)) => {
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
