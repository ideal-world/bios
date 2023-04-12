use wasm_bindgen::{JsCast, JsValue};

use serde::Serialize;

use crate::constants::INST;

use super::basic::TardisResult;

pub fn send<T: Serialize>(key: &str, obj: &T) -> TardisResult<()> {
    let data = crate::mini_tardis::serde::obj_to_jsvalue(obj)?;
    let send_data = crate::mini_tardis::serde::str_to_jsvalue("{}").unwrap();
    let inst = INST.read().unwrap().to_string();
    js_sys::Reflect::set(&send_data, &"inst".into(), &inst.into()).unwrap();
    js_sys::Reflect::set(&send_data, &"data".into(), &data).unwrap();
    web_sys::BroadcastChannel::new(key)?.post_message(&send_data)?;
    Ok(())
}

pub fn init<FF, RF>(key: &str, fetch_data_fun: FF, receive_data_fun: RF) -> TardisResult<()>
where
    FF: Fn(String) + 'static,
    RF: Fn(JsValue) + 'static,
{
    let inst = INST.read().unwrap().to_string();
    let inst_clone = inst.clone();
    let receive_data_process_fn = wasm_bindgen::prelude::Closure::<dyn Fn(web_sys::Event)>::new(move |e: web_sys::Event| {
        let e = wasm_bindgen::JsCast::dyn_into::<web_sys::MessageEvent>(e).unwrap();
        let receive_data = e.data();
        let send_inst = js_sys::Reflect::get(&receive_data, &"inst".into()).unwrap().as_string().unwrap();
        if send_inst != inst {
            receive_data_fun(js_sys::Reflect::get(&receive_data, &"data".into()).unwrap());
        }
    });
    let fetch_data_process_fn = wasm_bindgen::prelude::Closure::<dyn Fn(web_sys::Event)>::new(move |e: web_sys::Event| {
        let e = wasm_bindgen::JsCast::dyn_into::<web_sys::MessageEvent>(e).unwrap();
        let receive_data = e.data();
        let send_inst = js_sys::Reflect::get(&receive_data, &"inst".into()).unwrap().as_string().unwrap();
        if send_inst != inst_clone {
            fetch_data_fun(js_sys::Reflect::get(&receive_data, &"data".into()).unwrap().as_string().unwrap());
        }
    });

    web_sys::BroadcastChannel::new(key)?.add_event_listener_with_callback("message", receive_data_process_fn.as_ref().unchecked_ref())?;
    web_sys::BroadcastChannel::new("__fetch__")?.add_event_listener_with_callback("message", fetch_data_process_fn.as_ref().unchecked_ref())?;

    receive_data_process_fn.forget();
    fetch_data_process_fn.forget();

    send("__fetch__", &key)?;
    Ok(())
}
