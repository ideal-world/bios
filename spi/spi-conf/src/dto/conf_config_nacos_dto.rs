use std::time::*;

use serde::{Deserialize, Serialize};

use tardis::web::poem_openapi;
use tardis::web::poem_openapi::types::*;

use crate::utils::parse_tags;

use super::conf_config_dto::{ConfigDescriptor, ConfigPublishRequest};
use super::conf_namespace_dto::{NamespaceAttribute, NamespaceId, NamespaceItem};
#[derive(Debug, Serialize, Deserialize, poem_openapi::Object)]
pub struct NacosResponse<T: Type + ParseFromJSON + ToJSON> {
    code: u16,
    message: String,
    data: T,
}

macro_rules! define_api_code {
    (
        $(
            $(#[$attr:meta])*
            $name:ident: $code:literal, $message:literal;
        )*

    ) => {
        $(
            $(#[$attr])*
            pub fn $name(data: T) -> Self {
                Self { code: $code, message: $message.to_string(), data }
            }
        )*
    };
}

impl<T: Type + ParseFromJSON + ToJSON> NacosResponse<T> {
    define_api_code! {
        /// 成功
        ok: 0, "success";
        /// 参数缺失
        parameter_missing: 10000, "parameter missing";
        /// 访问拒绝
        access_denied: 10001, "access denied";
        /// 数据访问错误
        data_access_error: 10002, "data access error";
        /// tenant参数错误
        tenant_parameter_error: 20001, "'tenant' parameter error";
        /// 参数验证错误
        parameter_validate_error: 20002, "parameter validate error";
        /// 请求的MediaType错误
        media_type_error: 20003, "MediaType Error";
        /// 资源未找到
        resource_not_found: 20004, "resource not found";
        /// 资源访问冲突
        resource_conflict: 20005, "resource conflict";
        /// 监听配置为空
        config_listener_is_null: 20006, "config listener is null";
        /// 监听配置错误
        config_listener_error: 20007, "config listener error";
        /// 无效的dataId（鉴权失败）
        invalid_dataid: 20008, "invalid dataId";
        /// 请求参数不匹配
        parameter_mismatch: 20009, "parameter mismatch";
        /// serviceName服务名错误
        service_name_error: 21000, "service name error";
        /// weight权重参数错误
        weight_error: 21001, "weight error";
        /// 实例metadata元数据错误
        instance_metadata_error: 21002, "instance metadata error";
        /// instance实例不存在
        instance_not_found: 21003, "instance not found";
        /// instance实例信息错误
        instance_error: 21004, "instance error";
        /// 服务metadata元数据错误
        service_metadata_error: 21005, "service metadata error";
        /// 访问策略selector错误
        selector_error: 21006, "selector error";
        /// 服务已存在
        service_already_exist: 21007, "service already exist";
        /// 服务不存在
        service_not_exist: 21008, "service not exist";
        /// 存在服务实例，服务删除失败
        service_delete_failure: 21009, "service delete failure";
        /// healthy参数缺失
        healthy_param_miss: 21010, "healthy param miss";
        /// 健康检查仍在运行
        health_check_still_running: 21011, "health check still running";
        /// 命名空间namespace不合法
        illegal_namespace: 22000, "illegal namespace";
        /// 命名空间不存在
        namespace_not_exist: 22001, "namespace not exist";
        /// 命名空间已存在
        namespace_already_exist: 22002, "namespace already exist";
        /// 状态state不合法
        illegal_state: 23000, "illegal state";
        /// 节点信息错误
        node_info_error: 23001, "node info error";
        /// 节点离线操作出错
        node_down_failure: 23002, "node down failure";
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NacosJwtClaim {
    pub exp: u64,
    pub sub: String,
}

impl NacosJwtClaim {
    pub fn gen(ttl: u64, user: &str) -> Self {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).expect("invalid system time cause by time travel").as_secs();
        Self {
            exp: now + ttl,
            sub: String::from(user),
        }
    }
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Default)]
#[serde(rename = "camelCase")]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Default)]
#[serde(rename = "camelCase")]
#[oai(rename_all = "camelCase")]
pub struct LoginResponse {
    #[oai(rename = "accessToken")]
    pub access_token: String,
    #[oai(rename = "tokenTtl")]
    pub token_ttl: u32,
    #[oai(rename = "globalAdmin")]
    pub global_admin: bool,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Default)]
pub struct PublishConfigForm {
    pub(crate) username: Option<String>,
    pub(crate) password: Option<String>,
    pub content: Option<String>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Default)]
#[allow(non_snake_case)]
pub struct PublishConfigFormV2 {
    //否 命名空间，默认为public与 ''相同
    pub namespaceId: Option<String>,
    #[oai(validator(min_length = 1, max_length = 256))]
    //是 配置组名
    pub group: String,
    #[oai(validator(min_length = 1, max_length = 256))]
    //是 配置名
    pub dataId: String,
    //是 配置内容
    pub content: String,
    //否 标签
    pub tag: Option<String>,
    //否 应用名
    pub appName: Option<String>,
    //否 源用户
    pub srcUser: Option<String>,
    //否 配置标签列表，可多个，逗号分隔
    pub configTags: Option<String>,
    //否 配置描述
    pub desc: Option<String>,
    //否 -
    pub r#use: Option<String>,
    //否 -
    pub effect: Option<String>,
    //否 配置类型
    pub r#type: Option<String>,
    //否 -
    pub schema: Option<String>,
}

impl From<PublishConfigFormV2> for ConfigPublishRequest {
    fn from(val: PublishConfigFormV2) -> Self {
        let config_tags = val.configTags.as_deref().map(parse_tags).unwrap_or_default();
        ConfigPublishRequest {
            content: val.content,
            descriptor: ConfigDescriptor {
                namespace_id: val.namespaceId.unwrap_or("public".into()),
                group: val.group,
                data_id: val.dataId,
                tags: val.tag.into_iter().collect(),
                tp: val.r#type,
            },
            app_name: val.appName,
            src_user: val.srcUser,
            config_tags,
            desc: val.desc,
            r#use: val.r#use,
            effect: val.effect,
            schema: val.schema,
        }
    }
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Default)]
#[allow(non_snake_case)]
#[oai(rename_all = "camelCase")]
pub struct NacosCreateNamespaceRequest {
    pub(crate) username: Option<String>,
    pub(crate) password: Option<String>,
    customNamespaceId: String,
    namespaceName: String,
    namespaceDesc: Option<String>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Default)]
#[allow(non_snake_case)]
#[oai(rename_all = "camelCase")]
pub struct NacosEditNamespaceRequest {
    pub(crate) username: Option<String>,
    pub(crate) password: Option<String>,
    namespace: String,
    namespaceShowName: String,
    namespaceDesc: Option<String>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Default)]
#[allow(non_snake_case)]
pub struct NacosDeleteNamespaceRequest {
    pub(crate) username: Option<String>,
    pub(crate) password: Option<String>,
    pub(crate) namespaceId: String,
}

impl From<NacosCreateNamespaceRequest> for NamespaceAttribute {
    fn from(value: NacosCreateNamespaceRequest) -> Self {
        Self {
            namespace: value.customNamespaceId,
            namespace_show_name: value.namespaceName,
            namespace_desc: value.namespaceDesc,
        }
    }
}

impl From<NacosEditNamespaceRequest> for NamespaceAttribute {
    fn from(value: NacosEditNamespaceRequest) -> Self {
        Self {
            namespace: value.namespace,
            namespace_show_name: value.namespaceShowName,
            namespace_desc: value.namespaceDesc,
        }
    }
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Default)]
#[allow(non_snake_case)]
pub struct NamespaceItemNacos {
    pub namespace: NamespaceId,
    pub namespaceShowName: String,
    pub namespaceDesc: Option<String>,
    pub r#type: u32,
    /// quota / 容量,
    /// refer to design of nacos,
    /// see: https://github.com/alibaba/nacos/issues/4558
    pub quota: u32,
    pub configCount: u32,
}

impl From<NamespaceItem> for NamespaceItemNacos {
    fn from(value: NamespaceItem) -> Self {
        Self {
            namespace: value.namespace,
            namespaceShowName: value.namespace_show_name,
            namespaceDesc: value.namespace_desc,
            r#type: value.tp,
            quota: value.quota,
            configCount: value.config_count,
        }
    }
}
