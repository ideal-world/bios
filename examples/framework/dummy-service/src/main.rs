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

use std::sync::Mutex;

use actix_web::{web, App, HttpServer};
use log::info;

use bios_framework::basic::config::{BIOSConfig, NoneConfig};
use bios_framework::basic::logger::BIOSLogger;
use bios_framework::web::web_server::{BIOSWebServer, Init};
use bios_framework::BIOSFuns;

mod controller;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    BIOSLogger::init("").unwrap();
    let conf = BIOSConfig::<NoneConfig>::init("").unwrap();

    let output_info = format!(
        r#"
=========================
----- regular interface -----
POST http://{host}:{port}/normal/<id>?size=?&forward=?
- Path <id> : Any integer type
- Query <size> : (optional) Need to construct the size of the returned packet
- Query <forward> : (optional) The URL to be forwarded
- Body: : (optional) or
    {{
      "body": <any character>
    }}
----- fallback interface -----
POST http://{host}:{port}/fallback/<id>
- Path <id> : Any integer type
----- failure rate dynamic adjustment interface -----
PUT http://{host}:{port}/conf/err_rate/<err_rate>
- Path <err_rate> : The simulated failure rate
=========================
    "#,
        host = conf.fw.web.host,
        port = conf.fw.web.port
    );

    info!("{}", output_info);

    let app_conf = web::Data::new(controller::AppStateContainer {
        err_rate: Mutex::new(0),
    });

    let fw_conf = conf.fw.clone();
    BIOSFuns::init(&fw_conf).await.unwrap();
    HttpServer::new(move || {
        App::new()
            .app_data(app_conf.clone())
            .wrap(BIOSWebServer::init_cors(&fw_conf))
            .wrap(BIOSWebServer::init_error_handlers())
            .wrap(BIOSWebServer::init_logger())
            .service(controller::normal)
            .service(controller::fallback)
            .service(controller::conf_err_rate)
    })
    .init(&conf.fw)
    .unwrap()
    .await
}
