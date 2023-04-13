pub mod base64 {
    use base64::engine::general_purpose;
    use base64::Engine;

    use crate::mini_tardis::basic::TardisResult;
    use crate::mini_tardis::error::TardisError;

    pub fn decode(data: &str) -> TardisResult<String> {
        match general_purpose::STANDARD.decode(data) {
            Ok(result) => Ok(String::from_utf8(result)?),
            Err(error) => Err(TardisError::format_error(
                &format!("[Tardis.Crypto] Base64 decode error:{error}"),
                "406-tardis-crypto-base64-decode-error",
            )),
        }
    }

    pub fn encode(data: &str) -> String {
        general_purpose::STANDARD.encode(data)
    }
}

pub mod key {
    use rand::RngCore;

    use crate::mini_tardis::basic::TardisResult;

    pub fn rand_16_hex() -> TardisResult<String> {
        let mut key: [u8; 8] = [0; 8];
        rand::rngs::OsRng::default().fill_bytes(&mut key);
        Ok(hex::encode(key))
    }

    pub fn rand_16_hex_by_str(str: &str) -> TardisResult<String> {
        Ok(hex::encode(str)[..16].to_string())
    }
}

pub mod sm {
    use libsm::{
        sm2::ecc::Point,
        sm2::encrypt::{DecryptCtx, EncryptCtx},
        sm2::signature::SigCtx,
        sm3::hash::Sm3Hash,
        sm4::{Cipher, Mode},
    };
    use num_bigint::BigUint;

    use crate::mini_tardis::{basic::TardisResult, error::TardisError};

    pub fn digest(data: &str) -> TardisResult<String> {
        Ok(hex::encode(Sm3Hash::new(data.as_bytes()).get_hash()))
    }

    pub struct TardisCryptoSm4;

    pub struct TardisCryptoSm2;

    pub struct TardisCryptoSm2PrivateKey {
        pri_key: BigUint,
    }

    pub struct TardisCryptoSm2PublicKey {
        pub_key: Point,
    }

    impl TardisCryptoSm2 {
        pub fn new_private_key(&self) -> TardisResult<TardisCryptoSm2PrivateKey> {
            TardisCryptoSm2PrivateKey::new()
        }

        pub fn new_public_key(&self, private_key: &TardisCryptoSm2PrivateKey) -> TardisResult<TardisCryptoSm2PublicKey> {
            TardisCryptoSm2PublicKey::from_private_key(private_key)
        }

        pub fn new_public_key_from_public_key(&self, public_key: &str) -> TardisResult<TardisCryptoSm2PublicKey> {
            TardisCryptoSm2PublicKey::from_public_key_str(public_key)
        }
    }

    impl TardisCryptoSm2PrivateKey {
        pub fn new() -> TardisResult<Self> {
            let (_, sk) = SigCtx::new()
                .new_keypair()
                .map_err(|error| TardisError::internal_error(&format!("[Tardis.Crypto] SM2 new keypair error:{error}"), "500-tardis-crypto-sm2-keypair-error"))?;
            Ok(TardisCryptoSm2PrivateKey { pri_key: sk })
        }

        pub fn from(private_key: &str) -> TardisResult<Self> {
            let sk = SigCtx::new()
                .load_seckey(&hex::decode(private_key)?)
                .map_err(|error| TardisError::format_error(&format!("[Tardis.Crypto] SM2 load sk error:{error}"), "406-tardis-crypto-sm2-sk-error"))?;
            Ok(TardisCryptoSm2PrivateKey { pri_key: sk })
        }

        pub fn serialize(&self) -> TardisResult<String> {
            let sk = SigCtx::new()
                .serialize_seckey(&self.pri_key)
                .map_err(|error| TardisError::format_error(&format!("[Tardis.Crypto] SM2 serialize sk error:{error}"), "406-tardis-crypto-sm2-sk-error"))?;
            Ok(hex::encode(sk))
        }

        pub fn decrypt(&self, encrypted_data: &str) -> TardisResult<String> {
            let encrypted_data = hex::decode(encrypted_data)?;
            // https://github.com/citahub/libsm/issues/46
            let data = DecryptCtx::new(encrypted_data.len() - 97, self.pri_key.clone())
                .decrypt(&encrypted_data)
                .map_err(|error| TardisError::format_error(&format!("[Tardis.Crypto] SM2 decrypt error:{error}"), "406-tardis-crypto-sm2-decrypt-error"))?;
            Ok(String::from_utf8(data)?)
        }
    }

    impl TardisCryptoSm2PublicKey {
        pub fn from_private_key(private_key: &TardisCryptoSm2PrivateKey) -> TardisResult<Self> {
            let pk = SigCtx::new()
                .pk_from_sk(&private_key.pri_key)
                .map_err(|error| TardisError::format_error(&format!("[Tardis.Crypto] SM2 get pk error:{error}"), "406-tardis-crypto-sm2-pk-error"))?;
            Ok(TardisCryptoSm2PublicKey { pub_key: pk })
        }

        pub fn from_public_key_str(public_key: &str) -> TardisResult<Self> {
            let pk = SigCtx::new()
                .load_pubkey(&hex::decode(public_key)?)
                .map_err(|error| TardisError::format_error(&format!("[Tardis.Crypto] SM2 load pk error:{error}"), "406-tardis-crypto-sm2-pk-error"))?;
            Ok(TardisCryptoSm2PublicKey { pub_key: pk })
        }

        pub fn serialize(&self) -> TardisResult<String> {
            let pk = SigCtx::new()
                .serialize_pubkey(&self.pub_key, true)
                .map_err(|error| TardisError::format_error(&format!("[Tardis.Crypto] SM2 serialize pk error:{error}"), "406-tardis-crypto-sm2-pk-error"))?;
            Ok(hex::encode(pk))
        }

        pub fn encrypt(&self, data: &str) -> TardisResult<String> {
            let encrypted_data = EncryptCtx::new(data.len(), self.pub_key)
                .encrypt(data.as_bytes())
                .map_err(|error| TardisError::format_error(&format!("[Tardis.Crypto] SM2 encrypt error:{error}"), "406-tardis-crypto-sm2-encrypt-error"))?;
            Ok(hex::encode(encrypted_data))
        }
    }

    impl TardisCryptoSm4 {
        pub fn encrypt_cbc(&self, data: &str, hex_key: &str, hex_iv: &str) -> TardisResult<String> {
            let cipher = Cipher::new(hex_key.as_bytes(), Mode::Cbc)
                .map_err(|error| TardisError::format_error(&format!("[Tardis.Crypto] SM4 new cipher error:{error}"), "406-tardis-crypto-sm4-cipher-error"))?;
            let encrypted_data = cipher
                .encrypt(data.as_bytes(), hex_iv.as_bytes())
                .map_err(|error| TardisError::format_error(&format!("[Tardis.Crypto] SM4 encrypt error:{error}"), "406-tardis-crypto-sm4-encrypt-error"))?;
            Ok(hex::encode(encrypted_data))
        }

        pub fn decrypt_cbc(&self, encrypted_data: &str, hex_key: &str, hex_iv: &str) -> TardisResult<String> {
            let cipher = Cipher::new(hex_key.as_bytes(), Mode::Cbc)
                .map_err(|error| TardisError::format_error(&format!("[Tardis.Crypto] SM4 new cipher error:{error}"), "406-tardis-crypto-sm4-cipher-error"))?;
            let encrypted_data = hex::decode(encrypted_data)?;
            let data = cipher
                .decrypt(encrypted_data.as_slice(), hex_iv.as_bytes())
                .map_err(|error| TardisError::format_error(&format!("[Tardis.Crypto] SM4 decrypt error:{error}"), "406-tardis-crypto-sm4-decrypt-error"))?;
            Ok(String::from_utf8(data)?)
        }
    }
}
