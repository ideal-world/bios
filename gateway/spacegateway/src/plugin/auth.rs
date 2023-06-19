use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use spacegate_kernel::{
    config::http_route_dto::SgHttpRouteRule,
    functions::http_route::SgHttpRouteMatchInst,
    plugins::{
        context::SgRoutePluginContext,
        filters::{BoxSgPluginFilter, SgPluginFilter, SgPluginFilterDef, SgPluginFilterKind},
    },
};
use tardis::{async_trait, basic::result::TardisResult, serde_json, TardisFuns};

pub const CODE: &str = "auth";
pub struct SgFilterAuthDef;

impl SgPluginFilterDef for SgFilterAuthDef {
    fn inst(&self, spec: serde_json::Value) -> TardisResult<BoxSgPluginFilter> {
        let filter = TardisFuns::json.json_to_obj::<SgFilterAuth>(spec)?;
        Ok(filter.boxed())
    }
}

#[derive(Serialize, Deserialize)]
pub struct SgFilterAuth {
    is_enabled: bool,
    title: Option<String>,
    msg: Option<String>,
}

#[async_trait]
impl SgPluginFilter for SgFilterAuth {
    fn kind(&self) -> SgPluginFilterKind {
        SgPluginFilterKind::Http
    }

    async fn init(&self, _: &[SgHttpRouteRule]) -> TardisResult<()> {
        Ok(())
    }

    async fn destroy(&self) -> TardisResult<()> {
        Ok(())
    }

    async fn req_filter(&self, _: &str, mut ctx: SgRoutePluginContext, _matched_match_inst: Option<&SgHttpRouteMatchInst>) -> TardisResult<(bool, SgRoutePluginContext)> {
        Ok((true, ctx))
    }

    async fn resp_filter(&self, _: &str, ctx: SgRoutePluginContext, _: Option<&SgHttpRouteMatchInst>) -> TardisResult<(bool, SgRoutePluginContext)> {
        Ok((true, ctx))
    }
}
