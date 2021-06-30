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

use std::error::Error;
use std::fmt;
use std::fmt::Display;
use std::num::ParseIntError;
use std::str::Utf8Error;
use std::string::FromUtf8Error;

pub static ERROR_DEFAULT_CODE: &str = "-1";

pub type BIOSResult<T> = Result<T, BIOSError>;

#[derive(Debug)]
pub enum BIOSError {
    E(String, String),
    Box(Box<dyn Error + Send + Sync>),
    InternalError(String),
    IOError(String),
    FormatError(String),
    BadRequest(String),
    Unauthorized(String),
    NotFound(String),
    Conflict(String),
    NotImplemented(String),
    Timeout(String),
    ValidationError(String),
}

impl Error for BIOSError {}

impl Display for BIOSError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BIOSError::Box(e) => write!(f, "{:?}", e),
            BIOSError::E(code, msg) => write!(f, "{}:{}", code, msg),
            BIOSError::InternalError(s) => write!(f, "{}", s),
            BIOSError::IOError(s) => write!(f, "{}", s),
            BIOSError::FormatError(s) => write!(f, "{}", s),
            BIOSError::BadRequest(s) => write!(f, "{}", s),
            BIOSError::Unauthorized(s) => write!(f, "{}", s),
            BIOSError::NotFound(s) => write!(f, "{}", s),
            BIOSError::Conflict(s) => write!(f, "{}", s),
            BIOSError::NotImplemented(s) => write!(f, "{}", s),
            BIOSError::Timeout(s) => write!(f, "{}", s),
            BIOSError::ValidationError(s) => write!(f, "{}", s),
        }
    }
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
