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

// https://github.com/CleverCloud/lapin

use std::collections::HashMap;
use std::time::Duration;

use tokio::time::sleep;

use bios::basic::config::{BIOSConfig, FrameworkConfig, MQConfig, NoneConfig};
use bios::basic::error::BIOSResult;
use bios::basic::logger::BIOSLogger;
use bios::test::test_container::BIOSTestContainer;
use bios::BIOSFuns;

#[tokio::test]
async fn test_mq_client() -> BIOSResult<()> {
    BIOSLogger::init("").unwrap();
    BIOSTestContainer::rabbit(|url| async move {
        // Default test
        BIOSFuns::init(BIOSConfig {
            ws: NoneConfig {},
            fw: FrameworkConfig {
                app: Default::default(),
                web: Default::default(),
                cache: Default::default(),
                db: Default::default(),
                mq: MQConfig { url },
                adv: Default::default(),
            },
        })
        .await?;

        let client = BIOSFuns::mq();

        let mut header = HashMap::new();
        header.insert("k1".to_string(), "v1".to_string());

        /*let latch_req = CountDownLatch::new(4);
        let latch_cp = latch_req.clone();*/
        client
            .response("test-addr", |(header, msg)| async move {
                println!("response1");
                assert_eq!(header.get("k1").unwrap(), "v1");
                assert_eq!(msg, "测试!");
                // move occurs because ..., which does not implement the `Copy` trait
                //latch_cp.countdown();
                Ok(())
            })
            .await?;

        client
            .response("test-addr", |(header, msg)| async move {
                println!("response2");
                assert_eq!(header.get("k1").unwrap(), "v1");
                assert_eq!(msg, "测试!");
                Ok(())
            })
            .await?;

        client
            .request("test-addr", "测试!".to_owned(), &header)
            .await?;
        client
            .request("test-addr", "测试!".to_owned(), &header)
            .await?;
        client
            .request("test-addr", "测试!".to_owned(), &header)
            .await?;
        client
            .request("test-addr", "测试!".to_owned(), &header)
            .await?;

        client
            .subscribe("test-topic", |(header, msg)| async move {
                println!("subscribe1");
                assert_eq!(header.get("k1").unwrap(), "v1");
                assert_eq!(msg, "测试!");
                Ok(())
            })
            .await?;

        client
            .subscribe("test-topic", |(header, msg)| async move {
                println!("subscribe2");
                assert_eq!(header.get("k1").unwrap(), "v1");
                assert_eq!(msg, "测试!");
                Ok(())
            })
            .await?;

        client
            .publish("test-topic", "测试!".to_owned(), &header)
            .await?;
        client
            .publish("test-topic", "测试!".to_owned(), &header)
            .await?;
        client
            .publish("test-topic", "测试!".to_owned(), &header)
            .await?;
        client
            .publish("test-topic", "测试!".to_owned(), &header)
            .await?;

        sleep(Duration::from_millis(1000)).await;

        Ok(())
    })
    .await
}
