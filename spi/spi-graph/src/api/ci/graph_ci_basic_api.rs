use bios_basic::TardisFunInstExtractor;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem::Request;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::param::Query;
use tardis::web::poem_openapi::payload::Json;
use tardis::web::web_resp::{TardisApiResult, TardisResp, Void};

use crate::dto::graph_dto::{GraphNodeVersionResp, GraphRelAddReq, GraphRelDetailResp, GraphRelUpgardeVersionReq};
use crate::serv::graph_basic_serv;

pub struct GraphCiRelApi;

/// Interface Console Graph API
#[poem_openapi::OpenApi(prefix_path = "/ci", tag = "bios_basic::ApiTag::Interface")]
impl GraphCiRelApi {
    /// Add Rel
    #[oai(path = "/rel", method = "put")]
    async fn add_rel(&self, add_or_modify_req: Json<GraphRelAddReq>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        let funs = request.tardis_fun_inst();
        graph_basic_serv::add_rel(&add_or_modify_req.0, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    /// Upgrade Version
    #[oai(path = "/version", method = "put")]
    async fn upgrade_version(&self, upgrade_version_req: Json<GraphRelUpgardeVersionReq>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        let funs = request.tardis_fun_inst();
        graph_basic_serv::upgrade_version(&upgrade_version_req.0, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    /// Find Versions
    #[oai(path = "/versions", method = "get")]
    async fn find_versions(&self, tag: Query<String>, key: Query<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Vec<GraphNodeVersionResp>> {
        let funs = request.tardis_fun_inst();
        let resp = graph_basic_serv::find_versions(tag.0, key.0, &funs, &ctx.0).await?;
        TardisResp::ok(resp)
    }

    /// Delete Rel
    #[oai(path = "/rel", method = "delete")]
    async fn delete_rels(
        &self,
        tag: Query<String>,
        from_key: Query<Option<String>>,
        to_key: Query<Option<String>>,
        from_version: Query<Option<String>>,
        to_version: Query<Option<String>>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<Void> {
        let funs = request.tardis_fun_inst();
        graph_basic_serv::delete_rels(tag.0, from_key.0, to_key.0, from_version.0, to_version.0, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    /// Find Rels
    #[oai(path = "/rels", method = "get")]
    async fn find_rels(
        &self,
        from_key: Query<String>,
        from_version: Query<String>,
        depth: Query<Option<u8>>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<GraphRelDetailResp> {
        let funs = request.tardis_fun_inst();
        let resp = graph_basic_serv::find_rels(from_key.0, from_version.0, depth.0, &funs, &ctx.0).await?;
        TardisResp::ok(resp)
    }
}
