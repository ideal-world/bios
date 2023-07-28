#![warn(clippy::unwrap_used)]
#![warn(clippy::dbg_macro)]

pub mod api;
pub mod client;
pub mod config;
pub mod consts;
pub mod invoke;
mod task;
mod domain;
mod dto;
mod errors;
mod init;
pub use init::init;
pub mod reach_initializer {
    pub use crate::init;
}
mod serv;
