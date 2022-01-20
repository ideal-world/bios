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
use std::path::Path;
use std::str::FromStr;
use std::sync::Mutex;

use log::{LevelFilter, SetLoggerError};
use log4rs;
use log4rs::append::console::ConsoleAppender;
use log4rs::config::runtime::ConfigErrors;
use log4rs::config::{Appender, Root};
use log4rs::encode::pattern::PatternEncoder;

use crate::basic::error::{BIOSError, ERROR_DEFAULT_CODE};
use crate::basic::fetch_profile;
use crate::basic::result::BIOSResult;

lazy_static! {
    static ref INITIALIZED: Mutex<bool> = Mutex::new(false);
}

pub(crate) struct BIOSLogger;

impl BIOSLogger {
    pub(crate) fn init(root_path: &str) -> BIOSResult<()> {
        let mut initialized = INITIALIZED.lock().unwrap();
        if *initialized == true {
            return Ok(());
        }
        *initialized = true;

        let default_level = &env::var("RUST_LOG").unwrap_or("INFO".to_string());

        tracing_subscriber::fmt().with_max_level(tracing::Level::from_str(default_level).unwrap()).with_test_writer().init();
        Ok(())

        // TODO 与 tracing_subscriber 冲突

        // let profile = fetch_profile();
        // let conf_file = Path::new(root_path).join(&format!("log-{}.yaml", profile));
        // let root_level = match LevelFilter::from_str(default_level) {
        //     Ok(l) => l,
        //     Err(_) => LevelFilter::Info,
        // };
        // if conf_file.is_file() {
        //     match log4rs::init_file(Path::new(root_path).join(&format!("log-{}.yaml", profile)), Default::default()) {
        //         Ok(_) => Ok(()),
        //         Err(e) => Err(BIOSError::Custom(ERROR_DEFAULT_CODE.to_string(), e.to_string())),
        //     }
        // } else {
        //     let stdout = ConsoleAppender::builder().encoder(Box::new(PatternEncoder::new("[{l}] {d} {T} [{t}] {X(requestId, user_id)} - {m}{n}"))).build();
        //     let conf_default =
        //         log4rs::config::Config::builder().appender(Appender::builder().build("stdout", Box::new(stdout))).build(Root::builder().appender("stdout").build(root_level))?;
        //     log4rs::init_config(conf_default)?;
        //     Ok(())
        // }
    }
}

impl From<ConfigErrors> for BIOSError {
    fn from(error: ConfigErrors) -> Self {
        BIOSError::Box(Box::new(error))
    }
}

impl From<SetLoggerError> for BIOSError {
    fn from(error: SetLoggerError) -> Self {
        BIOSError::Box(Box::new(error))
    }
}
