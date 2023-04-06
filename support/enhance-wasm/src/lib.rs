use wasm_bindgen::prelude::*;
mod constants;
mod initializer;
mod mini_tardis;
mod modules;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub async fn init_by_url(service_url: &str) -> Result<(), JsValue> {
    initializer::init_by_url(service_url).await
}

#[wasm_bindgen]
pub fn init_by_conf(config: JsValue) -> Result<(), JsValue> {
    initializer::init_by_conf(config)
}

#[wasm_bindgen]
pub fn crypto_encrypt(body: &str, method: &str, uri: &str) -> Result<JsValue, JsValue> {
    modules::crypto_process::encrypt(body, method, uri)
}

#[wasm_bindgen]
pub fn crypto_decrypt(encrypt_body: &str, encrypt_key: &str) -> Result<String, JsValue> {
    modules::crypto_process::decrypt(encrypt_body, encrypt_key)
}
