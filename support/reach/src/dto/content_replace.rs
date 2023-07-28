use std::collections::HashMap;

use std::fmt::Display;
use std::ops::{Deref, DerefMut};
use std::str::FromStr;

use tardis::basic::error::TardisError;

use tardis::regex::Regex;

/// ContentReplace
/// 格式
/// `{[<key>:<value>],*,?}`
#[repr(transparent)]
pub struct ContentReplace(HashMap<String, String>);

impl<K, V, I> From<I> for ContentReplace
where
    I: IntoIterator<Item = (K, V)>,
    K: ToString,
    V: ToString,
{
    fn from(value: I) -> Self {
        let map = value.into_iter().map(|(k, v)| (k.to_string(), v.to_string())).collect::<HashMap<String, String>>();
        Self(map)
    }
}

impl Deref for ContentReplace {
    type Target = HashMap<String, String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ContentReplace {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl FromStr for ContentReplace {
    type Err = TardisError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        tardis::serde_json::from_str(s)
            .map_err(|e| TardisError::bad_request(&format!("content replace is not a valid json map: {e}"), "400-invalid-content-replace"))
            .map(ContentReplace)
    }
}

impl Display for ContentReplace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut content = String::new();
        for (key, value) in self.iter() {
            content.push_str(&format!("{}:{},", key, value));
        }
        write!(f, "{{{}}}", content.trim_end_matches(','))
    }
}

impl ContentReplace {
    pub fn new(map: HashMap<String, String>) -> Self {
        Self(map)
    }
    pub fn render_final_content<const MAXLEN: usize>(&self, template: &str) -> String {
        content_replace::<MAXLEN>(template, &self.0)
    }
}

fn content_replace<const MAXLEN: usize>(content: &str, values: &HashMap<String, String>) -> String {
    lazy_static::lazy_static! {
        static ref EXTRACT_R: Regex = Regex::new(r"(\[^}]+?})").expect("reach content replace extract regex is invalid");
    }
    let mut new_content = content.to_string();
    let matcher = EXTRACT_R.find_iter(content);
    for mat in matcher {
        let key = &content[mat.start() + 1..mat.end() - 1];
        if let Some(value) = values.get(key) {
            let replace_value = if value.len() > MAXLEN {
                format!("{}...", &value[(MAXLEN - 3)..])
            } else {
                value.to_string()
            };
            new_content = new_content.replacen(mat.as_str(), &replace_value, 1);
        }
    }
    new_content
}
