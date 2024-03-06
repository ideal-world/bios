use spacegate_shell::hyper::header::HeaderValue;

#[derive(Clone, Default)]
pub struct RequestCryptoParam {
    pub is_mix: bool,
    pub head_crypto_key: HeadCryptoKey,
    pub is_skip_crypto: bool,
}

#[derive(Clone, Default)]
pub enum HeadCryptoKey {
    #[default]
    None,
    Some(HeaderValue),
}

impl HeadCryptoKey {
    pub fn is_some(&self) -> bool {
        matches!(self, HeadCryptoKey::Some(_))
    }
    pub fn is_none(&self)->bool{
        !self.is_some()
    }
}
