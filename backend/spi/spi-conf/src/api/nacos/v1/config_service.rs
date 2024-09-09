use poem::web::RealIp;
use tardis::{
    basic::error::TardisError,
    db::sea_orm::prelude::Uuid,
    serde_json::{self, Value},
    web::{
        poem::{self, web::Form, Request},
        poem_openapi::{
            self,
            param::Query,
            payload::{Json, PlainText},
        },
    },
};

use crate::{
    api::nacos::{extract_context, extract_context_from_body},
    dto::{conf_config_dto::*, conf_config_nacos_dto::PublishConfigForm, conf_namespace_dto::*},
    serv::placeholder::render_content_for_ip,
};
use crate::{conf_constants::error, serv::*};

use super::{missing_param, tardis_err_to_poem_err};

#[derive(Default, Clone, Copy, Debug)]
pub struct ConfNacosV1CsApi;

/// Interface Console config server API
#[poem_openapi::OpenApi(prefix_path = "/nacos/v1/cs", tag = "bios_basic::ApiTag::Interface")]
impl ConfNacosV1CsApi {
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
        request: &Request,
        real_ip: RealIp,
    ) -> poem::Result<PlainText<String>> {
        let namespace_id = namespace_id.0.or(tenant.0).unwrap_or("public".into());
        let mut descriptor = ConfigDescriptor {
            namespace_id,
            group: group.0,
            data_id: data_id.0,
            ..Default::default()
        };
        let funs = crate::get_tardis_inst();
        let ctx = extract_context(request).await?;
        let mut content = get_config(&mut descriptor, &funs, &ctx).await.map_err(tardis_err_to_poem_err)?;
        content = render_content_for_ip(&descriptor, content, real_ip.0, &funs, &ctx).await?;
        Ok(PlainText(content))
    }
    #[oai(path = "/configs", method = "post")]
    async fn publish_config(
        &self,
        tenant: Query<Option<NamespaceId>>,
        #[oai(name = "namespaceId")] namespace_id: Query<Option<NamespaceId>>,
        /// 配置分组名
        group: Query<Option<String>>,
        /// 配置名
        #[oai(name = "dataId")]
        data_id: Query<Option<String>>,
        content: Query<Option<String>>,
        r#type: Query<Option<String>>,
        form: Option<Form<PublishConfigForm>>,
        request: &Request,
    ) -> poem::Result<Json<bool>> {
        let funs = crate::get_tardis_inst();
        let namespace_id = form.as_ref().and_then(|f| f.0.tenant.clone()).or(tenant.0).or(namespace_id.0).unwrap_or("public".into());
        let ctx = if let Some(form) = &form {
            extract_context_from_body(&form.0).await.unwrap_or(extract_context(request).await)?
        } else {
            extract_context(request).await?
        };
        let src_user = &ctx.owner;
        let descriptor = ConfigDescriptor {
            namespace_id,
            group: form.as_ref().and_then(|f| f.0.group.clone()).or(group.0).ok_or_else(|| missing_param("group"))?,
            data_id: form.as_ref().and_then(|f| f.0.dataId.clone()).or(data_id.0).ok_or_else(|| missing_param("data_id"))?,
            ..Default::default()
        };
        let mut publish_request = ConfigPublishRequest {
            descriptor,
            schema: r#type.0,
            content: form.and_then(|f| f.0.content).or(content.0).ok_or_else(|| missing_param("content"))?,
            src_user: Some(src_user.clone()),
            ..Default::default()
        };
        let success = publish_config(&mut publish_request, &funs, &ctx).await?;
        Ok(Json(success))
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
    ) -> poem::Result<Json<bool>> {
        let namespace_id = namespace_id.0.or(tenant.0).unwrap_or("public".into());
        let mut descriptor = ConfigDescriptor {
            namespace_id,
            group: group.0,
            data_id: data_id.0,
            ..Default::default()
        };
        let funs = crate::get_tardis_inst();
        let ctx = extract_context(request).await?;
        let result = delete_config(&mut descriptor, &funs, &ctx).await;
        match result {
            Ok(success) => Ok(Json(success)),
            Err(e) => {
                // 未找到也返回200
                if e.code == "404" {
                    Ok(Json(false))
                } else {
                    Err(e.into())
                }
            }
        }
    }
    #[oai(path = "/configs/listener", method = "post")]
    async fn listener(
        &self,
        // mut descriptor: Query<ConfigDescriptor>,
        // Listening-Configs
        #[oai(name = "Listening-Configs")] listening_configs: Query<String>,
        request: &Request,
        real_ip: RealIp,
    ) -> poem::Result<PlainText<String>> {
        let listening_configs = listening_configs.0;
        let funs = crate::get_tardis_inst();
        let ctx = extract_context(request).await?;
        let err_missing = |msg: &str| poem::Error::from_string(format!("missing field {msg}"), poem::http::StatusCode::BAD_REQUEST);
        let mut config_fields = listening_configs.trim_end_matches(1 as char).split(2 as char);
        let data_id = config_fields.next().ok_or_else(|| err_missing("data_id"))?;
        let group = config_fields.next().ok_or_else(|| err_missing("group"))?;
        let md5 = config_fields.next().ok_or_else(|| err_missing("contentMD5"))?;
        let tenant = config_fields.next().unwrap_or("public");
        let mut descriptor = ConfigDescriptor {
            namespace_id: tenant.to_owned(),
            group: group.to_owned(),
            data_id: data_id.to_owned(),
            ..Default::default()
        };
        let config = if md5.is_empty() || md5 != get_md5(&mut descriptor, real_ip.0, &funs, &ctx).await? {
            // if md5 is empty or changed, return listening_configs
            listening_configs
        } else {
            // if md5 is not changed, return none
            String::default()
        };
        Ok(PlainText(config))
    }
    #[oai(path = "/history", method = "get")]
    async fn history(
        &self,
        /// 租户
        tenant: Query<Option<NamespaceId>>,
        /// 配置分组名
        group: Query<String>,
        /// 配置名
        #[oai(name = "dataId")]
        data_id: Query<String>,
        nid: Query<String>,
        search: Query<Option<String>>,
        /// 页数
        #[oai(name = "pageNo")]
        page_no: Query<Option<u32>>,
        /// 每页大小
        #[oai(name = "pageSize")]
        page_size: Query<Option<u32>>,
        request: &Request,
    ) -> poem::Result<Json<Value>> {
        let mut namespace_id = tenant.0.unwrap_or("public".into());
        if namespace_id.is_empty() {
            namespace_id = "public".into();
        }
        let mut descriptor = ConfigDescriptor {
            namespace_id,
            group: group.0,
            data_id: data_id.0,
            ..Default::default()
        };
        let funs = crate::get_tardis_inst();
        let ctx = extract_context(request).await?;
        if let Some("accurate") = search.0.as_deref() {
            let page_size = page_size.0.unwrap_or(100).min(500).max(1);
            let page_no = page_no.0.unwrap_or(1).max(0);
            let mut request = ConfigHistoryListRequest { descriptor, page_no, page_size };
            let response = get_history_list_by_namespace(&mut request, &funs, &ctx).await?;
            Ok(Json(serde_json::to_value(response).expect("fail to convert ConfigListResponse to json value")))
        } else {
            let id = Uuid::parse_str(&nid.0).map_err(|e| TardisError::bad_request(&e.to_string(), error::INVALID_UUID))?;
            let config = find_history(&mut descriptor, &id, &funs, &ctx).await?;
            Ok(Json(serde_json::to_value(config).expect("fail to convert ConfigItem to json value")))
        }
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
        request: &Request,
    ) -> poem::Result<Json<ConfigItem>> {
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
        let ctx = extract_context(request).await?;
        let config = find_previous_history(&mut descriptor, &id, &funs, &ctx).await?;
        Ok(Json(config))
    }
    #[oai(path = "/history/configs", method = "get")]
    async fn configs_by_namespace(
        &self,
        namespace_id: Query<Option<NamespaceId>>,
        tenant: Query<Option<NamespaceId>>,
        request: &Request,
    ) -> poem::Result<Json<Vec<ConfigItemDigest>>> {
        let mut namespace_id = namespace_id.0.or(tenant.0).unwrap_or("public".into());
        if namespace_id.is_empty() {
            namespace_id = "public".into();
        }
        let funs = crate::get_tardis_inst();
        let ctx = extract_context(request).await?;
        let config = get_configs_by_namespace(&namespace_id, &funs, &ctx).await?;
        Ok(Json(config))
    }
}
