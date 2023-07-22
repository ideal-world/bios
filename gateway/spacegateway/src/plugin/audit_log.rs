use std::{collections::HashMap, mem, str::FromStr, sync::Arc};

use async_trait::async_trait;
use bios_auth::{
    auth_config::AuthConfig,
    auth_initializer,
    dto::{
        auth_crypto_dto::AuthEncryptReq,
        auth_kernel_dto::{AuthReq, AuthResp},
    },
    serv::{auth_crypto_serv, auth_kernel_serv},
};
use bios_sdk_invoke::invoke_initializer;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use spacegate_kernel::{
    config::http_route_dto::SgHttpRouteRule,
    functions::http_route::SgHttpRouteMatchInst,
    http::{self, HeaderMap, HeaderName, HeaderValue},
    plugins::{
        context::{SgRouteFilterRequestAction, SgRoutePluginContext},
        filters::{BoxSgPluginFilter, SgPluginFilter, SgPluginFilterAccept, SgPluginFilterDef, SgPluginFilterInitDto},
    },
};
use tardis::{
    async_trait,
    basic::{error::TardisError, result::TardisResult},
    config::config_dto::{AppConfig, CacheConfig, FrameworkConfig, TardisConfig, WebServerConfig, WebServerModuleConfig},
    log,
    serde_json::{self, Value},
    tokio::{self, sync::Mutex, task::JoinHandle},
    TardisFuns,
};

pub const CODE: &str = "audit_log";
pub struct SgFilterAuditLogDef;

impl SgPluginFilterDef for SgFilterAuditLogDef {
    fn inst(&self, spec: serde_json::Value) -> TardisResult<BoxSgPluginFilter> {
        let filter = TardisFuns::json.json_to_obj::<SgFilterAuditLog>(spec)?;
        Ok(filter.boxed())
    }
}

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct SgFilterAuditLog {
    pub log_url: String,
}

impl Default for SgFilterAuditLog {
    fn default() -> Self {
        Self {log_url:"".to_string()}
    }
}

#[async_trait]
impl SgPluginFilter for SgFilterAuditLog {
    fn accept(&self) -> SgPluginFilterAccept {
        SgPluginFilterAccept::default()
    }

    async fn init(&self, _: &SgPluginFilterInitDto) -> TardisResult<()> {

        invoke_initializer::init(funs.module_code(), funs.conf::<IamConfig>().invoke.clone())?;
        Ok(())
    }

    async fn destroy(&self) -> TardisResult<()> {
        Ok(())
    }

    async fn req_filter(&self, _: &str, mut ctx: SgRoutePluginContext, _matched_match_inst: Option<&SgHttpRouteMatchInst>) -> TardisResult<(bool, SgRoutePluginContext)> {
        return Ok((true, ctx));
    }

    async fn resp_filter(&self, _: &str, mut ctx: SgRoutePluginContext, _: Option<&SgHttpRouteMatchInst>) -> TardisResult<(bool, SgRoutePluginContext)> {
        Ok((true, ctx))
    }
}
