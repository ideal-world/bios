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

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::basic::error::BIOSError;
use crate::basic::result::BIOSResult;

pub struct BIOSJson;

impl BIOSJson {
    pub fn str_to_obj<'a, T: Deserialize<'a>>(&self, str: &'a str) -> BIOSResult<T> {
        let result = serde_json::from_str::<'a, T>(str);
        match result {
            Ok(r) => Ok(r),
            Err(e) => Err(BIOSError::Box(Box::new(e))),
        }
    }

    pub fn str_to_json<'a>(&self, str: &'a str) -> BIOSResult<Value> {
        let result = serde_json::from_str::<'a, Value>(str);
        match result {
            Ok(r) => Ok(r),
            Err(e) => Err(BIOSError::Box(Box::new(e))),
        }
    }

    pub fn json_to_obj<T: DeserializeOwned>(&self, value: Value) -> BIOSResult<T> {
        let result = serde_json::from_value::<T>(value);
        match result {
            Ok(r) => Ok(r),
            Err(e) => Err(BIOSError::Box(Box::new(e))),
        }
    }

    pub fn obj_to_string<T: ?Sized + Serialize>(&self, obj: &T) -> BIOSResult<String> {
        let result = serde_json::to_string(obj);
        match result {
            Ok(r) => Ok(r),
            Err(e) => Err(BIOSError::Box(Box::new(e))),
        }
    }

    pub fn obj_to_json<T: Serialize>(&self, obj: &T) -> BIOSResult<Value> {
        let result = serde_json::to_value(obj);
        match result {
            Ok(r) => Ok(r),
            Err(e) => Err(BIOSError::Box(Box::new(e))),
        }
    }
}
