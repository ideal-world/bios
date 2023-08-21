use tardis::{web::poem, log, basic::result::TardisResult};
#[allow(non_snake_case)]
mod proto;
pub use proto:: {
    Request as RequestProto,
    RequestServer as RequestGrpcServer,
    Payload,
    Metadata,
};
use poem_grpc::{Request, Response, Status, Streaming, Code};

#[derive(Clone, Default)]
pub struct RequestProtoImpl;

#[poem::async_trait]
impl RequestProto for RequestProtoImpl {
    async fn request(
        &self,
        request: Request<Payload>,
    ) -> Result<Response<Payload>, Status> {
        let Some(metadata) = &request.metadata else {
            return Err(Status::new(Code::InvalidArgument));
        };
        let Some(body) = &request.body else {
            return Err(Status::new(Code::InvalidArgument));
        };
        let body = String::from_utf8_lossy(&body.value);
        log::debug!("body: {}", body);
        let type_info = &metadata.r#type;
        
        // reflect type_info to get the type of the request
        log::debug!("type_info: {}", type_info);
        return Ok(Response::new(Payload::default()));
    }
}


pub async fn dispatch_request(
    type_info: &str, value: &str,
) -> TardisResult<Payload> {
    match request.type_info {
        "ServerCheckRequest" => {
            
        }
    }
}