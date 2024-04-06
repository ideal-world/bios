use poem::web::RealIp;
use tardis::web::{
    poem::{self, web::Form, Request},
    poem_openapi::{self, param::Query, payload::Json},
};

use super::tardis_err_to_poem_err;
use crate::serv::{placeholder::render_content_for_ip, *};
use crate::{
    api::nacos::extract_context,
    dto::{conf_config_dto::*, conf_config_nacos_dto::*, conf_namespace_dto::*},
};

#[derive(Default, Clone, Copy, Debug)]
pub struct ConfNacosV2CsApi;
type NacosResult<T> = poem::Result<Json<NacosResponse<T>>>;
/// Interface Console config server API
#[poem_openapi::OpenApi(prefix_path = "/nacos/v2/cs", tag = "bios_basic::ApiTag::Interface")]
impl ConfNacosV2CsApi {
    #[oai(path = "/configs", method = "get")]
    async fn get_config(
        &self,
        // mut descriptor: Query<ConfigDescriptor>,
        tenant: Query<Option<NamespaceId>>,
        #[oai(name = "namespaceId")] namespace_id: Query<Option<NamespaceId>>,
        /// 配置分组名
        group: Query<String>,
        /// 配置名
        #[oai(name = "dataId")]
        data_id: Query<String>,
        tag: Query<Option<String>>,
        request: &Request,
        real_ip: RealIp,
    ) -> NacosResult<String> {
        let namespace_id = namespace_id.0.or(tenant.0).unwrap_or("public".into());
        let tags = tag.0.map(|tag| tag.split(',').map(String::from).collect::<Vec<_>>()).unwrap_or_default();
        let mut descriptor = ConfigDescriptor {
            namespace_id,
            group: group.0,
            data_id: data_id.0,
            tags,
            ..Default::default()
        };
        let funs = crate::get_tardis_inst();
        let ctx = extract_context(request).await?;
        let mut content = get_config(&mut descriptor, &funs, &ctx).await.map_err(tardis_err_to_poem_err)?;
        if let Some(ip) = real_ip.0 {
            content = render_content_for_ip(content, ip, &funs, &ctx).await?;
        }
        Ok(Json(NacosResponse::ok(content)))
    }
    #[oai(path = "/configs", method = "post")]
    async fn publish_config(&self, form: Form<PublishConfigFormV2>, request: &Request) -> NacosResult<bool> {
        let funs = crate::get_tardis_inst();
        let ctx = extract_context(request).await?;
        let mut publish_request = form.0.into();
        let success = publish_config(&mut publish_request, &funs, &ctx).await?;
        Ok(Json(NacosResponse::ok(success)))
    }
    #[oai(path = "/configs", method = "delete")]
    async fn delete_config(
        &self,
        tenant: Query<Option<NamespaceId>>,
        #[oai(name = "namespaceId")] namespace_id: Query<Option<NamespaceId>>,
        /// 配置分组名
        group: Query<String>,
        /// 配置名
        #[oai(name = "dataId")]
        data_id: Query<String>,
        request: &Request,
    ) -> NacosResult<bool> {
        let namespace_id = namespace_id.0.or(tenant.0).unwrap_or("public".into());
        let mut descriptor = ConfigDescriptor {
            namespace_id,
            group: group.0,
            data_id: data_id.0,
            ..Default::default()
        };
        let funs = crate::get_tardis_inst();
        let ctx = extract_context(request).await?;
        let success = delete_config(&mut descriptor, &funs, &ctx).await?;
        Ok(Json(NacosResponse::ok(success)))
    }
}
