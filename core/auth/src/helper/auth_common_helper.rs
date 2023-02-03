use itertools::Itertools;

pub fn sort_query(query: &str) -> String {
    if query.is_empty() {
        return "".to_string();
    }
    query.split('&').sorted_by(|a, b| Ord::cmp(&a.to_lowercase(), &b.to_lowercase())).join("&")
}
