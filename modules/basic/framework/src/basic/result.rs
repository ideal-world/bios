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

use core::result::Result;
use std::fmt::Display;

use derive_more::Display;

use crate::basic::error::BIOSError;
use crate::basic::field::GENERAL_SPLIT;

pub type BIOSResult<T> = Result<T, BIOSError>;

#[derive(Display, Debug)]
pub enum StatusCodeKind {
    #[display(fmt = "200")]
    Success,
    #[display(fmt = "000")]
    UnKnown,
    #[display(fmt = "400")]
    BadRequest,
    #[display(fmt = "401")]
    Unauthorized,
    #[display(fmt = "404")]
    NotFound,
    #[display(fmt = "406")]
    FormatError,
    #[display(fmt = "408")]
    Timeout,
    #[display(fmt = "409")]
    Conflict,
    #[display(fmt = "419")]
    ConflictExists,
    #[display(fmt = "429")]
    ConflictExistFieldsAtSomeTime,
    #[display(fmt = "439")]
    ConflictExistAssociatedData,
    #[display(fmt = "500")]
    InternalError,
    #[display(fmt = "501")]
    NotImplemented,
    #[display(fmt = "503")]
    IOError,
}

#[derive(Display, Debug)]
pub enum ActionKind {
    #[display(fmt = "01")]
    Create,
    #[display(fmt = "02")]
    Modify,
    #[display(fmt = "03")]
    FetchOne,
    #[display(fmt = "04")]
    FetchList,
    #[display(fmt = "05")]
    Delete,
    #[display(fmt = "06")]
    Exists,
}

pub fn parse<E: Display>(content: E) -> (String, String) {
    let text = content.to_string();
    let split_idx = text.find(GENERAL_SPLIT).expect("Illegal error description format");
    let code = &text[..split_idx];
    let message = &text[split_idx + 2..];
    (code.to_string(), message.to_string())
}
