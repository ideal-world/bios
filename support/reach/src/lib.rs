#![warn(clippy::unwrap_used)]
#![warn(clippy::dbg_macro)]

pub mod api;
pub mod client;
pub mod config;
pub mod consts;
mod domain;
mod dto;
mod errors;
mod init;
#[cfg(feature = "simple-client")]
pub mod invoke;
mod task;
pub use init::init;
pub mod reach_initializer {
    pub use crate::init;
}
mod serv;
