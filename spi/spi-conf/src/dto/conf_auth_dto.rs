use serde::{Deserialize, Serialize};
use tardis::{basic::field::TrimString, web::poem_openapi};

#[derive(Debug, Serialize, Deserialize, poem_openapi::Object)]
pub struct RegisterResponse {
    pub username: String,
    pub password: String,
}

impl RegisterResponse {
    pub fn new(username: impl Into<String>, password: impl Into<String>) -> Self {
        Self { username: username.into(), password: password.into() }
    }
}

#[derive(Debug, Serialize, Deserialize, poem_openapi::Object, Default)]
pub struct RegisterRequest {
    #[oai(validator(pattern = r"^[a-zA-Z\d_]{6,16}$"))]
    pub username: Option<TrimString>,
    #[oai(validator(pattern = r"^[a-zA-Z\d~!@#$%^&*\(\)_+]{8,16}$"))]
    pub password: Option<TrimString>,
}

impl RegisterRequest {
    #[inline]
    pub fn ak(&self) -> Option<&str> {
        self.username.as_deref()
    }
    #[inline]
    pub fn sk(&self) -> Option<&str> {
        self.password.as_deref()
    }
}
