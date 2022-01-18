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

use std::fmt;

use actix_cors::Cors;
use actix_http::{Request, Response};
use actix_service::{IntoServiceFactory, Service, ServiceFactory};
use actix_web::body::MessageBody;
use actix_web::dev::{AppConfig, Server};
use actix_web::middleware::Logger;
use actix_web::HttpServer;
use log::info;

use crate::basic::config::FrameworkConfig;
use crate::basic::result::BIOSResult;
use crate::web::error_handler::WebErrorHandler;

pub struct BIOSWebServer;

impl BIOSWebServer {
    pub fn init_logger() -> Logger {
        Logger::default()
    }

    pub fn init_cors(conf: &FrameworkConfig) -> Cors {
        let mut cors = Cors::default().supports_credentials().max_age(3600);
        cors = if conf.web.allowed_origin == "*" {
            cors.send_wildcard()
        } else {
            cors.allowed_origin(&conf.web.allowed_origin.clone())
        };
        cors
    }

    pub fn init_error_handlers() -> WebErrorHandler {
        WebErrorHandler
    }
}

pub trait Init {
    fn init(self, conf: &FrameworkConfig) -> BIOSResult<Server>;
}

impl<F, I, S, B> Init for HttpServer<F, I, S, B>
where
    F: Fn() -> I + Send + Clone + 'static,
    I: IntoServiceFactory<S, Request> + 'static,
    S: ServiceFactory<Request, Config = AppConfig> + 'static,
    S::Error: Into<actix_web::Error> + 'static,
    S::InitError: fmt::Debug,
    S::Response: Into<Response<B>> + 'static,
    B: MessageBody + 'static,
    B::Error: Into<Box<dyn std::error::Error>>,
    <S::Service as Service<Request>>::Future: 'static,
    S::Service: 'static,
{
    fn init(self, conf: &FrameworkConfig) -> BIOSResult<Server> {
        let server = self.bind(((&conf.web.host).clone(), conf.web.port))?.run();
        let output_info = format!(
            r#"
=================
The {app} application has been launched. Visited at: http://{host}:{port}
=================
    "#,
            app = conf.app.name,
            host = conf.web.host,
            port = conf.web.port
        );
        info!("{}", output_info);
        Ok(server)
    }
}
