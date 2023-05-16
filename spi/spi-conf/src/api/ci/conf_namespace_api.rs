use tardis::web::{poem_openapi::{self, payload::Json}, context_extractor::TardisContextExtractor, poem::Request, web_resp::TardisApiResult};

use crate::dto::conf_namespace_dto::*;

pub struct ConfCiConfigServerApi;


/// Interface Console config server API
#[poem_openapi::OpenApi(prefix_path = "/ci/namespace", tag = "bios_basic::ApiTag::Interface")]
impl ConfCiConfigServerApi {
    #[oai(path = "/list", method = "get")]
    async fn get_namespace_list(&self, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Vec<NamespaceItem>> {
        unimplemented!()
    }
    #[oai(path = "/", method = "post")]
    async fn get_namespace(&self, descriptor: Json<NamespaceDescriptor>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<NamespaceItem> {
        unimplemented!()
    }
    #[oai(path = "/", method = "post")]
    async fn create_namespace(&self, attribute: Json<NamespaceAttribute>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<bool> {
        unimplemented!()
    }
    #[oai(path = "/", method = "put")]
    async fn edit_namespace(&self, attribute: Json<NamespaceAttribute>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<bool> {
        unimplemented!()
    }
    #[oai(path = "/", method = "get")]
    async fn delete_namespace(&self, descriptor: Json<NamespaceDescriptor>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<bool> {
        unimplemented!()
    }
}