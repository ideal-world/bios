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
        #[oai(name = "accessToken")] access_token: Query<String>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> poem::Result<PlainText<String>> {
        let namespace_id = namespace_id.0.or(tenant.0).unwrap_or("public".into());
        let mut descriptor = ConfigDescriptor {
            namespace_id,
            group: group.0,
            data_id: data_id.0,
            ..Default::default()
        };
        let funs = request.tardis_fun_inst();
        jwt_validate(&access_token.0, &funs)?;
        let content = get_config(&mut descriptor, &funs, &ctx.0).await.map_err(tardis_err_to_poem_err)?;
        Ok(PlainText(content))
    }
    #[oai(path = "/configs", method = "post")]
    async fn publish_config(
        &self,
        tenant: Query<Option<NamespaceId>>,
        #[oai(name = "namespaceId")] namespace_id: Query<Option<NamespaceId>>,
        /// 配置分组名
        group: Query<String>,
        /// 配置名
        #[oai(name = "dataId")]
        data_id: Query<String>,
        #[oai(name = "accessToken")] access_token: Query<String>,
        form: Form<PublishConfigForm>,
        r#type: Query<Option<String>>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> poem::Result<Json<bool>> {
        let funs = request.tardis_fun_inst();
        let namespace_id = namespace_id.0.or(tenant.0).unwrap_or("public".into());
        let username = jwt_validate(&access_token.0, &funs)?.sub;
        let descriptor = ConfigDescriptor {
            namespace_id,
            group: group.0,
            data_id: data_id.0,
            ..Default::default()
        };
        let mut publish_request = ConfigPublishRequest {
            descriptor,
            schema: r#type.0,
            content: form.0.content,
            src_user: Some(username),
            ..Default::default()
        };
        let success = publish_config(&mut publish_request, &funs, &ctx.0).await?;
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
        #[oai(name = "accessToken")] access_token: Query<String>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> poem::Result<Json<bool>> {
        let namespace_id = namespace_id.0.or(tenant.0).unwrap_or("public".into());
        let mut descriptor = ConfigDescriptor {
            namespace_id,
            group: group.0,
            data_id: data_id.0,
            ..Default::default()
        };
        let funs = request.tardis_fun_inst();
        jwt_validate(&access_token.0, &funs)?;
        let success = delete_config(&mut descriptor, &funs, &ctx.0).await?;
        Ok(Json(success))
    }
    #[oai(path = "/configs/listener", method = "post")]
    async fn listener(
        &self,
        // mut descriptor: Query<ConfigDescriptor>,
        // Listening-Configs
        #[oai(name = "Listening-Configs")] listening_configs: Query<String>,
        #[oai(name = "accessToken")] access_token: Query<String>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> poem::Result<PlainText<String>> {
        let listening_configs = listening_configs.0;
        let funs = request.tardis_fun_inst();
        jwt_validate(&access_token.0, &funs)?;
        let err_missing = |msg: &str| poem::Error::from_string(format!("missing field {msg}"), poem::http::StatusCode::BAD_REQUEST);
        let mut config_fields = listening_configs.trim_end_matches(1 as char).split(2 as char);
        let data_id = config_fields.next().ok_or(err_missing("data_id"))?;
        let group = config_fields.next().ok_or(err_missing("group"))?;
        let md5 = config_fields.next().ok_or(err_missing("contentMD5"))?;
        let tenant = config_fields.next().unwrap_or("public");
        let mut descriptor = ConfigDescriptor {
            namespace_id: tenant.to_owned(),
            group: group.to_owned(),
            data_id: data_id.to_owned(),
            ..Default::default()
        };
        let config = if md5.is_empty() || md5 != get_md5(&mut descriptor, &funs, &ctx.0).await? {
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
        #[oai(name = "accessToken")] access_token: Query<String>,
        ctx: TardisContextExtractor,
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
        let funs = request.tardis_fun_inst();
        jwt_validate(&access_token.0, &funs)?;
        if let Some("accurate") = search.0.as_deref() {
            let page_size = page_size.0.unwrap_or(100).min(500).max(1);
            let page_no = page_no.0.unwrap_or(1).max(0);
            let mut request = ConfigHistoryListRequest { descriptor, page_no, page_size };
            let response = get_history_list_by_namespace(&mut request, &funs, &ctx.0).await?;
            Ok(Json(serde_json::to_value(response).expect("fail to convert ConfigListResponse to json value")))
        } else {
            let id = Uuid::parse_str(&nid.0).map_err(|e| TardisError::bad_request(&e.to_string(), error::INVALID_UUID))?;
            let config = find_history(&mut descriptor, &id, &funs, &ctx.0).await?;
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
        #[oai(name = "accessToken")] access_token: Query<String>,
        ctx: TardisContextExtractor,
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
        let funs = request.tardis_fun_inst();
        jwt_validate(&access_token.0, &funs)?;
        let config = find_previous_history(&mut descriptor, &id, &funs, &ctx.0).await?;
        Ok(Json(config))
    }
    #[oai(path = "/history/configs", method = "get")]
    async fn configs_by_namespace(
        &self,
        namespace_id: Query<Option<NamespaceId>>,
        tenant: Query<Option<NamespaceId>>,
        #[oai(name = "accessToken")] access_token: Query<String>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> poem::Result<Json<Vec<ConfigItemDigest>>> {
        let mut namespace_id = namespace_id.0.or(tenant.0).unwrap_or("public".into());
        if namespace_id.is_empty() {
            namespace_id = "public".into();
        }
        let funs = request.tardis_fun_inst();
        jwt_validate(&access_token.0, &funs)?;
        let config = get_configs_by_namespace(&namespace_id, &funs, &ctx.0).await?;
        Ok(Json(config))
    }
}
