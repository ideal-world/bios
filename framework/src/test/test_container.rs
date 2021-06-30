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

use std::future::Future;

use testcontainers::images::generic::WaitFor;
use testcontainers::{clients, images, Docker};

use crate::basic::error::BIOSResult;

pub struct BIOSTestContainer;

impl BIOSTestContainer {
    pub async fn redis<F, T>(fun: F) -> BIOSResult<()>
    where
        F: Fn(String) -> T + Send + Sync + 'static,
        T: Future<Output = BIOSResult<()>> + Send + 'static,
    {
        let docker = clients::Cli::default();
        let node = docker.run(images::redis::Redis::default());
        let port = node
            .get_host_port(6379)
            .expect("Test port acquisition error");
        fun(format!("redis://127.0.0.1:{}/0", port)).await
    }

    pub async fn rabbit<F, T>(fun: F) -> BIOSResult<()>
    where
        F: Fn(String) -> T + Send + Sync + 'static,
        T: Future<Output = BIOSResult<()>> + Send + 'static,
    {
        let docker = clients::Cli::default();
        let node = docker.run(
            images::generic::GenericImage::new("rabbitmq:management")
                .with_wait_for(WaitFor::message_on_stdout("Server startup complete")),
        );
        let port = node
            .get_host_port(5672)
            .expect("Test port acquisition error");
        fun(format!("amqp://guest:guest@127.0.0.1:{}/%2f", port)).await
    }

    pub async fn mysql<F, T>(fun: F) -> BIOSResult<()>
    where
        F: Fn(String) -> T + Send + Sync + 'static,
        T: Future<Output = BIOSResult<()>> + Send + 'static,
    {
        let docker = clients::Cli::default();
        let node = docker.run(
            images::generic::GenericImage::new("mysql")
                .with_env_var("MYSQL_ROOT_PASSWORD", "123456")
                .with_env_var("MYSQL_DATABASE", "test")
                .with_wait_for(WaitFor::message_on_stderr(
                    "port: 3306  MySQL Community Server - GPL",
                )),
        );
        let port = node
            .get_host_port(3306)
            .expect("Test port acquisition error");
        fun(format!("mysql://root:123456@127.0.0.1:{}/test", port)).await
    }
}
