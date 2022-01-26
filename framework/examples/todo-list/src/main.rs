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

use actix_web::{App, HttpServer};

use bios::basic::config::{BIOSConfig, NoneConfig};
use bios::basic::logger::BIOSLogger;
use bios::web::web_server::{BIOSWebServer, Init};
use bios::BIOSFuns;

mod controller;
mod domain;
mod initializer;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "DEBUG");
    BIOSLogger::init("").unwrap();
    let conf = BIOSConfig::<NoneConfig>::init("./examples/framework/todo-list/conf").unwrap();
    BIOSFuns::init(conf).await.unwrap();
    initializer::init().await.unwrap();
    HttpServer::new(move || {
        App::new()
            .wrap(BIOSWebServer::init_cors(&BIOSFuns::fw_config()))
            .wrap(BIOSWebServer::init_error_handlers())
            .wrap(BIOSWebServer::init_logger())
            .service(controller::list_categories)
            .service(controller::add_category)
            .service(controller::modify_category)
            .service(controller::delete_category)
            .service(controller::page_items)
            .service(controller::add_item)
            .service(controller::modify_item)
            .service(controller::delete_item)
    })
    .init(&BIOSFuns::fw_config())
    .unwrap()
    .await
}
