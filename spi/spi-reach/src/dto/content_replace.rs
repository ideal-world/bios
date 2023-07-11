use std::collections::HashMap;

use std::fmt::Display;
use std::ops::{Deref, DerefMut};
use std::str::FromStr;

use tardis::basic::error::TardisError;
use tardis::basic::result::TardisResult;
use tardis::regex::Regex;

/// ContentReplace
/// 格式
/// `{[<key>:<value>],*,?}`
#[repr(transparent)]
pub struct ContentReplace<const MAX_RENDER_LEN: usize = 20>(HashMap<String, String>);

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
        fn not_trim_empty(s: &&str) -> bool {
            !s.trim().is_empty()
        }
        s.trim_start_matches('{')
            .trim_end_matches('}')
            // remove end trailing comma
            .trim_end_matches(',')
            .split(',')
            .map(|kv| {
                let mut kv = kv.split(':');
                let Some(key) = kv.next().filter(not_trim_empty) else {
                    return Err(TardisError::bad_request("key is empty", "400-invalid-content-replace"));
                };
                let Some(value) = kv.next().filter(not_trim_empty) else {
                    return Err(TardisError::bad_request("value is empty", "400-invalid-content-replace"));
                };
                Ok((key.to_string(), value.to_string()))
            })
            .collect::<Result<_, _>>()
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

impl<const MAX_RENDER_LEN: usize> ContentReplace<MAX_RENDER_LEN> {
    pub fn render_final_content(&self, template: &str) -> String {
        const DEFUALT_MAXLEN: usize = 20;
        content_replace::<MAX_RENDER_LEN>(template, &self.0)
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
