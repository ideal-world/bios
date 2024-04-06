pub fn now() -> f64 {
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as f64
    }
    #[cfg(target_arch = "wasm32")]
    {
        js_sys::Date::now()
    }
}
