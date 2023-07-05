#![warn(clippy::unwrap_used)]

extern crate lazy_static;

mod api;
pub mod dto;
pub mod reldb_config;
pub mod reldb_constants;
pub mod reldb_initializer;
pub(crate) use crate::reldb_initializer::get_tardis_inst;
mod serv;
