use bios_basic::{
    helper::request_helper::get_real_ip_from_ctx,
    rbum::{
        dto::rbum_filer_dto::{RbumBasicFilterReq, RbumSetCateFilterReq},
        serv::{rbum_crud_serv::RbumCrudOperation, rbum_item_serv::RbumItemCrudOperation, rbum_set_serv::RbumSetCateServ},
    },
};
use bios_sdk_invoke::clients::{
    event_client::{BiosEventCenter, EventCenter, EventExt},
    spi_log_client::{LogItemAddReq, SpiLogClient},
};
use serde::{Deserialize, Serialize};

use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    chrono::{DateTime, Utc},
    serde_json::json,
    tokio, TardisFuns, TardisFunsInst,
};

use crate::{
    basic::{
        dto::iam_filer_dto::{IamAccountFilterReq, IamResFilterReq, IamRoleFilterReq, IamTenantFilterReq},
        serv::{iam_account_serv::IamAccountServ, iam_cert_serv::IamCertServ, iam_res_serv::IamResServ, iam_role_serv::IamRoleServ, iam_tenant_serv::IamTenantServ},
    },
    iam_constants::{self, IAM_AVATAR},
    iam_enumeration::IamCertKernelKind,
};

use super::iam_kv_client::IamKvClient;
pub struct IamLogClient;

#[derive(Deserialize, Serialize, Default, Debug, Clone)]
struct LogConfig {
    code: String,
    icon: String,
    color: String,
    label: String,
}

#[derive(Serialize, Default, Debug, Clone)]
pub struct LogParamContent {
    pub op: String,
    pub key: Option<String>,
    pub name: String,
    pub ak: String,
    pub ip: String,
    pub key_name: Option<String>,
}

#[derive(Clone)]
pub enum LogParamTag {
    IamTenant,
    IamOrg,
    IamAccount,
    IamRole,
    IamRes,
    IamSystem,
    SecurityAlarm,
    SecurityVisit,
    Log,
    Token,
}

impl From<LogParamTag> for String {
    fn from(val: LogParamTag) -> Self {
        match val {
            LogParamTag::IamTenant => "iam_tenant".to_string(),
            LogParamTag::IamOrg => "iam_org".to_string(),
            LogParamTag::IamAccount => "iam_account".to_string(),
            LogParamTag::IamRole => "iam_role".to_string(),
            LogParamTag::IamRes => "iam_res".to_string(),
            LogParamTag::IamSystem => "iam_system".to_string(),
            LogParamTag::SecurityAlarm => "security_alarm".to_string(),
            LogParamTag::SecurityVisit => "security_visit".to_string(),
            LogParamTag::Log => "log".to_string(),
            LogParamTag::Token => "token".to_string(),
        }
    }
}

impl IamLogClient {
    pub async fn add_ctx_task(tag: LogParamTag, key: Option<String>, op_describe: Option<String>, op_kind: Option<String>, ctx: &TardisContext) -> TardisResult<()> {
        let ctx_clone = ctx.clone();
        ctx.add_async_task(Box::new(|| {
            Box::pin(async move {
                let task_handle = tokio::spawn(async move {
                    let funs = iam_constants::get_tardis_inst();
                    let mut ip = "".to_string();
                    if let Ok(remote_ip) = get_real_ip_from_ctx(&ctx_clone).await {
                        ip = remote_ip.unwrap_or_default();
                    }
                    let op_label = IamKvClient::get_item(format!("__tag__:_:{}", String::from(tag.clone())), None, &funs, &ctx_clone)
                        .await
                        .unwrap_or_default()
                        .map(|kv_desp| {
                            TardisFuns::json
                                .json_to_obj::<Vec<LogConfig>>(kv_desp.value)
                                .unwrap_or_default()
                                .into_iter()
                                .find(|conf| conf.code == op_kind.clone().unwrap_or_default())
                                .map(|conf| conf.label)
                                .unwrap_or_default()
                        })
                        .unwrap_or_default();
                    IamLogClient::add_item(
                        tag,
                        LogParamContent {
                            op: if let Some(op_describe) = op_describe {
                                format!("{}:{}", op_label, op_describe)
                            } else {
                                if op_label.is_empty() && op_describe.is_some() {
                                    op_describe.unwrap_or_default()
                                } else {
                                    op_label
                                }
                            },
                            key: key.clone(),
                            ip,
                            ..Default::default()
                        },
                        None,
                        key.clone(),
                        op_kind,
                        None,
                        Some(tardis::chrono::Utc::now().to_rfc3339()),
                        &funs,
                        &ctx_clone,
                    )
                    .await
                    .unwrap();
                });
                task_handle.await.unwrap();
                Ok(())
            })
        }))
        .await
    }

    async fn add_item(
        tag: LogParamTag,
        content: LogParamContent,
        kind: Option<String>,
        key: Option<String>,
        op_kind: Option<String>,
        rel_key: Option<String>,
        ts: Option<String>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<()> {
        let mut content = content.clone();
        // find operates info
        if let Ok(cert) = IamCertServ::get_cert_detail_by_id_and_kind(ctx.owner.as_str(), &IamCertKernelKind::UserPwd, funs, ctx).await {
            content.ak = cert.ak;
            content.name = cert.owner_name.unwrap_or("".to_string());
        }
        // get ext name
        content.key_name = Self::get_key_name(&tag, content.key.as_deref(), funs, ctx).await;
        // create search_ext
        let search_ext = json!({
            "name":content.name,
            "ak":content.ak,
            "ip":content.ip,
            "key":content.key,
            "ts":ts,
            "op":op_kind,
        });
        // generate log item
        let tag: String = tag.into();
        let own_paths = if ctx.own_paths.len() < 2 { None } else { Some(ctx.own_paths.clone()) };
        let owner = if ctx.owner.len() < 2 { None } else { Some(ctx.owner.clone()) };
        let add_req = LogItemAddReq {
            tag,
            content: TardisFuns::json.obj_to_string(&content).expect("req_msg not a valid json value"),
            kind,
            ext: Some(search_ext),
            key,
            op: op_kind,
            rel_key,
            id: None,
            ts: ts.map(|ts| DateTime::parse_from_rfc3339(&ts).unwrap_or_default().with_timezone(&Utc)),
            owner,
            own_paths,
        };
        if let Some(ws_client) = BiosEventCenter::worker_queue() {
            ws_client.publish(add_req.with_source(IAM_AVATAR).inject_context(funs, ctx)).await?;
        } else {
            SpiLogClient::add(&add_req, funs, ctx).await?;
        }
        Ok(())
    }

    async fn get_key_name(tag: &LogParamTag, key: Option<&str>, funs: &TardisFunsInst, ctx: &TardisContext) -> Option<String> {
        if let Some(key) = key {
            match tag {
                LogParamTag::IamTenant => {
                    if let Ok(item) = IamTenantServ::get_item(
                        key,
                        &IamTenantFilterReq {
                            basic: RbumBasicFilterReq {
                                ignore_scope: true,
                                with_sub_own_paths: true,
                                own_paths: Some("".to_string()),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        funs,
                        ctx,
                    )
                    .await
                    {
                        Some(item.name)
                    } else {
                        None
                    }
                }
                LogParamTag::IamOrg => {
                    if let Ok(item) = RbumSetCateServ::get_rbum(
                        key,
                        &RbumSetCateFilterReq {
                            basic: RbumBasicFilterReq {
                                ignore_scope: true,
                                with_sub_own_paths: true,
                                own_paths: Some("".to_string()),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        funs,
                        ctx,
                    )
                    .await
                    {
                        Some(item.name)
                    } else {
                        None
                    }
                }
                LogParamTag::IamAccount => {
                    if let Ok(item) = IamAccountServ::get_item(
                        key,
                        &IamAccountFilterReq {
                            basic: RbumBasicFilterReq {
                                ignore_scope: true,
                                with_sub_own_paths: true,
                                own_paths: Some("".to_string()),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        funs,
                        ctx,
                    )
                    .await
                    {
                        Some(item.name)
                    } else {
                        None
                    }
                }
                LogParamTag::IamRole => {
                    if let Ok(item) = IamRoleServ::get_item(
                        key,
                        &IamRoleFilterReq {
                            basic: RbumBasicFilterReq {
                                ignore_scope: true,
                                with_sub_own_paths: true,
                                own_paths: Some("".to_string()),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        funs,
                        ctx,
                    )
                    .await
                    {
                        Some(item.name)
                    } else {
                        None
                    }
                }
                LogParamTag::IamRes => {
                    if let Ok(item) = IamResServ::get_item(
                        key,
                        &IamResFilterReq {
                            basic: RbumBasicFilterReq {
                                ignore_scope: true,
                                with_sub_own_paths: true,
                                own_paths: Some("".to_string()),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        funs,
                        ctx,
                    )
                    .await
                    {
                        Some(item.name)
                    } else {
                        None
                    }
                }
                LogParamTag::IamSystem => {
                    if let Ok(item) = IamResServ::get_item(
                        key,
                        &IamResFilterReq {
                            basic: RbumBasicFilterReq {
                                ignore_scope: true,
                                with_sub_own_paths: true,
                                own_paths: Some("".to_string()),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        funs,
                        ctx,
                    )
                    .await
                    {
                        Some(item.name)
                    } else {
                        None
                    }
                }
                LogParamTag::SecurityAlarm => None,
                LogParamTag::SecurityVisit => {
                    if let Ok(item) = IamAccountServ::get_item(
                        key,
                        &IamAccountFilterReq {
                            basic: RbumBasicFilterReq {
                                ignore_scope: true,
                                with_sub_own_paths: true,
                                own_paths: Some("".to_string()),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        funs,
                        ctx,
                    )
                    .await
                    {
                        Some(item.name)
                    } else {
                        None
                    }
                }
                LogParamTag::Log => None,
                LogParamTag::Token => None,
            }
        } else {
            None
        }
    }
}
