use serde::{de::DeserializeOwned, Serialize};

use super::{basic::TardisResult, error::TardisError};

pub fn obj_to_str<T: ?Sized + Serialize>(obj: &T) -> TardisResult<String> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        serde_json::to_string(obj).map_err(|e| TardisError::bad_request(&format!("[Tardis.Serde] Serialize error:{e}"), ""))
    }
    #[cfg(target_arch = "wasm32")]
    {
        let result = obj_to_jsvalue(obj)?;
        jsvalue_to_str(&result)
    }
}

#[allow(dead_code)]
pub fn str_to_obj<T: DeserializeOwned>(obj: &str) -> TardisResult<T> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        serde_json::from_str(obj).map_err(|e| TardisError::bad_request(&format!("[Tardis.Serde] Deserialize error:{e}"), ""))
    }
    #[cfg(target_arch = "wasm32")]
    {
        let result = str_to_jsvalue(obj)?;
        jsvalue_to_obj(result)
    }
}

#[allow(dead_code)]

pub fn copy<T: ?Sized + Serialize + DeserializeOwned>(obj: &T) -> TardisResult<T> {
    str_to_obj(&obj_to_str(obj)?)
}

// #[cfg(target_arch = "wasm32")]
pub fn obj_to_jsvalue<T: ?Sized + Serialize>(obj: &T) -> TardisResult<wasm_bindgen::JsValue> {
    obj.serialize(&serde_wasm_bindgen::Serializer::json_compatible()).map_err(|e| TardisError::bad_request(&format!("[Tardis.Serde] Serialize error:{e}"), ""))
}

// #[cfg(target_arch = "wasm32")]
pub fn jsvalue_to_obj<T: DeserializeOwned>(obj: wasm_bindgen::JsValue) -> TardisResult<T> {
    serde_wasm_bindgen::from_value::<T>(obj).map_err(|e| TardisError::bad_request(&format!("[Tardis.Serde] Deserialize error:{e}"), ""))
}

// #[cfg(target_arch = "wasm32")]
pub fn jsvalue_to_str(obj: &wasm_bindgen::JsValue) -> TardisResult<String> {
    let result = js_sys::JSON::stringify(obj).map_err(|e| TardisError::bad_request(&format!("[Tardis.Serde] Serialize error:{e:?}"), ""))?;
    let result = result.as_string().unwrap();
    Ok(result)
}

#[allow(dead_code)]
// #[cfg(target_arch = "wasm32")]
pub fn str_to_jsvalue(obj: &str) -> TardisResult<wasm_bindgen::JsValue> {
    let result = js_sys::JSON::parse(obj).map_err(|e| TardisError::bad_request(&format!("[Tardis.Serde] Deserialize error:{e:?}"), ""))?;
    Ok(result)
}
