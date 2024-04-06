use poem::http::StatusCode;
use tardis::basic::dto::TardisContext;
use tardis::web::poem;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::auth::BasicAuthorization;

use crate::dto::conf_auth_dto::NacosAuth;
use crate::serv::{auth, jwt_validate};

mod v1;
pub use self::v1::*;
mod v2;
pub use self::v2::*;
mod grpc;
pub use self::grpc::*;

pub type ConfNacosApi = (ConfNacosV1Api, ConfNacosV2Api);

pub async fn extract_context(request: &poem::Request) -> poem::Result<TardisContext> {
    #[derive(serde::Deserialize)]
    #[serde(untagged)]
    enum Auth {
        #[serde(rename_all = "camelCase")]
        AccessToken { access_token: String },
        #[serde(rename_all = "camelCase")]
        UsernamePassword { username: String, password: String },
    }
    let funs = crate::get_tardis_inst();
    if let Ok(basic_auth) = poem_openapi::auth::Basic::from_request(request) {
        auth(&basic_auth.username, &basic_auth.password, &funs).await.map_err(|e| poem::Error::from_string(e.message, StatusCode::FORBIDDEN))
    } else if let Ok(param_auth) = request.params::<Auth>() {
        match param_auth {
            Auth::AccessToken { access_token } => jwt_validate(&access_token, &funs).await,
            Auth::UsernamePassword { username, password } => auth(&username, &password, &funs).await.map_err(|e| poem::Error::from_string(e.message, StatusCode::FORBIDDEN)),
        }
    } else {
        // extract from from body:
        Err(poem::Error::from_status(StatusCode::NON_AUTHORITATIVE_INFORMATION))
    }
}

pub async fn extract_context_from_body<'a>(body_auth: impl Into<Option<NacosAuth<'a>>>) -> Option<poem::Result<TardisContext>> {
    if let Some(NacosAuth { username, password }) = body_auth.into() {
        let funs = crate::get_tardis_inst();
        Some(auth(username, password, &funs).await.map_err(|e| poem::Error::from_string(e.message, StatusCode::FORBIDDEN)))
    } else {
        None
    }
}
