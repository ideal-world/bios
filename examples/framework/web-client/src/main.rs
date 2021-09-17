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

use bios::basic::result::BIOSResult;
use bios::basic::logger::BIOSLogger;
use bios::web::web_client::BIOSWebClient;

#[actix_rt::main]
async fn main() -> BIOSResult<()> {
    BIOSLogger::init("").unwrap();
    let client = BIOSWebClient::init(60, 60)?;
    let mut response = client
        .raw()
        .get("https://www.baidu.com/")
        .insert_header(("User-Agent", "Actix-web"))
        .send()
        .await?;
    println!(
        "Response: {:?}",
        BIOSWebClient::body_as_str(&mut response).await.unwrap()
    );
    Ok(())
}
