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

pub mod digest {

    pub mod base64 {
        use crate::basic::error::{BIOSError, BIOSResult};

        pub fn decode(str: &str) -> BIOSResult<String> {
            match base64::decode(str) {
                Ok(result) => Ok(String::from_utf8(result).expect("Vec[] to String error")),
                Err(e) => Err(BIOSError::FormatError(e.to_string())),
            }
        }

        pub fn encode(str: &str) -> String {
            base64::encode(str)
        }
    }
}
