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

use bios::basic::config::{BIOSConfig, NoneConfig};
use bios::basic::logger::BIOSLogger;
use bios::web::web_server::{BIOSWebServer, Init};
use bios::BIOSFuns;

mod controller;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    BIOSLogger::init("").unwrap();
    let conf = BIOSConfig::<NoneConfig>::init("").unwrap();
    BIOSFuns::init(conf).await.unwrap();
    HttpServer::new(move || {
        App::new()
            .wrap(BIOSWebServer::init_cors(
                &BIOSFuns::config::<NoneConfig>().fw,
            ))
            .wrap(BIOSWebServer::init_error_handlers())
            .wrap(BIOSWebServer::init_logger())
            .service(controller::hello)
    })
    .init(&BIOSFuns::config::<NoneConfig>().fw)
    .unwrap()
    .await
}
