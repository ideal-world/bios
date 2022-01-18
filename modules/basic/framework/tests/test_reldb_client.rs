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

// https://github.com/SeaQL/sea-orm

use testcontainers::clients;

use bios::basic::result::BIOSResult;
use bios::db::reldb_client::BIOSRelDBClient;
use bios::test::test_container::BIOSTestContainer;

#[tokio::test]
async fn test_reldb_client() -> BIOSResult<()> {
    let docker = clients::Cli::default();
    let mysql_container = BIOSTestContainer::mysql(None, &docker);
    let port = mysql_container.get_host_port(3306).expect("Test port acquisition error");
    let url = format!("mysql://root:123456@localhost:{}/test", port);

    let client = BIOSRelDBClient::init(&url, 10, 5, None, None).await?;

    Ok(())
}
