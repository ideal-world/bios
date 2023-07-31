use tardis::basic::dto::TardisContext;
use tardis::web::poem::Request;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::payload::Json;
use tardis::web::web_resp::{TardisApiResult, TardisResp};

use crate::dto::auth_crypto_dto::{AuthEncryptReq, AuthEncryptResp};
use crate::serv::auth_crypto_serv;
use crate::serv::clients::spi_log_client::{LogParamContent, SpiLogClient};

#[derive(Clone)]
pub struct CryptoApi;

/// Crypto API 密码API
#[poem_openapi::OpenApi(prefix_path = "/auth/crypto")]
impl CryptoApi {
    /// Fetch public key 获取公钥
    #[oai(path = "/key", method = "get")]
    async fn fetch_public_key(&self, req: &Request) -> TardisApiResult<String> {
        let result = auth_crypto_serv::fetch_public_key().await?;
        let _ = SpiLogClient::add_item(
            LogParamContent {
                op: "fetch_pubilc_key".to_string(),
                ext: None,
                addr: req.remote_addr().to_string(),
                ..Default::default()
            },
            None,
            None,
            &TardisContext::default(),
        )
        .await;
        TardisResp::ok(result)
    }

    /// Encrypt body 加密body
    #[oai(path = "/", method = "put")]
    async fn encrypt_body(&self, req: Json<AuthEncryptReq>, request: &Request) -> TardisApiResult<AuthEncryptResp> {
        let result = auth_crypto_serv::encrypt_body(&req.0).await?;
        let _ = SpiLogClient::add_item(
            LogParamContent {
                op: "encrypt_body".to_string(),
                ext: None,
                addr: request.remote_addr().to_string(),
                ..Default::default()
            },
            None,
            None,
            &TardisContext::default(),
        )
        .await;
        TardisResp::ok(result)
    }
}
