use tardis::web::{poem_openapi::{self, payload::Json}, context_extractor::TardisContextExtractor, poem::Request, web_resp::TardisApiResult};

use crate::dto::{conf_config_dto::*, conf_namespace_dto::*};

pub struct ConfCiConfigServerApi;


/// Interface Console config server API
#[poem_openapi::OpenApi(prefix_path = "/ci/cs", tag = "bios_basic::ApiTag::Interface")]
impl ConfCiConfigServerApi {
    #[oai(path = "/config", method = "get")]
    async fn get_config(&self, add_or_modify_req: Json<ConfigDescriptor>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<String> {
        unimplemented!()
    }
    #[oai(path = "/config", method = "post")]
    async fn publish_config(&self, add_or_modify_req: Json<ConfigPublishRequest>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<bool> {
        unimplemented!()
    }
    #[oai(path = "/config", method = "post")]
    async fn delete_config(&self, add_or_modify_req: Json<ConfigDescriptor>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<bool> {
        unimplemented!()
    }
    #[oai(path = "/history/list", method = "get")]
    async fn history_list(&self, add_or_modify_req: Json<ConfigDescriptor>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<ConfigHistoryList> {
        unimplemented!()
    }
    #[oai(path = "/history", method = "get")]
    async fn history(&self, add_or_modify_req: Json<ConfigDescriptor>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<ConfigHistoryItem> {
        unimplemented!()
    }
    #[oai(path = "/history/previous", method = "get")]
    async fn history_previous(&self, add_or_modify_req: Json<ConfigDescriptor>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<ConfigHistoryItem> {
        unimplemented!()
    }
    #[oai(path = "/history/configs", method = "get")]
    async fn history_configs(&self, add_or_modify_req: Json<NamespaceDescriptor>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Vec<ConfigHistoryItem>> {
        unimplemented!()
    }
}