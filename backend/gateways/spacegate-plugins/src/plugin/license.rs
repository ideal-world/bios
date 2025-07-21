//! # Pem Struct
//! -----BEGIN BIOS-CERTIFICATION----
//! <base64 encoded json object of [`Certification`]>
//! -----END BIOS-CERTIFICATION----

use std::sync::{Arc, OnceLock};

use http::{header::CONTENT_TYPE, HeaderValue, StatusCode};
use ipnet::IpNet;
use serde::{Deserialize, Serialize};
use spacegate_shell::{
    hyper::body::Bytes,
    kernel::extension::PeerAddr,
    plugin::{
        schema,
        schemars::{self, JsonSchema},
        Inner, Plugin, PluginSchemaExt,
    },
    BoxError, SgBody, SgRequest, SgRequestExt, SgResponse, SgResponseExt,
};
use tardis::{
    chrono::{DateTime, Utc},
    crypto::crypto_sm2_4::TardisCryptoSm2PublicKey,
    serde_json, tokio, TardisFuns,
};
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct LicensePluginConfig {
    pub verify_key: String,
}

pub struct LicensePlugin {
    pub verify_key: Arc<TardisCryptoSm2PublicKey>,
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
    ip_white_list: Option<Vec<IpNet>>,
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
    pub fn verify(&self, vk: &TardisCryptoSm2PublicKey) -> Result<(), BoxError> {
        let result = vk.verify(&self.info, &self.signature)?;
        if !result {
            return Err("signature verify failed".into());
        }
        Ok(())
    }
    pub fn from_pem(pem: String) -> Result<Self, BoxError> {
        let chunk = pem.lines().fold(String::new(), |s, line| if line.starts_with('-') { s } else { s + line });
        let json = TardisFuns::crypto.base64.decode(chunk)?;
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
                // check ip white list
                if let Some(ip_white_list) = &has_certification.ip_white_list {
                    let ip = req.extract::<PeerAddr>();
                    if !ip_white_list.iter().any(|ip_net| ip_net.contains(&ip.0.ip())) {
                        return Ok(SgResponse::with_code_message(StatusCode::UNAUTHORIZED, "Your IP is not in the white list of license."));
                    }
                }
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
        let config: LicensePluginConfig = serde_json::from_value(plugin_config.spec)?;
        let verify_key = TardisCryptoSm2PublicKey::from_public_key_str(&config.verify_key)?;
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
    use super::*;
    use tardis::chrono::TimeDelta;
    use tardis::crypto::crypto_sm2_4::{TardisCryptoSm2, TardisCryptoSm2PrivateKey, TardisCryptoSm2PublicKey};
    #[test]
    fn gen_pem() {
        let info = CertificationInfo {
            mid_info: "mid".into(),
            expire: Utc::now() + TimeDelta::weeks(999),
            ip_white_list: None,
        };
        let info = serde_json::to_string(&info).unwrap();
        let sk = TardisCryptoSm2.new_private_key().unwrap();
        println!("sk: {:#?}", sk.serialize());
        let vk = TardisCryptoSm2.new_public_key(&sk).unwrap();
        println!("vk: {:#?}", vk.serialize());
        let signature = sk.sign(&info).unwrap();
        let cert = Certification { info, signature };
        let cert = serde_json::to_string(&cert).unwrap();
        let cert_base64 = TardisFuns::crypto.base64.encode(cert);
        // 80 char per line
        let cert_base64_chunks: Vec<_> = cert_base64.as_bytes().chunks(80).map(|chunk| std::str::from_utf8(chunk).unwrap()).collect();
        let cert_base64 = cert_base64_chunks.join("\n");
        let pem = format!("-----BEGIN BIOS-CERTIFICATION-----\n{cert_base64}\n-----END BIOS-CERTIFICATION-----");
        println!("{pem}");
        let _ = Certification::from_pem(pem.clone()).unwrap();
        std::fs::write("bios-certification.pem", pem).unwrap();
    }
}
