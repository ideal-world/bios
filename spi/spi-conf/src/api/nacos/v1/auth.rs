use tardis::web::{
    poem::{self, web::Form},
    poem_openapi::{self, param::Query, payload::Json},
};

use crate::serv::*;
use crate::{conf_config::ConfConfig, dto::conf_config_nacos_dto::*};

#[derive(Default, Clone, Copy, Debug)]
pub struct ConfNacosV1AuthApi;

/// Interface Console config server API
#[poem_openapi::OpenApi(prefix_path = "/nacos/v1/auth", tag = "bios_basic::ApiTag::Interface")]
impl ConfNacosV1AuthApi {
    #[oai(path = "/login", method = "post")]
    async fn login(&self, username: Query<Option<String>>, password: Query<Option<String>>, form: Form<LoginRequest>) -> poem::Result<Json<LoginResponse>> {
        let username = username.0.or(form.0.username).unwrap_or_default();
        let password = password.0.or(form.0.password).unwrap_or_default();
        let funs = crate::get_tardis_inst();
        let ctx = auth(&username, &password, &funs).await?;
        let token = jwt_sign(&funs, &ctx).await?;
        let cfg = funs.conf::<ConfConfig>();
        Ok(Json(LoginResponse {
            access_token: token,
            token_ttl: cfg.token_ttl,
            global_admin: true,
        }))
    }

    /// this is for java client
    #[oai(path = "/users/login", method = "post")]
    async fn users_login(&self, username: Query<Option<String>>, password: Query<Option<String>>, form: Form<LoginRequest>) -> poem::Result<Json<LoginResponse>> {
        self.login(username, password, form).await
    }

    /// this is for java client
    #[oai(path = "/user/login", method = "post")]
    async fn user_login(&self, username: Query<Option<String>>, password: Query<Option<String>>, form: Form<LoginRequest>) -> poem::Result<Json<LoginResponse>> {
        self.login(username, password, form).await
    }
}
