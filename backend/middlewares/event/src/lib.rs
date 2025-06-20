#![warn(clippy::unwrap_used, clippy::dbg_macro)]
extern crate lazy_static;
mod api;
mod domain;
pub mod dto;
pub mod event_config;
pub mod event_constants;
pub mod event_initializer;
pub mod mq_adapter;
mod serv;

// fix `instrument` find tracing error [issue](https://github.com/tokio-rs/tracing/issues/3309)
use tardis::tracing::*;
extern crate self as tracing;
