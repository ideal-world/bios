/*
 * Copyright 2022. the original author or authors.
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

use std::env;

use bios::basic::config::NoneConfig;
use bios::basic::result::BIOSResult;
use bios::BIOSFuns;

use crate::processor::Api;

mod processor;

///
/// Visit: http://127.0.0.1:8089/ui
///
#[tokio::main]
async fn main() -> BIOSResult<()> {
    env::set_var("RUST_LOG", "debug");
    env::set_var("PROFILE", "default");
    // Initial configuration
    BIOSFuns::init::<NoneConfig>("config").await?;
    // Register the processor and start the web service
    BIOSFuns::web_server().add_module("", Api).start().await
}
