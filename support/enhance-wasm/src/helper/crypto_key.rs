use rand::RngCore;

use crate::basic::basic::TardisResult;

pub fn rand_16_hex() -> TardisResult<String> {
    let mut key: [u8; 8] = [0; 8];
    rand::rngs::OsRng::default().fill_bytes(&mut key);
    Ok(hex::encode(key))
}
