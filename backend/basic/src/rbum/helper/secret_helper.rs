//! Helper for the storage encryption of the sensitive relationship attributes
//!
//! 关联关系敏感属性（``rbum_rel_attr.value``）的存储加解密工具
//!
//! Rules:
//! 1. Only the attributes whose [`crate::rbum::domain::rbum_kind_attr::Model::secret`] is ``true`` are encrypted when storing.
//! 1. All the reading paths return the plaintext value.
//!
//! 规则：
//! 1. 仅对 [`crate::rbum::domain::rbum_kind_attr::Model::secret`] 为 ``true`` 的属性做加密存储。
//! 1. 所有读取路径都返回明文值。

use tardis::basic::error::TardisError;
use tardis::basic::result::TardisResult;
use tardis::log::{error, warn};
use tardis::{TardisFuns, TardisFunsInst};

use crate::rbum::rbum_config::RbumConfigManager;

/// Ciphertext prefix, the value without this prefix is treated as plaintext
///
/// 密文前缀，不带此前缀的值按明文处理（兼容存量数据）
pub const SECRET_ENCRYPTED_PREFIX: &str = "ENC(sm4v1:";

/// The SM4 key/iv length of tardis, the key content is used as bytes directly
///
/// tardis 的 SM4 密钥/初始向量长度，密钥内容直接作为字节使用
const SM4_KEY_LEN: usize = 16;
const SM4_IV_LEN: usize = 16;

/// Check whether the value is encrypted
///
/// 判断值是否为密文
pub fn is_encrypted(value: &str) -> bool {
    value.starts_with(SECRET_ENCRYPTED_PREFIX)
}

/// Get the encryption key and the encryption switch of the current module
///
/// 获取当前模块的加密密钥及加密开关
fn get_secret_config(funs: &TardisFunsInst) -> (String, bool) {
    RbumConfigManager::try_get_config(funs.module_code(), |conf| (conf.secret_attr_key.trim().to_string(), conf.secret_attr_encrypt)).unwrap_or_default()
}

/// Check whether the key is configured, the length must be valid
///
/// 判断是否已配置合法密钥
pub fn is_key_configured(funs: &TardisFunsInst) -> bool {
    let (key, _) = get_secret_config(funs);
    key.as_bytes().len() == SM4_KEY_LEN
}

/// Check whether the storage encryption is enabled, the key must be configured and the encryption switch must be turned on
///
/// 判断存储加密是否已生效，需要已配置密钥且已开启加密开关
pub fn is_encrypt_enabled(funs: &TardisFunsInst) -> bool {
    let (_, encrypt) = get_secret_config(funs);
    encrypt && is_key_configured(funs)
}

/// Encrypt the value with the specified key
///
/// 使用指定密钥加密值
pub fn encrypt_with_key(value: &str, key: &str) -> TardisResult<String> {
    if is_encrypted(value) {
        // Avoid encrypting the ciphertext again
        // 避免对密文二次加密
        return Ok(value.to_string());
    }
    if key.as_bytes().len() != SM4_KEY_LEN {
        return Err(TardisError::internal_error(
            &format!(
                "[BIOS.Rbum] The configuration [rbum.secret_attr_key] must be {SM4_KEY_LEN} bytes, but got {}",
                key.as_bytes().len()
            ),
            "500-rbum-secret-attr-key-invalid",
        ));
    }
    // The iv is random, and it is stored together with the ciphertext
    //
    // iv 随机生成，与密文一起存储
    let iv = TardisFuns::crypto.key.rand_16_hex();
    let ciphertext = TardisFuns::crypto.sm4.encrypt_cbc(value, key, &iv)?;
    Ok(format!("{SECRET_ENCRYPTED_PREFIX}{iv}{ciphertext})"))
}

/// Decrypt the value with the specified key
///
/// 使用指定密钥解密值
pub fn decrypt_with_key(value: &str, key: &str) -> TardisResult<String> {
    let Some(body) = value.strip_prefix(SECRET_ENCRYPTED_PREFIX).and_then(|body| body.strip_suffix(')')) else {
        // Plaintext
        // 明文
        return Ok(value.to_string());
    };
    if key.as_bytes().len() != SM4_KEY_LEN {
        return Err(TardisError::internal_error(
            "[BIOS.Rbum] Fail to decrypt the sensitive relationship attribute because the configuration [rbum.secret_attr_key] is invalid",
            "500-rbum-secret-attr-key-invalid",
        ));
    }
    if body.len() <= SM4_IV_LEN {
        return Err(TardisError::internal_error(
            "[BIOS.Rbum] Fail to decrypt the sensitive relationship attribute because the ciphertext is invalid",
            "500-rbum-secret-attr-ciphertext-invalid",
        ));
    }
    let (iv, ciphertext) = body.split_at(SM4_IV_LEN);
    TardisFuns::crypto.sm4.decrypt_cbc(ciphertext, key, iv)
}

/// Encrypt the value before storing
///
/// 存储前加密值
///
/// NOTE: The value is returned as is when the encryption is disabled or the key is not configured.
///
/// NOTE： 未开启加密或未配置密钥时原样返回。
pub fn encrypt(value: &str, funs: &TardisFunsInst) -> TardisResult<String> {
    let (key, encrypt) = get_secret_config(funs);
    if !encrypt {
        return Ok(value.to_string());
    }
    if key.is_empty() {
        warn!("[BIOS.Rbum] The sensitive relationship attribute is stored in plaintext because the configuration [rbum.secret_attr_key] is empty");
        return Ok(value.to_string());
    }
    encrypt_with_key(value, &key)
}

/// Decrypt the value after reading
///
/// 读取后解密值
///
/// NOTE: The value is returned as is when it is not encrypted.
///
/// NOTE： 值未加密时原样返回。
pub fn decrypt(value: &str, funs: &TardisFunsInst) -> TardisResult<String> {
    if !is_encrypted(value) {
        return Ok(value.to_string());
    }
    let (key, _) = get_secret_config(funs);
    decrypt_with_key(value, &key)
}

/// Decrypt the value, and return the original value with an error log when the decryption fails
///
/// 解密值，解密失败时返回原值并记录错误日志
///
/// NOTE: It is used for the reading paths to avoid the reading failure caused by the decryption failure.
///
/// NOTE： 用于读取路径，避免因解密失败导致读取不可用。
pub fn decrypt_or_original(value: &str, funs: &TardisFunsInst) -> String {
    if !is_encrypted(value) {
        return value.to_string();
    }
    match decrypt(value, funs) {
        Ok(plaintext) => plaintext,
        Err(err) => {
            error!("[BIOS.Rbum] Fail to decrypt the sensitive relationship attribute: {err}");
            value.to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const KEY: &str = "0123456789abcdef";
    const OTHER_KEY: &str = "fedcba9876543210";

    #[test]
    fn test_encrypt_and_decrypt() {
        let plaintext = "p@ssw0rd-中文-1234567890";
        let ciphertext = encrypt_with_key(plaintext, KEY).expect("encrypt failed");
        assert!(is_encrypted(&ciphertext));
        assert_ne!(ciphertext, plaintext);
        assert_eq!(decrypt_with_key(&ciphertext, KEY).expect("decrypt failed"), plaintext);
        // The ciphertext is not encrypted again
        //
        // 密文不会被二次加密
        assert_eq!(encrypt_with_key(&ciphertext, KEY).expect("encrypt failed"), ciphertext);
        // The plaintext is returned as is
        //
        // 明文原样返回
        assert_eq!(decrypt_with_key(plaintext, KEY).expect("decrypt failed"), plaintext);
    }

    #[test]
    fn test_decrypt_with_other_key() {
        let ciphertext = encrypt_with_key("p@ssw0rd", KEY).expect("encrypt failed");
        assert!(decrypt_with_key(&ciphertext, OTHER_KEY).map(|plaintext| plaintext != "p@ssw0rd").unwrap_or(true));
    }

    #[test]
    fn test_invalid_key() {
        assert!(encrypt_with_key("p@ssw0rd", "too-short").is_err());
        assert!(encrypt_with_key("p@ssw0rd", "0123456789abcdef").is_ok());
    }
}
