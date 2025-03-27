//! # Pem Struct
//! -----BEGIN BIOS-CERTIFICATION----
//! <base64 encoded json object of [`Certification`]>
//! -----END BIOS-CERTIFICATION----

use std::sync::{Arc, OnceLock};

use ed25519_dalek::VerifyingKey;
use http::{header::CONTENT_TYPE, HeaderValue, StatusCode};
use serde::{Deserialize, Serialize};
use spacegate_shell::{
    hyper::body::Bytes,
    plugin::{
        schema,
        schemars::{self, JsonSchema},
        Inner, Plugin, PluginSchemaExt,
    },
    BoxError, SgBody, SgRequest, SgResponse, SgResponseExt,
};
use tardis::{
    chrono::{DateTime, Utc},
    serde_json, tokio,
};
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct LicensePluginConfig {
    pub verify_key: String,
}

pub struct LicensePlugin {
    pub verify_key: Arc<VerifyingKey>,
    pub certification_info: Arc<tokio::sync::RwLock<Option<CertificationInfo>>>,
}

impl LicensePlugin {
    const HEADER_UPLOAD_LICENSE: &str = "x-bios-upload-license";
}

pub fn mid(pubkey: &str) -> Result<&'static str, BoxError> {
    static MID: OnceLock<Arc<str>> = OnceLock::new();
    let mid = MID.get_or_init(move || {
        machineid_rs::IdBuilder::new(machineid_rs::Encryption::SHA256)
            .add_component(machineid_rs::HWIDComponent::DriveSerial)
            .add_component(machineid_rs::HWIDComponent::MacAddress)
            .build(pubkey)
            // this should not fail because we only extract mac and drive serial number
            .expect("fail to generate machine id")
            .into()
    });
    Ok(mid.as_ref())
}
#[derive(Debug, Serialize, Deserialize)]
pub struct CertificationInfo {
    mid_info: String,
    expire: DateTime<Utc>,
}

impl CertificationInfo {
    pub fn id_expired(&self) -> bool {
        self.expire < Utc::now()
    }
}

schema!(LicensePlugin, LicensePluginConfig);
#[derive(Debug, Serialize, Deserialize)]
pub struct Certification {
    info: String,
    signature: String,
}

impl Certification {
    pub fn verify(&self, vk: &VerifyingKey) -> Result<(), BoxError> {
        use ed25519_dalek::Verifier;
        use std::str::FromStr;
        let signature = ed25519_dalek::Signature::from_str(&self.signature)?;
        vk.verify(self.info.as_bytes(), &signature)?;
        Ok(())
    }
    pub fn from_pem(pem: String) -> Result<Self, BoxError> {
        use base64::prelude::*;
        let chunk = pem.lines().fold(String::new(), |s, line| if line.starts_with('-') { s } else { s + line });
        let json = BASE64_STANDARD.decode(chunk)?;
        let this = serde_json::from_slice(&json)?;
        Ok(this)
    }
    pub fn certification_expired_response() -> SgResponse {
        let mut response = SgResponse::with_code_message(StatusCode::UNAUTHORIZED, "License expired");
        response.headers_mut().insert(CONTENT_TYPE, HeaderValue::from_static("text/html"));
        response
    }
}

//
const PAGE: Bytes = Bytes::from_static(include_bytes!("./license/index.html"));
impl Plugin for LicensePlugin {
    const CODE: &'static str = "license";

    async fn call(&self, req: SgRequest, inner: Inner) -> Result<spacegate_shell::SgResponse, spacegate_shell::plugin::BoxError> {
        if req.headers().get(Self::HEADER_UPLOAD_LICENSE).is_some() {
            let body = req.into_body();
            let body = if !body.is_dumped() { body.dump().await? } else { body };
            let body_bytes = body.get_dumped().expect("should dumped");
            let pem = String::from_utf8(body_bytes.to_vec())?;
            let certification = Certification::from_pem(pem)?;
            let vk = self.verify_key.as_ref();
            certification.verify(vk)?;
            let certification_info: CertificationInfo = serde_json::from_str(&certification.info)?;
            if certification_info.id_expired() {
                return Ok(Certification::certification_expired_response());
            }
            // verify certification
            let mut certification = self.certification_info.write().await;
            *certification = Some(certification_info);
            Ok(SgResponse::with_code_empty(StatusCode::ACCEPTED))
        } else if let Some(has_certification) = self.certification_info.read().await.as_ref() {
            if has_certification.id_expired() {
                {
                    let _ = has_certification;
                }
                self.certification_info.write().await.take();
                return Ok(Certification::certification_expired_response());
            } else {
                return Ok(inner.call(req).await);
            }
        } else {
            // response
            let mut response = SgResponse::new(SgBody::full(PAGE));
            response.headers_mut().insert(CONTENT_TYPE, HeaderValue::from_static("text/html"));
            return Ok(response);
        }
    }

    fn create(plugin_config: spacegate_shell::model::PluginConfig) -> Result<Self, spacegate_shell::plugin::BoxError> {
        use base64::prelude::*;
        let config: LicensePluginConfig = serde_json::from_value(plugin_config.spec)?;
        let vk_base64 = config.verify_key;
        let vk_bytes = BASE64_STANDARD.decode(&vk_base64)?;
        let vk_bytes = vk_bytes.try_into().map_err(|e| format!("invalid verify key {e:X?}"))?;
        let verify_key = VerifyingKey::from_bytes(&vk_bytes)?;
        Ok(LicensePlugin {
            verify_key: Arc::new(verify_key),
            certification_info: Arc::new(tokio::sync::RwLock::new(None)),
        })
    }

    fn schema_opt() -> Option<schemars::schema::RootSchema> {
        Some(Self::schema())
    }
}

#[cfg(test)]
mod test {
    use ed25519_dalek::{ed25519::signature::Signer, SigningKey};
    use tardis::{chrono::TimeDelta, rand};

    use super::*;
    #[test]
    fn gen_pem() {
        let info = CertificationInfo {
            mid_info: "mid".into(),
            expire: Utc::now() + TimeDelta::weeks(999),
        };
        use base64::prelude::*;
        let info = serde_json::to_string(&info).unwrap();
        let sk: SigningKey = SigningKey::from_bytes(&rand::random());
        let vk: VerifyingKey = sk.verifying_key();
        println!("vk: {}", BASE64_STANDARD.encode(vk.to_bytes()));
        let signature = sk.sign(info.as_bytes()).to_string();
        let cert = Certification { info, signature };
        let cert = serde_json::to_string(&cert).unwrap();
        let cert_base64 = BASE64_STANDARD.encode(cert);
        // 80 char per line
        let cert_base64_chunks: Vec<_> = cert_base64.as_bytes().chunks(80).map(|chunk| std::str::from_utf8(chunk).unwrap()).collect();
        let cert_base64 = cert_base64_chunks.join("\n");
        let pem = format!("-----BEGIN BIOS-CERTIFICATION-----\n{cert_base64}\n-----END BIOS-CERTIFICATION-----");
        println!("{pem}");
        let _ = Certification::from_pem(pem.clone()).unwrap();
        std::fs::write("bios-certification.pem", pem).unwrap();
    }
}
