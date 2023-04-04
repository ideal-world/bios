use wasm_bindgen::prelude::*;
mod basic;
mod helper;
mod modules;

#[wasm_bindgen]
pub async fn init(serv_url: String) -> Result<(), JsValue> {
    modules::crypto::crypto::init(&serv_url).await
}

#[wasm_bindgen]
pub fn encrypt(body: &str, need_crypto_req: bool, need_crypto_resp: bool) -> Result<JsValue, JsValue> {
    modules::crypto::crypto::encrypt(body, need_crypto_req, need_crypto_resp)
}

#[wasm_bindgen]
pub fn decrypt(encrypt_body: &str, encrypt_key: &str) -> Result<String, JsValue> {
    modules::crypto::crypto::decrypt(encrypt_body, encrypt_key)
}

#[wasm_bindgen]
pub fn test(text: &str) -> Result<String, JsValue> {
    modules::crypto::crypto::test(text)
}

// TODO double auth

// TODO APIs

// TODO WS TOKEN
