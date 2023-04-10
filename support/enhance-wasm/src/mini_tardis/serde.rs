use serde::{de::DeserializeOwned, Serialize};
use wasm_bindgen::JsValue;

use super::{basic::TardisResult, error::TardisError};

pub fn obj_to_str<T: ?Sized + Serialize>(obj: &T) -> TardisResult<String> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        serde_json::to_string(obj).map_err(|e| TardisError::bad_request(&format!("[BIOS.GlobalApi] Serialize error:{e}"), ""))
    }
    #[cfg(target_arch = "wasm32")]
    {
        let result = obj_to_jsvalue(obj)?;
        jsvalue_to_str(&result)
    }
}

pub fn obj_to_jsvalue<T: ?Sized + Serialize>(obj: &T) -> TardisResult<JsValue> {
    obj.serialize(&serde_wasm_bindgen::Serializer::json_compatible()).map_err(|e| TardisError::bad_request(&format!("[BIOS.GlobalApi] Serialize error:{e}"), ""))
}

pub fn jsvalue_to_obj<T: DeserializeOwned>(obj: JsValue) -> TardisResult<T> {
    serde_wasm_bindgen::from_value::<T>(obj).map_err(|e| TardisError::bad_request(&format!("[BIOS.GlobalApi] Deserialize error:{e}"), ""))
}

pub fn jsvalue_to_str(obj: &JsValue) -> TardisResult<String> {
    let result = js_sys::JSON::stringify(&obj).map_err(|e| TardisError::bad_request(&format!("[BIOS.GlobalApi] Serialize error:{e:?}"), ""))?;
    let result = result.as_string().unwrap();
    Ok(result)
}
