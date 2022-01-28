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
use std::time::Duration;

use testcontainers::clients;
use tokio::time::sleep;

use bios::basic::config::NoneConfig;
use bios::basic::result::BIOSResult;
use bios::test::test_container::BIOSTestContainer;
use bios::BIOSFuns;

#[tokio::main]
async fn main() -> BIOSResult<()> {
    // Here is a demonstration of using docker to start a mysql simulation scenario.
    let docker = clients::Cli::default();
    let redis_container = BIOSTestContainer::redis_custom(&docker);
    let port = redis_container.get_host_port(6379).expect("Test port acquisition error");
    let url = format!("redis://127.0.0.1:{}/0", port);

    env::set_var("RUST_LOG", "debug");
    env::set_var("PROFILE", "default");
    env::set_var("BIOS_CACHE.URL", url);

    // Initial configuration
    BIOSFuns::init::<NoneConfig>("config").await?;

    let client = BIOSFuns::cache();

    // --------------------------------------------------

    // basic operations

    let mut opt_value = client.get("test_key").await?;
    assert_eq!(opt_value, None);

    client.set("test_key", "测试").await?;
    let mut str_value = client.get("test_key").await?.unwrap();
    assert_eq!(str_value, "测试");

    let mut set_result = client.set_nx("test_key", "测试2").await?;
    assert_eq!(set_result, false);
    client.get("test_key").await?;
    assert_eq!(str_value, "测试");
    set_result = client.set_nx("test_key_nx", "测试2").await?;
    assert_eq!(set_result, true);
    str_value = client.get("test_key_nx").await?.unwrap();
    assert_eq!(str_value, "测试2");

    client.expire("test_key_nx", 1).await?;
    client.set_ex("test_key_ex", "测试3", 1).await?;
    str_value = client.get("test_key_ex").await?.unwrap();
    assert_eq!(str_value, "测试3");
    let mut bool_value = client.exists("test_key_ex").await?;
    assert_eq!(bool_value, true);
    sleep(Duration::from_millis(1200)).await;
    opt_value = client.get("test_key_ex").await?;
    assert_eq!(opt_value, None);
    bool_value = client.exists("test_key_ex").await?;
    assert_eq!(bool_value, false);
    bool_value = client.exists("test_key_nx").await?;
    assert_eq!(bool_value, false);

    opt_value = client.getset("test_key_none", "孤岛旭日").await?;
    assert_eq!(opt_value, None);
    opt_value = client.getset("test_key_none", "idealworld").await?;
    assert_eq!(opt_value.unwrap(), "孤岛旭日");

    client.del("test_key_none1").await?;
    client.del("test_key_none").await?;
    bool_value = client.exists("test_key_none").await?;
    assert_eq!(bool_value, false);

    let mut num_value = client.incr("incr", 1).await?;
    assert_eq!(num_value, 1);
    num_value = client.incr("incr", 1).await?;
    assert_eq!(num_value, 2);
    num_value = client.incr("incr", -1).await?;
    assert_eq!(num_value, 1);

    client.expire_at("test_key_xp", 1893430861).await?;
    num_value = client.ttl("test_key_xp").await?;
    println!("Expire AT : {}", num_value);
    assert!(num_value > 0);

    // hash operations

    client.hset("h", "f1", "v1").await?;
    client.hset("h", "f2", "v2").await?;
    assert_eq!(client.hget("h", "f0").await?, None);
    assert_eq!(client.hget("h", "f1").await?.unwrap(), "v1");

    assert_eq!(client.hexists("h", "f1").await?, true);
    client.hdel("h", "f1").await?;
    assert_eq!(client.hexists("h", "f1").await?, false);

    assert_eq!(client.hset_nx("h", "f0", "v0").await?, true);
    assert_eq!(client.hset_nx("h", "f0", "v0").await?, false);

    assert_eq!(client.hincr("h", "f3", 1).await?, 1);
    assert_eq!(client.hincr("h", "f3", 1).await?, 2);
    assert_eq!(client.hincr("h", "f3", -1).await?, 1);

    assert_eq!(client.hkeys("h").await?, vec!("f2", "f0", "f3"));
    assert_eq!(client.hvals("h").await?, vec!("v2", "v0", "1"));

    assert_eq!(client.hlen("h").await?, 3);

    let map_result = client.hgetall("h").await?;
    assert_eq!(map_result.len(), 3);
    assert_eq!(map_result.get("f2").unwrap(), "v2");
    assert_eq!(map_result.get("f0").unwrap(), "v0");
    assert_eq!(map_result.get("f3").unwrap(), "1");

    Ok(())
}
