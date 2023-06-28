use bios_basic::TardisFunInstExtractor;
use tardis::web::{
    poem::{self, web::Form, Request},
    poem_openapi::{self, payload::Json},
};

use crate::serv::*;
use crate::{conf_config::ConfConfig, dto::conf_config_nacos_dto::*};

#[derive(Default, Clone, Debug)]
pub struct ConfNacosV1AuthApi;

/// Interface Console config server API
#[poem_openapi::OpenApi(prefix_path = "/nacos/v1/auth", tag = "bios_basic::ApiTag::Interface")]
impl ConfNacosV1AuthApi {
    #[oai(path = "/login", method = "post")]
    async fn login(&self, form: Form<LoginRequest>, request: &Request) -> poem::Result<Json<LoginResponse>> {
        let username = form.0.username;
        let password = form.0.password;
        let funs = request.tardis_fun_inst();
        let ctx = auth(&username, &password, &funs).await?;
        let token = jwt_sign(&funs, &ctx).await?;
        let cfg = funs.conf::<ConfConfig>();
        Ok(Json(LoginResponse {
            access_token: token,
            token_ttl: cfg.token_ttl,
            global_admin: true,
        }))
    }
}
