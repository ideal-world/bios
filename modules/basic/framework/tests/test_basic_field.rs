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

use bios::basic::result::BIOSResult;
use bios::BIOSFuns;

#[tokio::test]
async fn test_basic_field() -> BIOSResult<()> {
    assert!(BIOSFuns::field.is_phone("18657120202"));

    assert_eq!(BIOSFuns::field.incr_by_base62("abcd1").unwrap(), "abcd2");
    assert_eq!(BIOSFuns::field.incr_by_base62("abcd12").unwrap(), "abcd13");
    assert_eq!(BIOSFuns::field.incr_by_base62("abcd9").unwrap(), "abceA");
    assert_eq!(BIOSFuns::field.incr_by_base62("azzz9").unwrap(), "azz0A");
    assert_eq!(BIOSFuns::field.incr_by_base62("a9999").unwrap(), "bAAAA");
    assert!(BIOSFuns::field.incr_by_base62("999").is_none());

    assert_eq!(BIOSFuns::field.incr_by_base36("abcd1").unwrap(), "abcd2");
    assert_eq!(BIOSFuns::field.incr_by_base36("abcd12").unwrap(), "abcd13");
    assert_eq!(BIOSFuns::field.incr_by_base36("abcd9").unwrap(), "abcea");
    assert_eq!(BIOSFuns::field.incr_by_base36("azzz9").unwrap(), "azz0a");
    assert_eq!(BIOSFuns::field.incr_by_base36("a9999").unwrap(), "baaaa");
    assert!(BIOSFuns::field.incr_by_base36("999").is_none());

    assert_eq!(BIOSFuns::field.is_code_cs("Adw834_dfds"), true);
    assert_eq!(BIOSFuns::field.is_code_cs(" Adw834_dfds"), false);
    assert_eq!(BIOSFuns::field.is_code_cs("Adw834_d-fds"), false);
    assert_eq!(BIOSFuns::field.is_code_ncs("adon2_43323tr"), true);
    assert_eq!(BIOSFuns::field.is_code_ncs("adon2_43323tr "), false);
    assert_eq!(BIOSFuns::field.is_code_ncs("Adw834_dfds"), false);

    Ok(())
}
