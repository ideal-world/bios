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

use bios::basic::error::BIOSResult;

#[tokio::test]
async fn test_basic_uri() -> BIOSResult<()> {
    assert_eq!(
        bios::basic::uri::format("http://idealwrold.group").unwrap(),
        "http://idealwrold.group"
    );
    assert_eq!(
        bios::basic::uri::format("jdbc:h2:men:iam").unwrap(),
        "jdbc:h2:men:iam"
    );
    assert_eq!(
        bios::basic::uri::format("api://a1.t1/e1?q2=2&q1=1&q3=3").unwrap(),
        "api://a1.t1/e1?q1=1&q2=2&q3=3"
    );
    Ok(())
}
