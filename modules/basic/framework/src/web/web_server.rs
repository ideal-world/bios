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

use log::info;
use poem::listener::TcpListener;
use poem::middleware::{Cors, CorsEndpoint};
use poem::{Endpoint, EndpointExt, IntoEndpoint, Route};
use poem_openapi::{OpenApi, OpenApiService, ServerObject};

use crate::basic::config::{FrameworkConfig, WebServerConfig};
use crate::basic::result::BIOSResult;

pub struct BIOSWebServer {
    app_name: String,
    config: WebServerConfig,
    routes: Vec<Route>,
}

impl BIOSWebServer {
    pub async fn init_by_conf(conf: &FrameworkConfig) -> BIOSResult<BIOSWebServer> {
        Ok(BIOSWebServer {
            app_name: conf.app.name.clone(),
            config: conf.web_server.clone(),
            routes: Vec::default(),
        })
    }

    pub fn add_module<T>(&mut self, code: &str, apis: T) -> &mut Self
    where
        T: OpenApi + 'static,
    {
        let module = self.config.modules.iter().find(|m| m.code == code);
        if module.is_none() {
            panic!("[BIOS.Framework.WebServer] Module {} not found", code);
        }
        let module = module.unwrap();
        info!("[BIOS.Framework.WebServer] Add module {}", module.code);
        let mut api_serv = OpenApiService::new(apis, &module.title, &module.version);
        let ui_serv = api_serv.rapidoc();
        let spec_serv = api_serv.spec();
        for (env, url) in &module.doc_urls {
            api_serv = api_serv.server(ServerObject::new(url).description(env));
        }
        let mut route = Route::new();
        route = route.nest(format!("/{}", module.code), api_serv);
        if let Some(ui_path) = &module.ui_path {
            route = route.at(format!("/{}", ui_path), ui_serv);
        }
        if let Some(spec_path) = &module.spec_path {
            route = route.at(format!("/{}", spec_path), poem::endpoint::make_sync(move |_| spec_serv.clone()));
        }
        // TODO
        // let route = route.with(Cors::new().allow_origin(&self.config.allowed_origin));
        self.routes.push(route);
        self
    }

    pub async fn start(&'static self) -> BIOSResult<()> {
        let mut routes = Route::new();
        for route in self.routes.iter() {
            // TODO
            // info!("[BIOS.Framework.WebServer] Add route {:?}", route);
            routes = routes.nest("/", route);
        }
        poem::Server::new(TcpListener::bind(format!("{}:{}", self.config.host, self.config.port))).run(routes).await?;
        let output_info = format!(
            r#"
    =================
    [BIOS.Framework.WebServer] The {app} application has been launched. Visited at: http://{host}:{port}
    =================
        "#,
            app = self.app_name,
            host = self.config.host,
            port = self.config.port
        );
        info!("{}", output_info);
        Ok(())
    }
}
