use tardis::web::poem_openapi;
use tardis::web::poem_openapi::payload::Json;
use tardis::web::web_resp::{TardisApiResult, TardisResp};

use crate::dto::auth_crypto_dto::{AuthEncryptReq, AuthEncryptResp};
use crate::serv::auth_crypto_serv;

pub struct CryptoApi;

/// Crypto API
#[poem_openapi::OpenApi(prefix_path = "/auth/crypto")]
impl CryptoApi {
    /// Fetch public key
    #[oai(path = "/key", method = "get")]
    async fn fetch_public_key(&self) -> TardisApiResult<String> {
        let result = auth_crypto_serv::fetch_public_key().await?;
        TardisResp::ok(result)
    }

    /// Encrypt body
    #[oai(path = "/", method = "put")]
    async fn encrypt_body(&self, req: Json<AuthEncryptReq>) -> TardisApiResult<AuthEncryptResp> {
        let result = auth_crypto_serv::encrypt_body(&req.0).await?;
        TardisResp::ok(result)
    }
}
