use bios_basic::spi::spi_funs::SpiTardisFunInstExtractor;
use tardis::web::{
    context_extractor::TardisContextExtractor,
    poem::Request,
    poem_openapi::{self, param::Query, payload::Json},
    web_resp::{TardisApiResult, TardisResp, Void},
};

use crate::dto::{conf_config_dto::*, conf_namespace_dto::*};
use crate::serv::*;

pub struct ConfCiConfigServiceApi;

/// Interface Console config server API
#[poem_openapi::OpenApi(prefix_path = "/ci/cs", tag = "bios_basic::ApiTag::Interface")]
impl ConfCiConfigServiceApi {
    #[oai(path = "/config", method = "get")]
    async fn get_config(
        &self,
        // mut descriptor: Query<ConfigDescriptor>,
        tenant: Query<Option<NamespaceId>>,
        namespace_id: Query<Option<NamespaceId>>,
        /// 配置分组名
        group: Query<String>,
        /// 配置名
        data_id: Query<String>,
        /// 标签
        tag: Query<Option<String>>,
        /// 配置类型
        r#type: Query<Option<String>>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<String> {
        let namespace_id = namespace_id.0.or(tenant.0).unwrap_or("public".into());
        let mut descriptor = ConfigDescriptor {
            namespace_id,
            group: group.0,
            data_id: data_id.0,
            tag: tag.0,
            tp: r#type.0,
        };
        let funs = request.tardis_fun_inst();
        let content = get_config(&mut descriptor, &funs, &ctx.0).await?;
        TardisResp::ok(content)
    }
    #[oai(path = "/config", method = "post")]
    async fn publish_config(&self, mut publish_request: Json<ConfigPublishRequest>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<bool> {
        let funs = request.tardis_fun_inst();
        let item = publish_config(&mut publish_request.0, &funs, &ctx.0).await?;
        TardisResp::ok(item)
    }
    #[oai(path = "/config", method = "delete")]
    async fn delete_config(
        &self,
        // mut descriptor: Query<ConfigDescriptor>,
        tenant: Query<Option<NamespaceId>>,
        namespace_id: Query<Option<NamespaceId>>,
        /// 配置分组名
        group: Query<String>,
        /// 配置名
        data_id: Query<String>,
        /// 标签
        tag: Query<Option<String>>,
        /// 配置类型
        r#type: Query<Option<String>>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<Void> {
        let namespace_id = namespace_id.0.or(tenant.0).unwrap_or("public".into());
        let mut descriptor = ConfigDescriptor {
            namespace_id,
            group: group.0,
            data_id: data_id.0,
            tag: tag.0,
            tp: r#type.0,
        };
        let funs = request.tardis_fun_inst();
        let result = delete_config(&mut descriptor, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }
    #[oai(path = "/configs/listener", method = "get")]
    async fn listenr(
        &self,
        // mut descriptor: Query<ConfigDescriptor>,
        tenant: Query<Option<NamespaceId>>,
        namespace_id: Query<Option<NamespaceId>>,
        /// 配置分组名
        group: Query<String>,
        /// 配置名
        data_id: Query<String>,
        /// 标签
        tag: Query<Option<String>>,
        /// 配置类型
        r#type: Query<Option<String>>,
        md5: Query<Option<String>>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<String> {
        let namespace_id = namespace_id.0.or(tenant.0).unwrap_or("public".into());
        let mut descriptor = ConfigDescriptor {
            namespace_id,
            group: group.0,
            data_id: data_id.0,
            tag: tag.0,
            tp: r#type.0,
        };
        let md5 = md5.0.unwrap_or_default();
        let funs = request.tardis_fun_inst();
        let config = if md5.is_empty() && md5 != get_md5(&mut descriptor, &funs, &ctx.0).await? {
            // if md5 is empty or changed, return config
            get_config(&mut descriptor, &funs, &ctx.0).await?
        } else {
            // if md5 is not changed, return empty string
            String::new()
        };
        TardisResp::ok(config)
    }
    #[oai(path = "/history/list", method = "get")]
    async fn history_list(&self, add_or_modify_req: Json<ConfigDescriptor>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<ConfigHistoryList> {
        unimplemented!()
    }
    #[oai(path = "/history", method = "get")]
    async fn history(&self, add_or_modify_req: Json<ConfigDescriptor>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<ConfigItem> {
        unimplemented!()
    }
    #[oai(path = "/history/previous", method = "get")]
    async fn history_previous(&self, add_or_modify_req: Json<ConfigDescriptor>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<ConfigItem> {
        unimplemented!()
    }
    #[oai(path = "/history/configs", method = "get")]
    async fn history_configs(&self, add_or_modify_req: Json<NamespaceDescriptor>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Vec<ConfigItem>> {
        unimplemented!()
    }
}
