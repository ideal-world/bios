/*
 * Copyright 2021. gudaoxuri
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

#[cfg(feature = "web-server")]
pub mod basic_processor;
#[cfg(feature = "web-server")]
pub mod error_handler;
#[cfg(feature = "web-server")]
pub mod resp_handler;
#[cfg(feature = "web-client")]
pub mod web_client;
#[cfg(feature = "web-server")]
pub mod web_server;
