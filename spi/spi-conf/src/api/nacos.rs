mod v1;

use tardis::basic::dto::TardisContext;
use tardis::web::poem;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::auth::BasicAuthorization;

use crate::serv::{auth, jwt_validate};

pub use self::v1::*;
mod v2;
pub use self::v2::*;

pub type ConfNacosApi = (ConfNacosV1Api, ConfNacosV2Api);

pub async fn extract_context(request: &poem::Request) -> poem::Result<TardisContext> {
    #[derive(serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct AccessToken {
        access_token: String,
    }
    let funs = crate::get_tardis_inst();
    if let Ok(basic_auth) = poem_openapi::auth::Basic::from_request(request) {
        auth(&basic_auth.username, &basic_auth.password, &funs).await.map_err(|e| poem::Error::from_string(e.message, StatusCode::FORBIDDEN))
    } else if let Ok(AccessToken { access_token }) = request.params::<AccessToken>() {
        jwt_validate(&access_token, &funs).await
    } else {
        Err(poem::Error::from_status(StatusCode::NON_AUTHORITATIVE_INFORMATION))
    }
}
