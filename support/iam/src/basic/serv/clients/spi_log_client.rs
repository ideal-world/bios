use bios_basic::rbum::{dto::rbum_filer_dto::RbumBasicFilterReq, serv::rbum_item_serv::RbumItemCrudOperation};
use serde::Serialize;
use std::collections::HashMap;

use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    log::debug,
    serde_json::json,
    TardisFuns, TardisFunsInst,
};

use crate::{
    basic::{
        dto::iam_filer_dto::IamAccountFilterReq,
        serv::{iam_account_serv::IamAccountServ, iam_cert_serv::IamCertServ},
    },
    iam_config::IamConfig,
    iam_enumeration::IamCertKernelKind,
};
pub struct SpiLogClient;

#[derive(Serialize, Default, Debug)]
pub struct LogParamContent {
    pub op: String,
    pub ext: Option<String>,
    pub name: String,
    pub ak: String,
    pub ip: String,
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
        }
    }
}

pub enum LogParamOp {
    Add,
    Modify,
    Delete,
    None,
}

impl From<LogParamOp> for String {
    fn from(val: LogParamOp) -> Self {
        match val {
            LogParamOp::Add => "Add".to_string(),
            LogParamOp::Modify => "Modify".to_string(),
            LogParamOp::Delete => "Delete".to_string(),
            LogParamOp::None => "".to_string(),
        }
    }
}

impl SpiLogClient {
    pub async fn add_item(
        tag: LogParamTag,
        mut content: LogParamContent,
        kind: Option<String>,
        key: Option<String>,
        op: LogParamOp,
        rel_key: Option<String>,
        ts: Option<String>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<()> {
        let log_url = funs.conf::<IamConfig>().spi.log_url.clone();
        if log_url.is_empty() {
            return Ok(());
        }
        let spi_ctx = TardisContext {
            owner: funs.conf::<IamConfig>().spi.owner.clone(),
            ..ctx.clone()
        };
        let headers = Some(vec![(
            "Tardis-Context".to_string(),
            TardisFuns::crypto.base64.encode(&TardisFuns::json.obj_to_string(&spi_ctx).unwrap()),
        )]);
        // find operater info
        let account = IamAccountServ::get_item(
            ctx.owner.as_str(),
            &IamAccountFilterReq {
                basic: RbumBasicFilterReq {
                    own_paths: Some("".to_owned()),
                    ignore_scope: true,
                    ..Default::default()
                },
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?;
        let cert = IamCertServ::get_kernel_cert(ctx.owner.as_str(), &IamCertKernelKind::UserPwd, funs, ctx).await?;
        content.name = account.name;
        content.ak = cert.ak;
        //add log item
        let mut body = HashMap::from([
            ("tag", tag.into()),
            ("content", TardisFuns::json.obj_to_string(&content).unwrap()),
            ("owner", ctx.owner.clone()),
            ("owner_paths", ctx.own_paths.clone()),
            ("op", op.into()),
        ]);
        // create search_ext
        let search_ext = json!({
            "name":content.name,
            "ak":content.ak,
            "ip":content.ip,
            "rel_key":rel_key,
            "ts":ts,
            "op":content.op,
        })
        .to_string();
        body.insert("search_ext", search_ext);

        if let Some(kind) = kind {
            body.insert("kind", kind);
        }

        if let Some(key) = key {
            body.insert("key", key);
        }
        if let Some(rel_key) = rel_key {
            body.insert("rel_key", rel_key);
        }
        if let Some(ts) = ts {
            body.insert("ts", ts);
        }
        funs.web_client().post_obj_to_str(&format!("{log_url}/ci/item"), &body, headers.clone()).await.unwrap();
        Ok(())
    }
}
