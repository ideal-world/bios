use std::collections::HashMap;

use std::fmt::Display;
use std::ops::{Deref, DerefMut};
use std::str::FromStr;

use tardis::basic::error::TardisError;

use tardis::regex::Regex;
use tardis::serde::{Deserialize, Serialize};
/// ContentReplace
/// 格式
/// `{[<key>:<value>],*,?}`
#[repr(transparent)]
#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(transparent)]
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
        tardis::serde_json::to_string(self).expect("content replace is not a valid json map").fmt(f)
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
        static ref EXTRACT_R: Regex = Regex::new(r"(\{[^}]+?})").expect("reach content replace extract regex is invalid");
    }
    let mut new_content = String::new();
    let matcher = EXTRACT_R.find_iter(content);
    let mut idx = 0;
    for mat in matcher {
        new_content.push_str(&content[idx..mat.start()]);
        let key = &content[(mat.start() + 1)..(mat.end() - 1)];
        if let Some(value) = values.get(key) {
            if value.chars().count() > MAXLEN {
                new_content.extend(value.chars().take(MAXLEN - 3));
                new_content.push_str("...");
            } else {
                new_content.push_str(value)
            };
        }
        idx = mat.end();
    }
    new_content.push_str(&content[idx..]);
    new_content
}

#[test]
fn test_content_replace() {
    let content = "hello {name}, your code is {code}";
    let replaced = content_replace::<10>(content, &[("name".to_string(), "Alice".to_string()), ("code".to_string(), "123456".to_string())].into());
    assert_eq!(replaced, "hello Alice, your code is 123456");
    let replaced = content_replace::<10>(
        content,
        &[("name".to_string(), "Alice".to_string()), ("code".to_string(), "123456789abcdef".to_string())].into(),
    );
    assert_eq!(replaced, "hello Alice, your code is 1234567...");
}
