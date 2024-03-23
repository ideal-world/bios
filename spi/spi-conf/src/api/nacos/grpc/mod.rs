use std::{collections::HashMap, net::IpAddr};

use serde::{Deserialize, Serialize};
use tardis::{
    basic::{error::TardisError, result::TardisResult},
    futures_util::StreamExt,
    log, serde_json,
    web::poem,
};
#[allow(non_snake_case)]

mod proto;
pub use proto::{
    BiRequestStream as BiRequestStreamProto, BiRequestStreamServer as BiRequestStreamGrpcServer, Metadata, Payload, Request as RequestProto, RequestServer as RequestGrpcServer,
};
use tardis::web::poem_grpc::{self, Code, Request, Response, Status};

use crate::{
    dto::conf_config_dto::{ConfigDescriptor, ConfigItem},
    serv::placeholder::render_content_for_ip,
};

#[derive(Clone, Default)]
pub struct RequestProtoImpl;

#[poem::async_trait]
impl RequestProto for RequestProtoImpl {
    async fn request(&self, request: Request<Payload>) -> Result<Response<Payload>, Status> {
        let Some(metadata) = &request.metadata else {
            return Err(Status::new(Code::InvalidArgument));
        };
        log::trace!("metadata: {metadata:?}");
        let access_token = metadata.headers.get("accessToken").map(|x| x.as_str());
        let client_ip = metadata.client_ip.parse::<IpAddr>().ok();
        let Some(body) = &request.body else {
            return Err(Status::new(Code::InvalidArgument));
        };
        let body = String::from_utf8_lossy(&body.value);
        log::trace!("body: {}", body);
        let type_info = &metadata.r#type;
        dispatch_request(type_info, &body, access_token, client_ip).await.map(Response::new).map_err(|e| {
            log::error!("[spi-conf.nacos.grpc] dispatch_request error: {}", e);
            Status::new(Code::Internal)
        })
    }
}

#[derive(Clone, Default)]
pub struct BiRequestStreamProtoImpl;

#[poem::async_trait]
impl BiRequestStreamProto for BiRequestStreamProtoImpl {
    async fn request_bi_stream(&self, mut request_stream: Request<poem_grpc::Streaming<Payload>>) -> Result<Response<poem_grpc::Streaming<Payload>>, Status> {
        let (mut _tx, rx) = tardis::tokio::sync::mpsc::unbounded_channel::<Result<Payload, Status>>();
        tardis::tokio::spawn(async move {
            while let Some(maybe_pld) = request_stream.next().await {
                if let Ok(payload) = maybe_pld {
                    let Some(metadata) = &payload.metadata else {
                        return Err(Status::new(Code::InvalidArgument));
                    };
                    log::trace!("bistream: metadata: {metadata:?}");
                    let Some(body) = &payload.body else {
                        return Err(Status::new(Code::InvalidArgument));
                    };
                    let body = String::from_utf8_lossy(&body.value);
                    log::trace!("bistream: body: {}", body);
                }
            }
            Ok(())
        });
        let stream = tokio_stream::wrappers::UnboundedReceiverStream::new(rx);
        let _resp_stream = poem_grpc::Streaming::new(stream);
        // temporary return unimplemented
        Err(Status::new(Code::Unimplemented))
        // Ok(Response::new(resp_stream))
    }
}

pub trait AsPayload: Serialize {
    const TYPE_NAME: &'static str;
    fn as_payload(&self) -> Payload {
        Payload {
            metadata: Some(Metadata {
                r#type: Self::TYPE_NAME.into(),
                client_ip: "".into(),
                headers: HashMap::new(),
            }),
            body: Some(::prost_types::Any {
                type_url: Self::TYPE_NAME.into(),
                value: serde_json::to_vec(&self).expect("can't serialize"),
            }),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NaocsGrpcResponse {
    pub result_code: i32,
    pub error_code: Option<i32>,
    pub message: Option<String>,
    pub request_id: Option<String>,
}

impl NaocsGrpcResponse {
    pub const fn success() -> Self {
        Self {
            result_code: 200,
            error_code: None,
            message: None,
            request_id: None,
        }
    }
    pub const fn not_found() -> Self {
        Self {
            result_code: 500,
            error_code: Some(300),
            message: None,
            request_id: None,
        }
    }
    pub const fn unregister() -> Self {
        Self {
            result_code: 500,
            error_code: Some(301),
            message: None,
            request_id: None,
        }
    }
}

impl AsPayload for NaocsGrpcResponse {
    const TYPE_NAME: &'static str = "Response";
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerCheckResponse {
    #[serde(flatten)]
    pub response: NaocsGrpcResponse,
    pub connection_id: Option<String>,
}

impl ServerCheckResponse {
    pub fn success(connection_id: Option<String>) -> Self {
        Self {
            response: NaocsGrpcResponse::success(),
            connection_id,
        }
    }
}

impl AsPayload for ServerCheckResponse {
    const TYPE_NAME: &'static str = "ServerCheckResponse";
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthCheckResponse {
    #[serde(flatten)]
    pub response: NaocsGrpcResponse,
}

impl AsPayload for HealthCheckResponse {
    const TYPE_NAME: &'static str = "HealthCheckResponse";
}

impl HealthCheckResponse {
    pub fn success() -> Self {
        Self {
            response: NaocsGrpcResponse::success(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigQueryRequest {
    pub data_id: String,
    pub group: String,
    pub tenant: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigQueryResponse {
    content: String,
    encrypted_data_key: Option<String>,
    content_type: Option<String>,
    md5: String,
    last_modified: u64,
    is_beta: bool,
    tag: String,
    #[serde(flatten)]
    pub response: NaocsGrpcResponse,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConfigQueryResponseNotFound {
    #[serde(flatten)]
    pub response: NaocsGrpcResponse,
}

impl Default for ConfigQueryResponseNotFound {
    fn default() -> Self {
        ConfigQueryResponseNotFound {
            response: NaocsGrpcResponse::not_found(),
        }
    }
}

impl AsPayload for ConfigQueryResponseNotFound {
    const TYPE_NAME: &'static str = "ConfigQueryResponse";
}

impl From<ConfigItem> for ConfigQueryResponse {
    fn from(item: ConfigItem) -> Self {
        Self {
            content: item.content,
            content_type: None,
            encrypted_data_key: item.encrypted_data_key,
            md5: item.md5,
            last_modified: item.last_modified_time.timestamp_millis() as u64,
            is_beta: false,
            tag: item.config_tags.join(","),
            response: NaocsGrpcResponse::success(),
        }
    }
}

impl AsPayload for ConfigQueryResponse {
    const TYPE_NAME: &'static str = "ConfigQueryResponse";
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigContext {
    group: String,
    data_id: String,
    tenant: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigListenContext {
    group: String,
    data_id: String,
    tenant: String,
    md5: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigBatchListenRequest {
    pub listen: bool,
    pub config_listen_contexts: Vec<ConfigListenContext>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigChangeBatchListenResponse {
    pub changed_configs: Vec<ConfigContext>,
    #[serde(flatten)]
    pub response: NaocsGrpcResponse,
}

impl AsPayload for ConfigChangeBatchListenResponse {
    const TYPE_NAME: &'static str = "ConfigChangeBatchListenResponse";
}

pub async fn dispatch_request(type_info: &str, value: &str, access_token: Option<&str>, ip: Option<IpAddr>) -> TardisResult<Payload> {
    use crate::serv::*;
    let funs = crate::get_tardis_inst();
    let get_ctx = async {
        let Some(token) = access_token else {
            return Err(TardisError::unauthorized("missing access token", ""));
        };
        jwt_validate(token, &funs).await.map_err(|e| TardisError::unauthorized(&format!("invalid access token, error: {e}, token: {token}"), ""))
    };
    let response = match type_info {
        "ServerCheckRequest" => ServerCheckResponse::success(None).as_payload(),
        "HealthCheckRequest" => HealthCheckResponse::success().as_payload(),
        "ConfigQueryRequest" => {
            let Ok(ctx) = get_ctx.await else {
                return Ok(NaocsGrpcResponse::unregister().as_payload());
            };
            let ConfigQueryRequest { data_id, group, tenant } = serde_json::from_str(value).map_err(|_e| TardisError::bad_request("expect a ConfigQueryRequest", ""))?;
            let mut descriptor = ConfigDescriptor {
                namespace_id: tenant.unwrap_or("public".into()),
                data_id,
                group,
                ..Default::default()
            };
            match get_config_detail(&mut descriptor, &funs, &ctx).await {
                Ok(mut data) => {
                    if let Some(ip) = ip {
                        data.content = render_content_for_ip(data.content, ip, &funs, &ctx).await?;
                    }
                    ConfigQueryResponse::from(data).as_payload()
                }
                Err(_) => ConfigQueryResponseNotFound::default().as_payload(),
            }
        }
        "ConfigBatchListenRequest" => {
            let ctx = get_ctx.await?;
            let ConfigBatchListenRequest { listen, config_listen_contexts } =
                serde_json::from_str(value).map_err(|_e| TardisError::bad_request("expect a ConfigBatchListenRequest", ""))?;
            let mut changed_configs = Vec::with_capacity(config_listen_contexts.len());
            if listen {
                for config in config_listen_contexts {
                    let mut descriptor = ConfigDescriptor {
                        namespace_id: config.tenant,
                        group: config.group,
                        data_id: config.data_id,
                        ..Default::default()
                    };
                    let server_side_md5 = get_md5(&mut descriptor, &funs, &ctx).await?;
                    if server_side_md5 != config.md5 {
                        changed_configs.push(ConfigContext {
                            group: descriptor.group,
                            data_id: descriptor.data_id,
                            tenant: descriptor.namespace_id,
                        })
                    }
                }
            }
            ConfigChangeBatchListenResponse {
                changed_configs,
                response: NaocsGrpcResponse::success(),
            }
            .as_payload()
        }
        _ => {
            log::debug!("[Spi-Conf.Nacos.Grpc] unknown type_info: {}", type_info);
            Payload::default()
        }
    };
    Ok(response)
}
