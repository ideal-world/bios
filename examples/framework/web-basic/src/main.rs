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

use actix_web::{App, HttpServer};

use bios_framework::basic::config::{BIOSConfig, NoneConfig};
use bios_framework::basic::logger::BIOSLogger;
use bios_framework::web::web_server::{BIOSWebServer, Init};

mod controller;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    BIOSLogger::init("").unwrap();
    let conf = BIOSConfig::<NoneConfig>::init("").unwrap();
    let fw_conf = conf.fw.clone();
    HttpServer::new(move || {
        App::new()
            .data(BIOSWebServer::init_client(60, 60))
            .wrap(BIOSWebServer::init_logger())
            .wrap(BIOSWebServer::init_cors(&fw_conf))
            .wrap(BIOSWebServer::init_error_handlers())
            .service(controller::hello)
    })
    .init(&conf.fw)
    .unwrap()
    .await
}
