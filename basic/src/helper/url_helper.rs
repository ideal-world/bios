use itertools::Itertools;
use std::collections::HashMap;

pub fn sort_query(query: &str) -> String {
    if query.is_empty() {
        return "".to_string();
    }
    query.split('&').sorted_by(|a, b| Ord::cmp(&a.to_lowercase(), &b.to_lowercase())).join("&")
}

pub fn sort_hashmap_query(query: HashMap<String, String>) -> String {
    if query.is_empty() {
        return "".to_string();
    }
    query.iter().map(|a| format!("{}={}", a.0, a.1)).sorted_by(|a, b| Ord::cmp(&a.to_lowercase(), &b.to_lowercase())).join("&")
}