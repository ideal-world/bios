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
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KINDither express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use derive_more::Display;
use std::convert::Infallible;
use std::error::Error;
use std::num::ParseIntError;
use std::str::Utf8Error;
use std::string::FromUtf8Error;

pub static ERROR_DEFAULT_CODE: &str = "-1";

#[derive(Display, Debug)]
pub enum BIOSError {
    #[display(fmt = "{}##{}", _0, _1)]
    Custom(String, String),
    #[display(fmt = "000000000##{:?}", _0)]
    Box(Box<dyn Error + Send + Sync>),
    #[display(fmt = "500000000##Internal error: {}", _0)]
    InternalError(String),
    #[display(fmt = "501000000##Not implemented error: {}", _0)]
    NotImplemented(String),
    #[display(fmt = "503000000##IO error: {}", _0)]
    IOError(String),
    #[display(fmt = "400000000##Bad Request error: {}", _0)]
    BadRequest(String),
    #[display(fmt = "401000000##Unauthorized error: {}", _0)]
    Unauthorized(String),
    #[display(fmt = "404000000##Not Found error: {}", _0)]
    NotFound(String),
    #[display(fmt = "406000000##Format error: {}", _0)]
    FormatError(String),
    #[display(fmt = "408000000##Timeout error: {}", _0)]
    Timeout(String),
    #[display(fmt = "409000000##Conflict error: {}", _0)]
    Conflict(String),
    #[display(fmt = "{}", _0)]
    _Inner(String),
}

impl From<std::io::Error> for BIOSError {
    fn from(error: std::io::Error) -> Self {
        BIOSError::IOError(error.to_string())
    }
}

impl From<Utf8Error> for BIOSError {
    fn from(error: Utf8Error) -> Self {
        BIOSError::FormatError(error.to_string())
    }
}

impl From<FromUtf8Error> for BIOSError {
    fn from(error: FromUtf8Error) -> Self {
        BIOSError::FormatError(error.to_string())
    }
}

impl From<url::ParseError> for BIOSError {
    fn from(error: url::ParseError) -> Self {
        BIOSError::FormatError(error.to_string())
    }
}

impl From<ParseIntError> for BIOSError {
    fn from(error: ParseIntError) -> Self {
        BIOSError::FormatError(error.to_string())
    }
}

impl From<Infallible> for BIOSError {
    fn from(error: Infallible) -> Self {
        BIOSError::FormatError(error.to_string())
    }
}
