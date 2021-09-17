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

use core::result::Result;
use std::fmt::Display;

use crate::basic::dto::{BIOSContext, BIOSResp};
use crate::basic::error::BIOSError;
use crate::basic::field::GENERAL_SPLIT;

pub type BIOSResult<T> = Result<T, BIOSError>;

pub fn parse<E: Display>(content: E) -> (String, String) {
    let text = content.to_string();
    let split_idx = text.find(GENERAL_SPLIT).expect("The error description format is code##content");
    let code = &text[..split_idx];
    let message = &text[split_idx + 2..];
    (code.to_string(), message.to_string())
}

pub fn output<E: Display>(content: E, context: &BIOSContext) -> BIOSResp<()> {
    let (code, msg) = parse(content);
    BIOSResp {
        code,
        msg,
        body: None,
        trace_id: Some(context.trace.id.to_string()),
        trace_app: Some(context.trace.app.to_string()),
        trace_inst: Some(context.trace.inst.to_string()),
    }
}
