use crate::{constants::SESSION_CONFIG, initializer, mini_tardis::basic::TardisResult};

pub fn set_token(token: &str) -> TardisResult<()> {
    let mut config_container = SESSION_CONFIG.write().unwrap();
    let session_config = config_container.as_mut().unwrap();
    session_config.token = Some(token.to_string());
    initializer::change_behavior(session_config, false)
}

pub fn remove_token() -> TardisResult<()> {
    let mut config_container = SESSION_CONFIG.write().unwrap();
    let session_config = config_container.as_mut().unwrap();
    session_config.token = None;
    initializer::change_behavior(session_config, false)
}

pub fn get_token() -> TardisResult<Option<String>> {
    let config = SESSION_CONFIG.read().unwrap();
    if let Some(token) = config.as_ref().unwrap().token.as_ref() {
        Ok(Some(token.to_string()))
    } else {
        Ok(None)
    }
}
