use tardis::web::{
    poem::{self, web::Form},
    poem_openapi::{self, param::Query, payload::Json},
};

use crate::dto::{
    conf_config_nacos_dto::{NacosCreateNamespaceRequest, NacosDeleteNamespaceRequest, NacosEditNamespaceRequest, NacosResponse, NamespaceItemNacos},
    conf_namespace_dto::*,
};
use crate::serv::*;

#[derive(Default, Clone, Copy, Debug)]
pub struct ConfNacosV2NamespaceApi;
type NacosResult<T> = poem::Result<Json<NacosResponse<T>>>;

/// Interface Console config server API
#[poem_openapi::OpenApi(prefix_path = "/nacos/v2/console/namespace", tag = "bios_basic::ApiTag::Interface")]
impl ConfNacosV2NamespaceApi {
    #[oai(path = "/list", method = "get")]
    /// because of nacos counterpart api's response is not paged, this api won't be paged either
    async fn get_namespace_list(&self, #[oai(name = "accessToken")] access_token: Query<String>) -> NacosResult<Vec<NamespaceItemNacos>> {
        let funs = crate::get_tardis_inst();
        let ctx = jwt_validate(&access_token.0, &funs).await?;
        let items = get_namespace_list(&funs, &ctx).await?.into_iter().map(NamespaceItemNacos::from).collect();
        Ok(Json(NacosResponse::ok(items)))
    }
    #[oai(path = "/", method = "get")]
    /// because of nacos counterpart api's response is not paged, this api won't be paged either
    async fn get_namespace(
        &self,
        #[oai(name = "namespaceId")] namespace_id: Query<Option<NamespaceId>>,
        #[oai(name = "accessToken")] access_token: Query<String>,
    ) -> NacosResult<NamespaceItemNacos> {
        let funs = crate::get_tardis_inst();
        let ctx = jwt_validate(&access_token.0, &funs).await?;
        let item = get_namespace(
            &mut NamespaceDescriptor {
                namespace_id: namespace_id.0.unwrap_or("public".into()),
            },
            &funs,
            &ctx,
        )
        .await?;
        Ok(Json(NacosResponse::ok(item.into())))
    }
    #[oai(path = "/", method = "post")]
    async fn create_namespace(&self, form: Form<NacosCreateNamespaceRequest>, #[oai(name = "accessToken")] access_token: Query<String>) -> NacosResult<bool> {
        let funs = crate::get_tardis_inst();
        let ctx = jwt_validate(&access_token.0, &funs).await?;
        let mut attribute = form.0.into();
        create_namespace(&mut attribute, &funs, &ctx).await?;
        Ok(Json(NacosResponse::ok(true)))
    }
    #[oai(path = "/", method = "put")]
    async fn edit_namespace(&self, form: Form<NacosEditNamespaceRequest>, #[oai(name = "accessToken")] access_token: Query<String>) -> NacosResult<bool> {
        let funs = crate::get_tardis_inst();
        let ctx = jwt_validate(&access_token.0, &funs).await?;
        let mut attribute = form.0.into();
        edit_namespace(&mut attribute, &funs, &ctx).await?;
        Ok(Json(NacosResponse::ok(true)))
    }
    #[oai(path = "/", method = "delete")]
    async fn delete_namespace(&self, form: Form<NacosDeleteNamespaceRequest>, #[oai(name = "accessToken")] access_token: Query<String>) -> NacosResult<bool> {
        let funs = crate::get_tardis_inst();
        let ctx = jwt_validate(&access_token.0, &funs).await?;
        let mut descriptor = NamespaceDescriptor { namespace_id: form.0.namespaceId };
        delete_namespace(&mut descriptor, &funs, &ctx).await?;
        Ok(Json(NacosResponse::ok(true)))
    }
}
