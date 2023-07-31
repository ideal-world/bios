use tardis::web::{
    poem::{self, web::Form, Request},
    poem_openapi::{self, payload::Json},
};

use crate::{
    api::nacos::extract_context,
    dto::{
        conf_config_nacos_dto::{NacosCreateNamespaceRequest, NacosDeleteNamespaceRequest, NacosEditNamespaceRequest, NacosResponse},
        conf_namespace_dto::*,
    },
};
use crate::{api::nacos::extract_context_from_body, serv::*};

#[derive(Default, Clone, Copy, Debug)]
pub struct ConfNacosV1NamespaceApi;

/// Interface Console config server API
#[poem_openapi::OpenApi(prefix_path = "/nacos/v1/console/namespaces", tag = "bios_basic::ApiTag::Interface")]
impl ConfNacosV1NamespaceApi {
    #[oai(path = "/", method = "get")]
    /// because of nacos counterpart api's response is not paged, this api won't be paged either
    async fn get_namespace_list(&self, request: &Request) -> poem::Result<Json<NacosResponse<Vec<NamespaceItem>>>> {
        let funs = crate::get_tardis_inst();
        let ctx = extract_context(request).await?;
        let items = get_namespace_list(&funs, &ctx).await?;
        Ok(Json(NacosResponse::ok(items)))
    }
    #[oai(path = "/", method = "post")]
    async fn create_namespace(&self, form: Form<NacosCreateNamespaceRequest>, request: &Request) -> poem::Result<Json<bool>> {
        let funs = crate::get_tardis_inst();
        let ctx = extract_context_from_body(&form.0).await.unwrap_or(extract_context(request).await)?;
        let mut attribute = form.0.into();
        create_namespace(&mut attribute, &funs, &ctx).await?;
        Ok(Json(true))
    }
    #[oai(path = "/", method = "put")]
    async fn edit_namespace(&self, form: Form<NacosEditNamespaceRequest>, request: &Request) -> poem::Result<Json<bool>> {
        let funs = crate::get_tardis_inst();
        let ctx = extract_context_from_body(&form.0).await.unwrap_or(extract_context(request).await)?;
        let mut attribute = form.0.into();
        edit_namespace(&mut attribute, &funs, &ctx).await?;
        Ok(Json(true))
    }
    #[oai(path = "/", method = "delete")]
    async fn delete_namespace(&self, form: Form<NacosDeleteNamespaceRequest>, request: &Request) -> poem::Result<Json<bool>> {
        let funs = crate::get_tardis_inst();
        let ctx = extract_context_from_body(&form.0).await.unwrap_or(extract_context(request).await)?;
        let mut descriptor = NamespaceDescriptor { namespace_id: form.0.namespaceId };
        delete_namespace(&mut descriptor, &funs, &ctx).await?;
        Ok(Json(true))
    }
}
