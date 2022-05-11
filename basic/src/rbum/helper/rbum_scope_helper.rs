use itertools::Itertools;
use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;

use crate::rbum::rbum_enumeration::RbumScopeLevelKind;

pub fn get_pre_paths(scope_level: i8, own_paths: &str) -> String {
    let own_paths = own_paths.trim();
    let own_paths = own_paths.strip_suffix('/').unwrap_or(own_paths).to_string();
    if scope_level == 0 && own_paths.is_empty() {
        return "".to_string();
    } else if own_paths.is_empty() {
        return "//".to_string();
    }
    let split_items = own_paths.split('/').collect::<Vec<&str>>();
    if split_items.len() < scope_level as usize {
        // unmatched characters
        return format!("{}//", own_paths);
    } else if split_items.len() == scope_level as usize {
        return own_paths;
    }
    split_items.iter().take(scope_level as usize).join("/")
}

pub fn get_path_item(scope_level: i8, own_paths: &str) -> Option<String> {
    let own_paths = own_paths.trim();
    let own_paths = own_paths.strip_suffix('/').unwrap_or(own_paths).to_string();
    if scope_level == 0 || own_paths.is_empty() {
        return None;
    }
    let split_items = own_paths.split('/').collect::<Vec<&str>>();
    if split_items.len() < scope_level as usize {
        return None;
    }
    split_items.get(scope_level as usize - 1).map(|s| s.to_string())
}

pub fn get_scope_level_by_context(cxt: &TardisContext) -> TardisResult<RbumScopeLevelKind> {
    let own_paths = cxt.own_paths.trim();
    let own_paths = own_paths.strip_suffix('/').unwrap_or(own_paths).to_string();
    RbumScopeLevelKind::from_int(own_paths.matches('/').count() as i8)
}

pub fn get_max_level_id_by_context(cxt: &TardisContext) -> Option<String> {
    let own_paths = cxt.own_paths.trim();
    if own_paths.is_empty() {
        return None;
    }
    let own_paths = own_paths.strip_suffix('/').unwrap_or(own_paths).to_string();
    own_paths.split('/').collect::<Vec<&str>>().last().map(|s| s.to_string())
}
