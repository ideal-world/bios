use super::error::TardisError;
use serde::{Deserialize, Serialize};

pub type TardisResult<T> = Result<T, TardisError>;

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct TardisResp<T>
where
    T: Serialize,
{
    pub code: String,
    pub msg: String,
    pub data: Option<T>,
}

impl<T: Serialize> TardisResp<T> {
    pub fn is_ok(&self) -> bool {
        self.code.starts_with('2')
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct TardisPage<T>
where
    T: Serialize,
{
    pub page_size: u64,
    pub page_number: u64,
    pub total_size: u64,
    pub records: Vec<T>,
}

pub fn remove_quotes(s: &str) -> &str {
    if s.starts_with('"') && s.ends_with('"') && s.len() >= 2 {
        &s[1..s.len() - 1]
    } else {
        s
    }
}
