use tardis::basic::result::TardisResult;

pub fn encode_item(id: &str, owner_path: &str, key: &[u8; 32]) -> TardisResult<String> {
    use tardis::crypto::{
        crypto_aead::{algorithm::Aes256, TardisCryptoAead},
        crypto_base64::TardisCryptoBase64,
    };
    let id = id.as_bytes();
    let id_len = id.len();
    let owner_path = owner_path.as_bytes();
    let owner_path_len = owner_path.len();
    let mut message = Vec::with_capacity(id_len + owner_path_len + 2);
    message.push(id_len as u8);
    message.extend_from_slice(id);
    message.push(owner_path_len as u8);
    message.extend_from_slice(owner_path);
    let bin_enc = TardisCryptoAead.encrypt_ecb::<Aes256>(message, key)?;
    Ok(TardisCryptoBase64.encode_raw(bin_enc))
}

pub fn decode_item(base64_code: &str, key: &[u8; 32]) -> TardisResult<(String, String)> {
    use tardis::crypto::{
        crypto_aead::{algorithm::Aes256, TardisCryptoAead},
        crypto_base64::TardisCryptoBase64,
    };
    let message = TardisCryptoBase64.decode(base64_code)?;
    let raw_bin = TardisCryptoAead.decrypt_ecb::<Aes256>(message, key)?;
    let id_len = raw_bin[0] as usize;
    let id = String::from_utf8(raw_bin[1..=id_len].to_vec())?;
    let owner_path_len = raw_bin[id_len + 1] as usize;
    let owner_path = String::from_utf8(raw_bin[(id_len + 2)..=(id_len + 1 + owner_path_len)].to_vec())?;
    Ok((id, owner_path))
}