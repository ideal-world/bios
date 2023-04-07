pub fn log(message: &str) {
    #[cfg(not(target_arch = "wasm32"))]
    {
        println!("{}", message);
    }
    #[cfg(target_arch = "wasm32")]
    {
        web_sys::console::log_1(&message.to_string().into());
    }
}
