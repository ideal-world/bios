use crate::{constants::TOKEN_INFO, mini_tardis::basic::TardisResult};

pub fn set_token(token: &str) -> TardisResult<()> {
    let mut token_info = TOKEN_INFO.write().unwrap();
    *token_info = Some(token.to_string());
    Ok(())
}

pub fn remove_token() -> TardisResult<()> {
    let mut token_info = TOKEN_INFO.write().unwrap();
    *token_info = None;
    Ok(())
}
