#![warn(clippy::unwrap_used)]
#![warn(clippy::dbg_macro)]

pub mod api;
pub mod client;
pub mod config;
pub mod consts;
mod domain;
mod dto;
mod errors;
pub mod init;
mod serv;
