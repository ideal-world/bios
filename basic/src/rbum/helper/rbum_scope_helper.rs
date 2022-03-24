use itertools::Itertools;

pub fn get_pre_paths(scope_level: i32, scope_paths: &str) -> String {
    let scope_paths = scope_paths.trim();
    let scope_paths = scope_paths.strip_suffix('/').unwrap_or(scope_paths).to_string();
    if scope_level == 0 || scope_paths.is_empty() {
        return "".to_string();
    }
    let split_items = scope_paths.split('/').collect::<Vec<&str>>();
    if split_items.len() < scope_level as usize {
        // unmatched characters
        return format!("{}//", scope_paths);
    } else if split_items.len() == scope_level as usize {
        return scope_paths;
    }
    split_items.iter().take(scope_level as usize).join("/")
}

pub fn get_path_item(scope_level: i32, scope_paths: &str) -> Option<String> {
    let scope_paths = scope_paths.trim();
    let scope_paths = scope_paths.strip_suffix('/').unwrap_or(scope_paths).to_string();
    if scope_level == 0 || scope_paths.is_empty() {
        return None;
    }
    let split_items = scope_paths.split('/').collect::<Vec<&str>>();
    if split_items.len() < scope_level as usize {
        return None;
    }
    split_items.get(scope_level as usize - 1).map(|s| s.to_string())
}
