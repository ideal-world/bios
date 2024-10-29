use bios_basic::{
    helper::request_helper::get_real_ip_from_ctx,
    rbum::{
        dto::rbum_filer_dto::{RbumBasicFilterReq, RbumSetCateFilterReq},
        serv::{rbum_crud_serv::RbumCrudOperation, rbum_item_serv::RbumItemCrudOperation, rbum_set_serv::RbumSetCateServ},
    },
};
use bios_sdk_invoke::clients::spi_log_client::{LogItemAddV2Req, SpiLogClient};
use serde::Serialize;

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
    iam_constants::{self},
    iam_enumeration::IamCertKernelKind,
};
pub struct IamLogClient;

#[derive(Serialize, Default, Debug, Clone)]
pub struct LogParamContent {
    pub op: String,
    pub key: Option<String>,
    pub name: String,
    pub ak: String,
    pub ip: String,
    pub key_name: Option<String>,
}

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
    pub async fn add_ctx_task(tag: LogParamTag, key: Option<String>, op_describe: String, op_kind: Option<String>, ctx: &TardisContext) -> TardisResult<()> {
        let ctx_clone = ctx.clone();
        ctx.add_async_task(Box::new(|| {
            Box::pin(async move {
                let task_handle = tokio::spawn(async move {
                    let funs = iam_constants::get_tardis_inst();
                    let mut ip = "".to_string();
                    if let Ok(remote_ip) = get_real_ip_from_ctx(&ctx_clone).await {
                        ip = remote_ip.unwrap_or_default();
                    }
                    IamLogClient::add_item(
                        tag,
                        LogParamContent {
                            op: op_describe,
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

    pub async fn add_item(
        tag: LogParamTag,
        content: LogParamContent,
        kind: Option<String>,
        key: Option<String>,
        op: Option<String>,
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
            "op":op,
        });
        // generate log item
        let tag: String = tag.into();
        let own_paths = if ctx.own_paths.len() < 2 { None } else { Some(ctx.own_paths.clone()) };
        let owner = if ctx.owner.len() < 2 { None } else { Some(ctx.owner.clone()) };

        let add_req = LogItemAddV2Req {
            tag,
            content: TardisFuns::json.obj_to_json(&content).expect("req_msg not a valid json value"),
            kind,
            ext: Some(search_ext),
            key,
            op,
            rel_key,
            idempotent_id: None,
            ts: ts.map(|ts| DateTime::parse_from_rfc3339(&ts).unwrap_or_default().with_timezone(&Utc)),
            owner,
            own_paths,
            msg: None,
            owner_name: None,
            push: false,
        };
        SpiLogClient::addv2(add_req, funs, ctx).await?;
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
