#![warn(clippy::unwrap_used)]
mod api;
pub mod dto;
pub mod kv_config;
pub mod kv_constants;
pub mod kv_initializer;
pub(crate) use crate::kv_initializer::get_tardis_inst;
pub mod event;
mod serv;

// fix `instrument` find tracing error [issue](https://github.com/tokio-rs/tracing/issues/3309)
use tardis::tracing::*;
extern crate self as tracing;
