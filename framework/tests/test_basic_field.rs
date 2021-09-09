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
async fn test_basic_field() -> BIOSResult<()> {
    assert_eq!(bios::basic::field::incr_by_base62("abcd1").unwrap(), "abcd2");
    assert_eq!(bios::basic::field::incr_by_base62("abcd12").unwrap(), "abcd13");
    assert_eq!(bios::basic::field::incr_by_base62("abcd9").unwrap(), "abceA");
    assert_eq!(bios::basic::field::incr_by_base62("azzz9").unwrap(), "azz0A");
    assert_eq!(bios::basic::field::incr_by_base62("a9999").unwrap(), "bAAAA");
    assert!(bios::basic::field::incr_by_base62("999").is_none());

    assert_eq!(bios::basic::field::incr_by_base36("abcd1").unwrap(), "abcd2");
    assert_eq!(bios::basic::field::incr_by_base36("abcd12").unwrap(), "abcd13");
    assert_eq!(bios::basic::field::incr_by_base36("abcd9").unwrap(), "abcea");
    assert_eq!(bios::basic::field::incr_by_base36("azzz9").unwrap(), "azz0a");
    assert_eq!(bios::basic::field::incr_by_base36("a9999").unwrap(), "baaaa");
    assert!(bios::basic::field::incr_by_base36("999").is_none());

    assert_eq!(bios::basic::field::R_CODE_CS.is_match("Adw834_dfds"), true);
    assert_eq!(bios::basic::field::R_CODE_CS.is_match(" Adw834_dfds"), false);
    assert_eq!(bios::basic::field::R_CODE_CS.is_match("Adw834_d-fds"), false);
    assert_eq!(bios::basic::field::R_CODE_NCS.is_match("adon2_43323tr"), true);
    assert_eq!(bios::basic::field::R_CODE_NCS.is_match("adon2_43323tr "), false);
    assert_eq!(bios::basic::field::R_CODE_NCS.is_match("Adw834_dfds"), false);

    Ok(())
}
