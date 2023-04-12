use crate::{
    constants::{self, CONFIG},
    mini_tardis::basic::TardisResult,
};

pub fn set_token(token: &str) -> TardisResult<()> {
    constants::conf_by_token(Some(token.to_string()))
}

pub fn remove_token() -> TardisResult<()> {
    constants::conf_by_token(None)
}

pub fn get_token() -> TardisResult<Option<String>> {
    let config = CONFIG.read().unwrap();
    if let Some(token) = config.as_ref().unwrap().token.as_ref() {
        Ok(Some(token.to_string()))
    } else {
        Ok(None)
    }
}
