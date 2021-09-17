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
    use crypto::digest::Digest;
    use crypto::hmac::Hmac;
    use crypto::mac::Mac;
    use crypto::md5::Md5;
    use crypto::sha1::Sha1;
    use crypto::sha2::{Sha256, Sha512};

    pub mod base64 {
        use crate::basic::error::BIOSError;
        use crate::basic::result::BIOSResult;

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

    pub fn digest(str: &str, key: Option<&str>, algorithm: &str) -> String {
        match algorithm.to_lowercase().as_str() {
            "sha1" => {
                let mut sha1 = Sha1::new();
                sha1.input_str(str);
                sha1.result_str()
            }
            "sha256" => {
                let mut sha265 = Sha256::new();
                sha265.input_str(str);
                sha265.result_str()
            }
            "sha512" => {
                let mut sha512 = Sha512::new();
                sha512.input_str(str);
                sha512.result_str()
            }
            "md5" => {
                let mut md5 = Md5::new();
                md5.input_str(str);
                md5.result_str()
            }
            "hmacsha1" => {
                let mut hmac = Hmac::new(Sha1::new(), key.unwrap().as_bytes());
                hmac.input(str.as_bytes());
                String::from_utf8(hmac.result().code().to_vec()).expect("Abstract algorithm conversion error")
            }
            "hmacsha256" => {
                let mut hmac = Hmac::new(Sha256::new(), key.unwrap().as_bytes());
                hmac.input(str.as_bytes());
                String::from_utf8(hmac.result().code().to_vec()).expect("Abstract algorithm conversion error")
            }
            "hmacsha512" => {
                let mut hmac = Hmac::new(Sha512::new(), key.unwrap().as_bytes());
                hmac.input(str.as_bytes());
                String::from_utf8(hmac.result().code().to_vec()).expect("Abstract algorithm conversion error")
            }
            _ => panic!("Digest algorithm [{}] doesn't support", algorithm),
        }
    }
}

pub mod key {

    pub fn generate_token() -> String {
        format!("tk{}", crate::basic::field::uuid())
    }

    pub fn generate_ak() -> String {
        format!("ak{}", crate::basic::field::uuid())
    }

    pub fn generate_sk(ak: &str) -> String {
        let sk = crate::basic::security::digest::digest(format!("{}{}", ak, crate::basic::field::uuid()).as_str(), None, "SHA1");
        format!("sk{}", sk)
    }
}
