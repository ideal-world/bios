use bios_basic::rbum::{
    dto::rbum_filer_dto::{RbumBasicFilterReq, RbumCertFilterReq},
    serv::rbum_item_serv::RbumItemCrudOperation,
};
use std::collections::HashMap;

use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    serde_json::Value,
    tokio, TardisFuns, TardisFunsInst,
};

use crate::{
    basic::{
        dto::iam_filer_dto::IamAccountFilterReq,
        serv::{iam_account_serv::IamAccountServ, iam_cert_serv::IamCertServ},
    },
    iam_config::IamConfig,
};
pub struct SpiLogClient;

#[derive(Default, Debug)]
pub struct LogContent {
    pub op: String,
    pub ext: Option<String>,
    pub name: String,
    pub ak: String,
    pub ip: String,
}

impl ToString for LogContent {
    fn to_string(&self) -> String {
        json!({
            "op": self.op,
            "ext": self.ext,
            "name": self.name,
            "ak": self.ak,
            "ip": self.ip,
        })
        .to_string()
    }
}

impl SpiLogClient {
    pub async fn add_item(
        tag: String,
        mut content: LogContent,
        kind: Option<String>,
        key: Option<String>,
        op: Option<String>,
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
        let account = IamAccountServ::get_item(ctx.owner.as_str(), &IamAccountFilterReq::default(), funs, ctx).await?;
        let cert = IamCertServ::find_certs(
            &RbumCertFilterReq {
                basic: RbumBasicFilterReq {
                    own_paths: Some("".to_string()),
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                rel_rbum_id: Some(account.id.clone()),
                ..Default::default()
            },
            None,
            None,
            funs,
            ctx,
        )
        .await?
        .pop();
        content.name = account.name;
        content.ak = if let Some(cert) = cert { cert.ak } else { "".to_string() };
        //add log item
        let mut body = HashMap::from([
            ("tag", tag),
            ("content", content.to_string()),
            ("owner", ctx.owner.clone()),
            ("owner_paths", ctx.own_paths.clone()),
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
        if let Some(op) = op {
            body.insert("op", op);
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
