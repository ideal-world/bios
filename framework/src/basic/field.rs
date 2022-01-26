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
use regex::Regex;
use uuid::Uuid;

lazy_static! {
    static ref R_PHONE: Regex = Regex::new(r"^1(3\d|4[5-9]|5[0-35-9]|6[2567]|7[0-8]|8\d|9[0-35-9])\d{8}$").unwrap();
    static ref R_MAIL: Regex =
        Regex::new(r"^[a-zA-Z0-9.!#$%&'*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$").unwrap();
    static ref R_CODE_NCS: Regex = Regex::new(r"^[a-z0-9_]+$").unwrap();
    static ref R_CODE_CS: Regex = Regex::new(r"^[A-Za-z0-9_]+$").unwrap();
}

static BASE62: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
static BASE36: &str = "abcdefghijklmnopqrstuvwxyz0123456789";

pub static GENERAL_SPLIT: &'static str = "##";

pub struct BIOSField;

impl BIOSField {
    pub fn is_phone(&self, phone: &str) -> bool {
        R_PHONE.is_match(phone)
    }

    pub fn is_mail(&self, mail: &str) -> bool {
        R_MAIL.is_match(mail)
    }

    pub fn is_code_cs(&self, str: &str) -> bool {
        R_CODE_CS.is_match(str)
    }

    pub fn is_code_ncs(&self, str: &str) -> bool {
        R_CODE_NCS.is_match(str)
    }

    pub fn uuid(&self) -> Uuid {
        uuid::Uuid::new_v4()
    }

    pub fn uuid_str(&self) -> String {
        uuid::Uuid::new_v4().to_simple().to_string()
    }

    pub fn incr_by_base62(&self, str: &str) -> Option<String> {
        self.incr_by(str, BASE62)
    }

    pub fn incr_by_base36(&self, str: &str) -> Option<String> {
        self.incr_by(str, BASE36)
    }

    pub fn incr_by(&self, str: &str, chars: &str) -> Option<String> {
        let mut result = Vec::new();
        let mut up = true;
        for x in str.chars().rev() {
            if !up {
                result.push(x.to_string());
                continue;
            }
            let idx = chars.find(x).unwrap();
            if idx == chars.len() - 1 {
                up = true;
                result.push(chars[..1].to_string());
            } else {
                up = false;
                result.push(chars[idx + 1..idx + 2].to_string());
            }
        }
        if !up {
            result.reverse();
            Some(result.join(""))
        } else {
            None
        }
    }
}
