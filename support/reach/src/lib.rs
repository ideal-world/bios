#![warn(clippy::dbg_macro)]
#![warn(clippy::unwrap_used)]

mod api;
mod domain;
pub mod dto;
pub mod reach_config;
pub mod reach_consts;
mod reach_init;
pub mod reach_send_channel;

#[cfg(feature = "simple-client")]
pub mod reach_invoke;
mod task;
pub use reach_init::init;
pub mod reach_initializer {
    pub use crate::init;
    pub use crate::reach_init::reach_init_trigger_scene;
}
mod serv;
