#[derive(Clone, Default)]
pub struct RequestCryptoStatus {
    pub is_mix: bool,
    pub have_head_crypto_key: bool,
    pub is_skip_crypto: bool,
}
