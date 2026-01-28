#![warn(clippy::unwrap_used, clippy::todo, clippy::unimplemented)]

extern crate lazy_static;

mod api;
pub mod dto;
mod serv;
pub mod stats_config;
pub mod stats_constants;
pub mod stats_enumeration;
pub mod stats_initializer;
pub(crate) use crate::stats_initializer::get_tardis_inst;

// fix `instrument` find tracing error [issue](https://github.com/tokio-rs/tracing/issues/3309)
use tardis::tracing::*;
extern crate self as tracing;
