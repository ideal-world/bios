use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::Event;

use crate::{
    constants::{STRICT_SECURITY_MODE, TARDIS_TOKEN, TOKEN_INFO},
    mini_tardis::{basic::TardisResult, error::TardisError},
};

pub fn init() -> TardisResult<()> {
    let channel = web_sys::BroadcastChannel::new(TARDIS_TOKEN).unwrap();
    let share_token_fn = Closure::<dyn Fn(Event)>::new(|e: Event| share_token_process(e.dyn_into::<web_sys::MessageEvent>().unwrap()));
    channel.add_event_listener_with_callback("message", share_token_fn.as_ref().unchecked_ref()).unwrap();
    share_token_fn.forget();
    Ok(())
}

fn share_token_process(e: web_sys::MessageEvent) {
    e.data().as_string().map(|token| {
        let mut token_info = TOKEN_INFO.write().unwrap();
        *token_info = Some(token);
    });
}

pub fn set_token(token: &str) -> TardisResult<()> {
    if *STRICT_SECURITY_MODE.read().unwrap() {
        let mut token_info = TOKEN_INFO.write().unwrap();
        *token_info = Some(token.to_string());
        web_sys::BroadcastChannel::new(TARDIS_TOKEN).unwrap().post_message(&token.to_string().into()).unwrap();
    } else {
        if let Ok(Some(storage)) = web_sys::window().unwrap().local_storage() {
            storage.set_item(TARDIS_TOKEN, token).unwrap()
        } else {
            return Err(TardisError::io_error("[BIOS.Token] Can't get local storage", ""));
        }
    }
    Ok(())
}

pub fn remove_token() -> TardisResult<()> {
    if *STRICT_SECURITY_MODE.read().unwrap() {
        let mut token_info = TOKEN_INFO.write().unwrap();
        *token_info = None;
        web_sys::BroadcastChannel::new(TARDIS_TOKEN).unwrap().post_message(&"".to_string().into()).unwrap();
    } else {
        if let Ok(Some(storage)) = web_sys::window().unwrap().local_storage() {
            storage.remove_item(TARDIS_TOKEN).unwrap()
        } else {
            return Err(TardisError::io_error("[BIOS.Token] Can't get local storage", ""));
        }
    }

    Ok(())
}

pub fn get_token() -> TardisResult<String> {
    if *STRICT_SECURITY_MODE.read().unwrap() {
        let token_info = TOKEN_INFO.read().unwrap();
        if let Some(token) = token_info.as_ref() {
            Ok(token.to_string())
        } else {
            Ok("".to_string())
        }
    } else {
        if let Ok(Some(storage)) = web_sys::window().unwrap().local_storage() {
            Ok(storage.get_item(TARDIS_TOKEN).unwrap().unwrap_or("".to_string()))
        } else {
            return Err(TardisError::io_error("[BIOS.Token] Can't get local storage", ""));
        }
    }
}
