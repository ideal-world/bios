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

use std::collections::HashMap;
use std::future::Future;

use amq_protocol_types::{AMQPValue, LongString, ShortString};
use futures_util::stream::StreamExt;
use lapin::{
    options::*, types::FieldTable, BasicProperties, Channel, Connection, ConnectionProperties,
    Consumer, ExchangeKind,
};
use log::{error, info, trace};
use url::Url;

use crate::basic::config::FrameworkConfig;
use crate::basic::error::{BIOSError, BIOSResult};

// TODO Elegant closure

pub struct BIOSMQClient {
    con: Connection,
}

impl BIOSMQClient {
    pub async fn init_by_conf(conf: &FrameworkConfig) -> BIOSResult<BIOSMQClient> {
        BIOSMQClient::init(&conf.mq.url).await
    }

    pub async fn init(str_url: &str) -> BIOSResult<BIOSMQClient> {
        let url = Url::parse(str_url)?;
        info!(
            "[BIOS.Framework.MQClient] Initializing, host:{}, port:{}",
            url.host_str().unwrap_or(""),
            url.port().unwrap_or(0)
        );
        let con = Connection::connect(
            str_url,
            ConnectionProperties::default().with_connection_name("bios".into()),
        )
        .await?;
        info!(
            "[BIOS.Framework.MQClient] Initialized, host:{}, port:{}",
            url.host_str().unwrap_or(""),
            url.port().unwrap_or(0)
        );
        Ok(BIOSMQClient { con })
    }

    pub async fn request(
        &mut self,
        address: &str,
        message: String,
        header: &HashMap<String, String>,
    ) -> BIOSResult<()> {
        trace!(
            "[BIOS.Framework.MQClient] Request, queue:{}, message:{}",
            address,
            message
        );
        let channel = self.con.create_channel().await?;
        channel
            .confirm_select(ConfirmSelectOptions::default())
            .await?;
        let mut mq_header = FieldTable::default();
        for (k, v) in header {
            mq_header.insert(
                ShortString::from(k.to_owned()),
                AMQPValue::from(LongString::from(v.to_owned())),
            );
        }
        let confirm = channel
            .basic_publish(
                "",
                address,
                BasicPublishOptions::default(),
                message.into_bytes(),
                BasicProperties::default()
                    .with_headers(mq_header)
                    .with_delivery_mode(2),
            )
            .await?
            .await?;
        if confirm.is_ack() {
            Ok(())
        } else {
            Err(BIOSError::InternalError(
                "MQ request confirmation error".to_owned(),
            ))
        }
    }

    pub async fn response<F, T>(&mut self, address: &str, fun: F) -> BIOSResult<()>
    where
        F: FnMut((HashMap<String, String>, String)) -> T + Send + Sync + 'static,
        T: Future<Output = BIOSResult<()>> + Send + 'static,
    {
        info!("[BIOS.Framework.MQClient] Response, queue:{}", address);
        let channel = self.con.create_channel().await?;
        channel
            .queue_declare(
                address,
                QueueDeclareOptions {
                    passive: false,
                    durable: true,
                    exclusive: false,
                    auto_delete: false,
                    nowait: false,
                },
                FieldTable::default(),
            )
            .await?;
        channel.basic_qos(1, BasicQosOptions::default()).await?;
        let consumer = channel
            .basic_consume(
                address,
                "",
                BasicConsumeOptions {
                    no_local: false,
                    no_ack: false,
                    exclusive: false,
                    nowait: false,
                },
                FieldTable::default(),
            )
            .await?;
        self.process(address.to_string(), consumer, fun).await
    }

    pub async fn publish(
        &mut self,
        topic: &str,
        message: String,
        header: &HashMap<String, String>,
    ) -> BIOSResult<()> {
        trace!(
            "[BIOS.Framework.MQClient] Publish, queue:{}, message:{}",
            topic,
            message
        );
        let channel = self.con.create_channel().await?;
        channel
            .confirm_select(ConfirmSelectOptions::default())
            .await?;
        let mut mq_header = FieldTable::default();
        for (k, v) in header {
            mq_header.insert(
                ShortString::from(k.to_owned()),
                AMQPValue::from(LongString::from(v.to_owned())),
            );
        }
        let confirm = channel
            .basic_publish(
                topic,
                "",
                BasicPublishOptions::default(),
                message.into_bytes(),
                BasicProperties::default()
                    .with_headers(mq_header)
                    .with_delivery_mode(2),
            )
            .await?
            .await?;
        if confirm.is_ack() {
            Ok(())
        } else {
            Err(BIOSError::InternalError(
                "MQ request confirmation error".to_owned(),
            ))
        }
    }

    pub async fn subscribe<F, T>(&mut self, topic: &str, fun: F) -> BIOSResult<()>
    where
        F: FnMut((HashMap<String, String>, String)) -> T + Send + Sync + 'static,
        T: Future<Output = BIOSResult<()>> + Send + 'static,
    {
        info!("[BIOS.Framework.MQClient] Subscribe, queue:{}", topic);
        let channel = self.con.create_channel().await?;
        self.declare_exchange(&channel, topic).await?;
        let temp_queue_name = channel
            .queue_declare(
                "",
                QueueDeclareOptions {
                    passive: false,
                    durable: false,
                    exclusive: true,
                    auto_delete: true,
                    nowait: false,
                },
                FieldTable::default(),
            )
            .await?
            .name()
            .to_string();
        channel
            .queue_bind(
                &temp_queue_name,
                topic,
                "",
                QueueBindOptions::default(),
                FieldTable::default(),
            )
            .await?;
        channel.basic_qos(1, BasicQosOptions::default()).await?;
        let consumer = channel
            .basic_consume(
                &temp_queue_name,
                "",
                BasicConsumeOptions {
                    no_local: false,
                    no_ack: false,
                    exclusive: false,
                    nowait: false,
                },
                FieldTable::default(),
            )
            .await?;
        self.process(topic.to_string(), consumer, fun).await
    }

    async fn declare_exchange(&mut self, channel: &Channel, topic: &str) -> BIOSResult<()> {
        channel
            .exchange_declare(
                topic,
                ExchangeKind::Fanout,
                ExchangeDeclareOptions {
                    passive: false,
                    durable: true,
                    auto_delete: false,
                    internal: false,
                    nowait: false,
                },
                FieldTable::default(),
            )
            .await?;
        Ok(())
    }

    async fn process<F, T>(
        &mut self,
        topic_or_address: String,
        mut consumer: Consumer,
        mut fun: F,
    ) -> BIOSResult<()>
    where
        F: FnMut((HashMap<String, String>, String)) -> T + Send + Sync + 'static,
        T: Future<Output = BIOSResult<()>> + Send + 'static,
    {
        async_global_executor::spawn(async move {
            while let Some(delivery) = consumer.next().await {
                match delivery{
                    Ok((_,info)) => {
                        match std::str::from_utf8(&info.data){
                            Ok(msg) => {
                                trace!(
                                    "[BIOS.Framework.MQClient] Receive, queue:{}, message:{}",
                                    topic_or_address,
                                    msg
                                );
                                let mut resp_header: HashMap<String, String> = HashMap::default();
                                &info.properties.headers().as_ref().map(|header| {
                                    for (k, v) in header.into_iter() {
                                        let value = if let AMQPValue::LongString(val) = v {
                                            val.to_string()
                                        } else {
                                            error!(
                                                "[BIOS.Framework.MQClient] Receive, queue:{}, message:{} | MQ Header only supports string types",
                                                topic_or_address,
                                                msg
                                            );
                                            panic!(
                                                "[BIOS.Framework.MQClient] Receive, queue:{}, message:{} | MQ Header only supports string types",
                                                topic_or_address, msg
                                            )
                                        };
                                        resp_header.insert(k.to_string(), value);
                                    }
                                });
                                match fun((resp_header, msg.to_string())).await{
                                    Ok(_) => {
                                        match info.ack(BasicAckOptions::default()).await{
                                            Ok(_) => {
                                                ()
                                            }
                                            Err(e) => {
                                                error!(
                                                    "[BIOS.Framework.MQClient] Receive, queue:{}, message:{} | {}",topic_or_address,msg,e.to_string()
                                                );
                                            }
                                        }
                                    },
                                    Err(e) => {
                                        error!(
                                            "[BIOS.Framework.MQClient] Receive, queue:{}, message:{} | {}",topic_or_address,msg,e.to_string()
                                        );
                                    }
                                }
                            }
                            Err(e) => {
                                error!(
                                    "[BIOS.Framework.MQClient] Receive, queue:{} | {}",topic_or_address,e.to_string()
                                );
                            }
                        }
                    }
                    Err(e) => {
                        error!(
                            "[BIOS.Framework.MQClient] Receive, queue:{} | {}",topic_or_address,e.to_string()
                        );
                    }
                }
            }
        })
        .detach();
        Ok(())
    }
}

impl From<lapin::Error> for BIOSError {
    fn from(error: lapin::Error) -> Self {
        BIOSError::Box(Box::new(error))
    }
}
