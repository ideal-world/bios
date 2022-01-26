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

use crate::BIOSFuns;
use poem_openapi::Validator;
use std::fmt::{Display, Formatter};

pub struct Phone;
pub struct Mail;

impl Display for Phone {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("Invalid phone number format")
    }
}

impl Display for Mail {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("Invalid mail format")
    }
}

impl Validator<String> for Phone {
    fn check(&self, value: &String) -> bool {
        BIOSFuns::field.is_phone(value)
    }
}

impl Validator<String> for Mail {
    fn check(&self, value: &String) -> bool {
        BIOSFuns::field.is_mail(value)
    }
}
