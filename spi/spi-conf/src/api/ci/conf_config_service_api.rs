use poem::web::RealIp;
use tardis::{
    basic::error::TardisError,
    db::sea_orm::prelude::Uuid,
    web::{
        context_extractor::TardisContextExtractor,
        poem_openapi::{self, param::Query, payload::Json},
        web_resp::{TardisApiResult, TardisResp, Void},
    },
};

use crate::{conf_constants::error, serv::*};
use crate::{
    dto::{conf_config_dto::*, conf_namespace_dto::*},
    serv::placehodler::render_content_for_ip,
};

#[derive(Default, Clone, Copy, Debug)]
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
        real_ip: RealIp,
    ) -> TardisApiResult<String> {
        let namespace_id = namespace_id.0.or(tenant.0).unwrap_or("public".into());
        let tags = tag.0.unwrap_or_default().split(',').map(str::trim).map(String::from).collect();
        let mut descriptor = ConfigDescriptor {
            namespace_id,
            group: group.0,
            data_id: data_id.0,
            tags,
            tp: r#type.0,
        };
        let funs = crate::get_tardis_inst();
        let mut content = get_config(&mut descriptor, &funs, &ctx.0).await?;
        if let Some(ip) = real_ip.0 {
            content = render_content_for_ip(content, ip, &funs, &ctx.0).await?;
        }
        TardisResp::ok(content)
    }
    #[oai(path = "/config/detail", method = "get")]
    async fn get_config_detail(
        &self,
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
        real_ip: RealIp,
    ) -> TardisApiResult<ConfigItem> {
        let namespace_id = namespace_id.0.or(tenant.0).unwrap_or("public".into());
        let tags = tag.0.unwrap_or_default().split(',').map(str::trim).map(String::from).collect();
        let mut descriptor = ConfigDescriptor {
            namespace_id,
            group: group.0,
            data_id: data_id.0,
            tags,
            tp: r#type.0,
        };
        let funs = crate::get_tardis_inst();
        let mut config_item = get_config_detail(&mut descriptor, &funs, &ctx.0).await?;
        if let Some(ip) = real_ip.0 {
            config_item.content = render_content_for_ip(config_item.content, ip, &funs, &ctx.0).await?;
        }
        TardisResp::ok(config_item)
    }
    #[oai(path = "/config", method = "post")]
    async fn publish_config(&self, mut publish_request: Json<ConfigPublishRequest>, ctx: TardisContextExtractor) -> TardisApiResult<bool> {
        let funs = crate::get_tardis_inst();
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
        ctx: TardisContextExtractor,
    ) -> TardisApiResult<Void> {
        let namespace_id = namespace_id.0.or(tenant.0).unwrap_or("public".into());
        let mut descriptor = ConfigDescriptor {
            namespace_id,
            group: group.0,
            data_id: data_id.0,
            ..Default::default()
        };
        let funs = crate::get_tardis_inst();
        delete_config(&mut descriptor, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }
    #[oai(path = "/configs/listener", method = "get")]
    async fn listener(
        &self,
        // mut descriptor: Query<ConfigDescriptor>,
        tenant: Query<Option<NamespaceId>>,
        namespace_id: Query<Option<NamespaceId>>,
        /// 配置分组名
        group: Query<String>,
        /// 配置名
        data_id: Query<String>,
        md5: Query<Option<String>>,
        ctx: TardisContextExtractor,
    ) -> TardisApiResult<Option<ConfigDescriptor>> {
        let mut namespace_id = namespace_id.0.or(tenant.0).unwrap_or("public".into());
        if namespace_id.is_empty() {
            namespace_id = "public".into();
        }
        let mut descriptor = ConfigDescriptor {
            namespace_id,
            group: group.0,
            data_id: data_id.0,
            ..Default::default()
        };
        let md5 = md5.0.unwrap_or_default();
        let funs = crate::get_tardis_inst();
        let config = if md5.is_empty() || md5 != get_md5(&mut descriptor, &funs, &ctx.0).await? {
            // if md5 is empty or changed, return descriptor
            Some(descriptor)
        } else {
            // if md5 is not changed, return none
            None
        };
        TardisResp::ok(config)
    }
    #[oai(path = "/configs", method = "get")]
    async fn get_configs(
        &self,
        namespace_id: Query<Option<NamespaceId>>,
        tenant: Query<Option<NamespaceId>>,
        #[oai(validator(min_length = 1, max_length = 256))] group: Query<Option<String>>,
        /// 配置名
        #[oai(validator(min_length = 1, max_length = 256))]
        data_id: Query<Option<String>>,
        /// 标签
        tags: Query<Option<String>>,
        /// 配置类型
        r#type: Query<Option<String>>,
        page_no: Query<Option<u32>>,
        page_size: Query<Option<u32>>,
        mode: Query<Option<String>>,
        ctx: TardisContextExtractor,
    ) -> TardisApiResult<ConfigListResponse> {
        let funs = crate::get_tardis_inst();
        let page_size = page_size.0.unwrap_or(100).min(500).max(1);
        let page_number = page_no.0.unwrap_or(1).max(0);
        let mode = mode.0.as_deref().unwrap_or_default().into();
        let request = ConfigListRequest {
            namespace_id: namespace_id.0.or(tenant.0),
            group: group.0,
            data_id: data_id.0,
            tags: tags.0.unwrap_or_default().split(',').map(String::from).collect(),
            tp: r#type.0,
            page_no: page_number,
            page_size,
        };
        // let config = get_configs_by_namespace(&namespace_id, &funs, &ctx.0).await?;
        let resp = get_configs(request, mode, &funs, &ctx.0).await?;
        TardisResp::ok(resp)
    }
    #[oai(path = "/history/list", method = "get")]
    async fn history_list(
        &self,
        /// 命名空间
        namespace_id: Query<Option<NamespaceId>>,
        /// 租户
        tenant: Query<Option<NamespaceId>>,
        /// 配置分组名
        group: Query<String>,
        /// 配置名
        data_id: Query<String>,
        /// 页数
        page_no: Query<Option<u32>>,
        /// 每页大小
        page_size: Query<Option<u32>>,
        ctx: TardisContextExtractor,
    ) -> TardisApiResult<ConfigListResponse> {
        let mut namespace_id = namespace_id.0.or(tenant.0).unwrap_or("public".into());
        if namespace_id.is_empty() {
            namespace_id = "public".into();
        }
        let descriptor = ConfigDescriptor {
            namespace_id,
            group: group.0,
            data_id: data_id.0,
            ..Default::default()
        };
        let funs = crate::get_tardis_inst();
        let page_size = page_size.0.unwrap_or(100).min(500).max(1);
        let page_number = page_no.0.unwrap_or(1).max(0);
        let mut request = ConfigHistoryListRequest {
            descriptor,
            page_no: page_number,
            page_size,
        };
        let response = get_history_list_by_namespace(&mut request, &funs, &ctx.0).await?;
        TardisResp::ok(response)
    }
    #[oai(path = "/history", method = "get")]
    async fn history(
        &self,
        /// 命名空间
        namespace_id: Query<Option<NamespaceId>>,
        /// 租户
        tenant: Query<Option<NamespaceId>>,
        /// 配置分组名
        group: Query<String>,
        /// 配置名
        data_id: Query<String>,
        id: Query<String>,
        ctx: TardisContextExtractor,
    ) -> TardisApiResult<ConfigItem> {
        let mut namespace_id = namespace_id.0.or(tenant.0).unwrap_or("public".into());
        if namespace_id.is_empty() {
            namespace_id = "public".into();
        }
        let mut descriptor = ConfigDescriptor {
            namespace_id,
            group: group.0,
            data_id: data_id.0,
            ..Default::default()
        };
        let id = Uuid::parse_str(&id.0).map_err(|e| TardisError::bad_request(&e.to_string(), error::INVALID_UUID))?;

        let funs = crate::get_tardis_inst();
        let config = find_history(&mut descriptor, &id, &funs, &ctx.0).await?;
        TardisResp::ok(config)
    }
    #[oai(path = "/history/previous", method = "get")]
    async fn history_previous(
        &self,
        /// 命名空间
        namespace_id: Query<Option<NamespaceId>>,
        /// 租户
        tenant: Query<Option<NamespaceId>>,
        /// 配置分组名
        group: Query<String>,
        /// 配置名
        data_id: Query<String>,
        id: Query<String>,
        ctx: TardisContextExtractor,
    ) -> TardisApiResult<ConfigItem> {
        let mut namespace_id = namespace_id.0.or(tenant.0).unwrap_or("public".into());
        if namespace_id.is_empty() {
            namespace_id = "public".into();
        }
        let id = Uuid::parse_str(&id.0).map_err(|e| TardisError::bad_request(&e.to_string(), error::INVALID_UUID))?;
        let mut descriptor = ConfigDescriptor {
            namespace_id,
            group: group.0,
            data_id: data_id.0,
            ..Default::default()
        };
        let funs = crate::get_tardis_inst();
        let config = find_previous_history(&mut descriptor, &id, &funs, &ctx.0).await?;
        TardisResp::ok(config)
    }
    #[oai(path = "/history/configs", method = "get")]
    async fn configs_by_namespace(
        &self,
        namespace_id: Query<Option<NamespaceId>>,
        tenant: Query<Option<NamespaceId>>,
        ctx: TardisContextExtractor,
    ) -> TardisApiResult<Vec<ConfigItemDigest>> {
        let mut namespace_id = namespace_id.0.or(tenant.0).unwrap_or("public".into());
        if namespace_id.is_empty() {
            namespace_id = "public".into();
        }
        let funs = crate::get_tardis_inst();
        let config = get_configs_by_namespace(&namespace_id, &funs, &ctx.0).await?;
        TardisResp::ok(config)
    }
}
