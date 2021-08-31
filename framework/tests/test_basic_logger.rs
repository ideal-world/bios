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

// https://github.com/estk/log4rs
// https://blog.csdn.net/s_lisheng/article/details/78271032

use std::env;

use log::info;

use bios::basic::error::BIOSResult;
use bios::basic::logger::BIOSLogger;

use crate::app::req::test_req;

#[tokio::test]
async fn test_basic_logger() -> BIOSResult<()> {
    // env::set_var("RUST_LOG", "OFF");
    env::set_var("PROFILE", "test");
    // BIOSLogger::init("tests/log")?;
    // 配置文件不存在，使用默认配置
    BIOSLogger::init("")?;
    info!("info...");
    test_req();
    Ok(())
}

mod app {
    pub mod req {
        use log::error;

        pub fn test_req() {
            error!("test error");
        }
    }
}
