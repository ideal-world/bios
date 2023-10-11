#![warn(clippy::dbg_macro)]
#![warn(clippy::unwrap_used)]

pub mod api;
pub mod client;
pub mod config;
pub mod consts;
mod domain;
pub mod dto;
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
