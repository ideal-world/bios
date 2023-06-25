use bios_basic::TardisFunInstExtractor;
use tardis::{
    basic::error::TardisError,
    db::sea_orm::prelude::Uuid,
    serde_json::{self, Value},
    web::{
        context_extractor::TardisContextExtractor,
        poem::{self, web::Form, Request},
        poem_openapi::{
            self,
            param::Query,
            payload::{Json, PlainText},
        },
        web_resp::{TardisApiResult, TardisResp},
    },
};

use crate::dto::{conf_config_dto::*, conf_config_nacos_dto::PublishConfigForm, conf_namespace_dto::*};
use crate::{conf_constants::error, serv::*};

use super::tardis_err_to_poem_err;

#[derive(Default)]
pub struct ConfNacosV1NamespaceApi;

/// Interface Console config server API
#[poem_openapi::OpenApi(prefix_path = "/nacos/v1/console/namespaces", tag = "bios_basic::ApiTag::Interface")]
impl ConfNacosV1NamespaceApi {
    #[oai(path = "/list", method = "get")]
    /// because of nacos counterpart api's response is not paged, this api won't be paged either
    async fn get_namespace_list(&self, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Vec<NamespaceItem>> {
        let funs = request.tardis_fun_inst();
        let items = get_namespace_list(&funs, &ctx.0).await?;
        TardisResp::ok(items)
    }
    #[oai(path = "/", method = "get")]
    async fn get_namespace(&self,
        namespace_id: Query<NamespaceId>, ctx: TardisContextExtractor, request: &Request
    ) -> poem::Result<Json<NamespaceItem>> {
        let funs = request.tardis_fun_inst();
        let mut descriptor = NamespaceDescriptor { namespace_id: namespace_id.0 };
        let item = get_namespace(&mut descriptor, &funs, &ctx.0).await?;
        Ok(Json(item))
    }
    #[oai(path = "/", method = "post")]
    async fn create_namespace(&self, 
        #[oai(name="customNamespaceId")]
        namespace: Query<NamespaceId>,
        #[oai(name="namespaceName")]
        namespace_show_name: Query<String>,
        #[oai(name="namespaceDesc")]
        namespace_desc: Query<Option<String>>,
        #[oai(name = "accessToken")] 
        access_token: Query<String>,
        ctx: TardisContextExtractor, 
        request: &Request
    ) -> TardisApiResult<bool> {
        let funs = request.tardis_fun_inst();

        let mut attribute = NamespaceAttribute {
            namespace: namespace.0,
            namespace_show_name: namespace_show_name.0,
            namespace_desc: namespace_desc.0,
        };
        create_namespace(&mut attribute, &funs, &ctx.0).await?;
        TardisResp::ok(true)
    }
    #[oai(path = "/", method = "put")]
    async fn edit_namespace(&self, 
        #[oai(name="customNamespaceId")]
        namespace: Query<NamespaceId>,
        #[oai(name="namespaceName")]
        namespace_show_name: Query<String>,
        #[oai(name="namespaceDesc")]
        namespace_desc: Query<Option<String>>,
        #[oai(name = "accessToken")] 
        access_token: Query<String>,
        ctx: TardisContextExtractor, 
        request: &Request
    ) -> TardisApiResult<bool> {
        let funs = request.tardis_fun_inst();
        let mut attribute = NamespaceAttribute {
            namespace: namespace.0,
            namespace_show_name: namespace_show_name.0,
            namespace_desc: namespace_desc.0,
        };
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
