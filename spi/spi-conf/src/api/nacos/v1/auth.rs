use bios_basic::TardisFunInstExtractor;
use tardis::{
    basic::error::TardisError,
    db::sea_orm::prelude::Uuid,
    serde_json::{self, Value},
    web::{
        context_extractor::TardisContextExtractor,
        poem::{self, web::Form, Request},
        poem_openapi::{self, param::Query, payload::Json},
        reqwest::StatusCode,
        web_resp::{TardisApiResult, TardisResp, Void},
    },
};

use crate::{
    conf_config::ConfConfig,
    dto::{conf_config_dto::*, conf_config_nacos_dto::*, conf_namespace_dto::*},
};
use crate::{conf_constants::error, serv::*};

#[derive(Default)]
pub struct ConfNacosV1AuthApi;

/// Interface Console config server API
#[poem_openapi::OpenApi(prefix_path = "/nacos/v1/auth", tag = "bios_basic::ApiTag::Interface")]
impl ConfNacosV1AuthApi {
    #[oai(path = "/login", method = "post")]
    async fn login(
        &self,
        // username: Query<String>,
        // password: Query<String>,
        form: Form<LoginRequest>,
        request: &Request,
    ) -> poem::Result<Json<LoginResponse>> {
        let username = form.0.username;
        let password = form.0.password;
        let funs = request.tardis_fun_inst();
        let auth = auth(&username, &password, &funs);
        if !auth {
            return Err(poem::Error::from_string("incorrect username or password", StatusCode::FORBIDDEN));
        }
        let token = jwt_sign(&funs)?;
        let ttl = funs.conf::<ConfConfig>().token_ttl;
        Ok(Json(LoginResponse {
            access_token: token,
            token_ttl: ttl,
            global_admin: auth,
        }))
    }
}
