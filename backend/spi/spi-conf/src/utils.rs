use crate::conf_constants::*;

/// gen random string by given charset (those are supposed to be ascii char)
pub(crate) fn random_string(len: usize, charset: &[u8]) -> String {
    use tardis::rand::Rng;
    let mut buf = String::with_capacity(len);
    let mut rng = tardis::rand::thread_rng();
    let size = charset.len();
    for _ in 0..len {
        let idx = rng.gen_range::<usize, _>(0..size);
        buf.push(charset[idx] as char);
    }
    buf
}

pub(crate) fn random_ak() -> String {
    crate::utils::random_string(8, CHARSET_SK)
}
pub(crate) fn random_sk() -> String {
    crate::utils::random_string(12, CHARSET_AK)
}

pub(crate) fn parse_tags(tags: &str) -> Vec<String> {
    let mut v = tags
        .split(',')
        .filter_map(|t| {
            let t = t.trim();
            if t.is_empty() {
                None
            } else {
                Some(t.to_string())
            }
        })
        .collect::<Vec<_>>();
    v.dedup();
    v
}
