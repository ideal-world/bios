use bios_basic::TardisFunInstExtractor;
use tardis::web::{
    context_extractor::TardisContextExtractor,
    poem::{self, web::Form, Request},
    poem_openapi::{self, payload::Json},
    reqwest::StatusCode,
};

use crate::serv::*;
use crate::{conf_config::ConfConfig, dto::conf_config_nacos_dto::*};

#[derive(Default, Clone, Debug)]
pub struct ConfNacosV1AuthApi;

/// Interface Console config server API
#[poem_openapi::OpenApi(prefix_path = "/nacos/v1/auth", tag = "bios_basic::ApiTag::Interface")]
impl ConfNacosV1AuthApi {
    #[oai(path = "/login", method = "post")]
    async fn login(&self, form: Form<LoginRequest>, ctx: TardisContextExtractor, request: &Request) -> poem::Result<Json<LoginResponse>> {
        let username = form.0.username;
        let password = form.0.password;
        let funs = request.tardis_fun_inst();
        let auth = auth(&username, &password, &funs);
        if !auth {
            return Err(poem::Error::from_string("incorrect username or password", StatusCode::FORBIDDEN));
        }
        let token = jwt_sign(&funs, &ctx.0).await?;
        let ttl = funs.conf::<ConfConfig>().token_ttl;
        Ok(Json(LoginResponse {
            access_token: token,
            token_ttl: ttl,
            global_admin: auth,
        }))
    }
}
