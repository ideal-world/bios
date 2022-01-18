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

use std::collections::HashMap;

use log::info;
use redis::aio::Connection;
use redis::{AsyncCommands, RedisError, RedisResult};
use url::Url;

use crate::basic::config::FrameworkConfig;
use crate::basic::error::BIOSError;
use crate::basic::result::BIOSResult;

pub struct BIOSCacheClient {
    con: Connection,
}

impl BIOSCacheClient {
    pub async fn init_by_conf(conf: &FrameworkConfig) -> BIOSResult<BIOSCacheClient> {
        BIOSCacheClient::init(&conf.cache.url).await
    }

    pub async fn init(str_url: &str) -> BIOSResult<BIOSCacheClient> {
        let url = Url::parse(str_url)?;
        info!(
            "[BIOS.Framework.CacheClient] Initializing, host:{}, port:{}, db:{}",
            url.host_str().unwrap_or(""),
            url.port().unwrap_or(0),
            if url.path().is_empty() { "" } else { &url.path()[1..] },
        );
        let client = redis::Client::open(str_url)?;
        let con = client.get_tokio_connection().await?;
        info!(
            "[BIOS.Framework.CacheClient] Initialized, host:{}, port:{}, db:{}",
            url.host_str().unwrap_or(""),
            url.port().unwrap_or(0),
            if url.path().is_empty() { "" } else { &url.path()[1..] },
        );
        Ok(BIOSCacheClient { con })
    }

    // basic operations

    pub async fn set(&mut self, key: &str, value: &str) -> RedisResult<()> {
        self.con.set(key, value).await
    }

    pub async fn set_ex(&mut self, key: &str, value: &str, ex_sec: usize) -> RedisResult<()> {
        self.con.set_ex(key, value, ex_sec).await
    }

    pub async fn set_nx(&mut self, key: &str, value: &str) -> RedisResult<bool> {
        self.con.set_nx(key, value).await
    }

    pub async fn get(&mut self, key: &str) -> RedisResult<Option<String>> {
        self.con.get(key).await
    }

    pub async fn getset(&mut self, key: &str, value: &str) -> RedisResult<Option<String>> {
        self.con.getset(key, value).await
    }

    pub async fn incr(&mut self, key: &str, delta: isize) -> RedisResult<usize> {
        self.con.incr(key, delta).await
    }

    pub async fn del(&mut self, key: &str) -> RedisResult<()> {
        self.con.del(key).await
    }

    pub async fn exists(&mut self, key: &str) -> RedisResult<bool> {
        self.con.exists(key).await
    }

    pub async fn expire(&mut self, key: &str, ex_sec: usize) -> RedisResult<()> {
        self.con.expire(key, ex_sec).await
    }

    pub async fn expire_at(&mut self, key: &str, timestamp_sec: usize) -> RedisResult<()> {
        self.con.expire_at(key, timestamp_sec).await
    }

    pub async fn ttl(&mut self, key: &str) -> RedisResult<usize> {
        self.con.ttl(key).await
    }

    // hash operations

    pub async fn hget(&mut self, key: &str, field: &str) -> RedisResult<Option<String>> {
        self.con.hget(key, field).await
    }

    pub async fn hset(&mut self, key: &str, field: &str, value: &str) -> RedisResult<()> {
        self.con.hset(key, field, value).await
    }

    pub async fn hset_nx(&mut self, key: &str, field: &str, value: &str) -> RedisResult<bool> {
        self.con.hset_nx(key, field, value).await
    }

    pub async fn hdel(&mut self, key: &str, field: &str) -> RedisResult<()> {
        self.con.hdel(key, field).await
    }

    pub async fn hincr(&mut self, key: &str, field: &str, delta: isize) -> RedisResult<usize> {
        self.con.hincr(key, field, delta).await
    }

    pub async fn hexists(&mut self, key: &str, field: &str) -> RedisResult<bool> {
        self.con.hexists(key, field).await
    }

    pub async fn hkeys(&mut self, key: &str) -> RedisResult<Vec<String>> {
        self.con.hkeys(key).await
    }

    pub async fn hvals(&mut self, key: &str) -> RedisResult<Vec<String>> {
        self.con.hvals(key).await
    }

    pub async fn hgetall(&mut self, key: &str) -> RedisResult<HashMap<String, String>> {
        self.con.hgetall(key).await
    }

    pub async fn hlen(&mut self, key: &str) -> RedisResult<usize> {
        self.con.hlen(key).await
    }

    // custom

    pub fn cmd(&mut self) -> &mut Connection {
        &mut self.con
    }
}

impl From<RedisError> for BIOSError {
    fn from(error: RedisError) -> Self {
        BIOSError::Box(Box::new(error))
    }
}
