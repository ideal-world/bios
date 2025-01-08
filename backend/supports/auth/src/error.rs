use tardis::basic::error::TardisError;

pub trait AuthError {
    // 410: The request signature is incorrect.
    fn signature_error(msg: &str, locale_code: &str) -> TardisError {
        TardisError::custom("410", msg, locale_code)
    }
}

impl AuthError for TardisError {}

