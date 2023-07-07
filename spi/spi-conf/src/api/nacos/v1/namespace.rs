use tardis::web::{
    poem::{self, web::Form},
    poem_openapi::{self, param::Query, payload::Json},
};

use crate::dto::{
    conf_config_nacos_dto::{NacosCreateNamespaceRequest, NacosDeleteNamespaceRequest, NacosEditNamespaceRequest, NacosResponse},
    conf_namespace_dto::*,
};
use crate::serv::*;

#[derive(Default, Clone, Copy, Debug)]
pub struct ConfNacosV1NamespaceApi;

/// Interface Console config server API
#[poem_openapi::OpenApi(prefix_path = "/nacos/v1/console/namespaces", tag = "bios_basic::ApiTag::Interface")]
impl ConfNacosV1NamespaceApi {
    #[oai(path = "/", method = "get")]
    /// because of nacos counterpart api's response is not paged, this api won't be paged either
    async fn get_namespace_list(&self, #[oai(name = "accessToken")] access_token: Query<String>) -> poem::Result<Json<NacosResponse<Vec<NamespaceItem>>>> {
        let funs = crate::get_tardis_inst();
        let ctx = jwt_validate(&access_token.0, &funs).await?;
        let items = get_namespace_list(&funs, &ctx).await?;
        Ok(Json(NacosResponse::ok(items)))
    }
    #[oai(path = "/", method = "post")]
    async fn create_namespace(&self, form: Form<NacosCreateNamespaceRequest>, #[oai(name = "accessToken")] access_token: Query<String>) -> poem::Result<Json<bool>> {
        let funs = crate::get_tardis_inst();
        let ctx = jwt_validate(&access_token.0, &funs).await?;
        let mut attribute = form.0.into();
        create_namespace(&mut attribute, &funs, &ctx).await?;
        Ok(Json(true))
    }
    #[oai(path = "/", method = "put")]
    async fn edit_namespace(&self, form: Form<NacosEditNamespaceRequest>, #[oai(name = "accessToken")] access_token: Query<String>) -> poem::Result<Json<bool>> {
        let funs = crate::get_tardis_inst();
        let ctx = jwt_validate(&access_token.0, &funs).await?;
        let mut attribute = form.0.into();
        edit_namespace(&mut attribute, &funs, &ctx).await?;
        Ok(Json(true))
    }
    #[oai(path = "/", method = "delete")]
    async fn delete_namespace(&self, form: Form<NacosDeleteNamespaceRequest>, #[oai(name = "accessToken")] access_token: Query<String>) -> poem::Result<Json<bool>> {
        let funs = crate::get_tardis_inst();
        let ctx = jwt_validate(&access_token.0, &funs).await?;
        let mut descriptor = NamespaceDescriptor { namespace_id: form.0.namespaceId };
        delete_namespace(&mut descriptor, &funs, &ctx).await?;
        Ok(Json(true))
    }
}
