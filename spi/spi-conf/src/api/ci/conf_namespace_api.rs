use bios_basic::TardisFunInstExtractor;
use tardis::web::{
    context_extractor::TardisContextExtractor,
    poem::Request,
    poem_openapi::{self, param::Query, payload::Json},
    web_resp::{TardisApiResult, TardisResp, Void},
};

use crate::dto::conf_namespace_dto::*;
use crate::serv::*;
#[derive(Default)]
pub struct ConfCiNamespaceApi;

/// Interface Console config server API
#[poem_openapi::OpenApi(prefix_path = "/ci/namespace", tag = "bios_basic::ApiTag::Interface")]
impl ConfCiNamespaceApi {
    #[oai(path = "/list", method = "get")]
    /// because of nacos counterpart api's response is not paged, this api won't be paged either
    async fn get_namespace_list(&self, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Vec<NamespaceItem>> {
        let funs = request.tardis_fun_inst();
        let items = get_namespace_list(&funs, &ctx.0).await?;
        TardisResp::ok(items)
    }
    #[oai(path = "/", method = "get")]
    async fn get_namespace(&self, namespace_id: Query<NamespaceId>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<NamespaceItem> {
        let funs = request.tardis_fun_inst();
        let mut descriptor = NamespaceDescriptor { namespace_id: namespace_id.0 };
        let item = get_namespace(&mut descriptor, &funs, &ctx.0).await?;
        TardisResp::ok(item)
    }
    #[oai(path = "/", method = "post")]
    async fn create_namespace(&self, mut attribute: Json<NamespaceAttribute>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<bool> {
        let funs = request.tardis_fun_inst();
        create_namespace(&mut attribute.0, &funs, &ctx.0).await?;
        TardisResp::ok(true)
    }
    #[oai(path = "/", method = "put")]
    async fn edit_namespace(&self, mut attribute: Json<NamespaceAttribute>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<bool> {
        let funs = request.tardis_fun_inst();
        edit_namespace(&mut attribute.0, &funs, &ctx.0).await?;
        TardisResp::ok(true)
    }
    #[oai(path = "/", method = "delete")]
    async fn delete_namespace(&self, namespace_id: Query<NamespaceId>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        let funs = request.tardis_fun_inst();
        let mut descriptor = NamespaceDescriptor { namespace_id: namespace_id.0 };
        delete_namespace(&mut descriptor, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }
}
