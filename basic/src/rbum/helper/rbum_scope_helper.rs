//!
//!
//! Scope matching rule, assuming item length is 4 :
//!
//! | own_paths        | scope_level | visit level 0  | visit level 1  | visit level 2  | visit level 3   |
//! | ---------------  | ----------- | -------------  | -------------  | -------------  | --------------   |
//! | ''               | -1          | ''             |                |                |                 |
//! | ''               | 0           | %              | %              | %              | %               |
//! | ''               | 1           | ''             | %              | %              | %               |
//! | ''               | 2           | ''             |                | %              | %               |
//! | ''               | 3           | ''             |                |                | %               |
//! | 'AAAA'           | -1          |                | AAAA           |                |                 |
//! | 'AAAA'           | 0           | %              | %              | %              | %               |
//! | 'AAAA'           | 1           |                | AAAA%          | AAAA%          | AAAA%           |
//! | 'AAAA'           | 2           |                |                | AAAA%          | AAAA%           |
//! | 'AAAA'           | 3           |                |                |                | AAAA%           |
//! | 'AAAA/BBBB'      | -1          |                |                | AAAA/BBBB      |                 |
//! | 'AAAA/BBBB'      | 0           | %              | %              | %              | %               |
//! | 'AAAA/BBBB'      | 1           |                | AAAA%          | AAAA%          | AAAA%           |
//! | 'AAAA/BBBB'      | 2           |                |                | AAAA/BBBB%     | AAAA/BBBB%      |
//! | 'AAAA/BBBB'      | 3           |                |                |                | AAAA/BBBB%      |
//! | 'AAAA/BBBB/CCCC' | -1          |                |                |                | AAAA/BBBB/CCCC  |
//! | 'AAAA/BBBB/CCCC' | 0           | %              | %              | %              | %               |
//! | 'AAAA/BBBB/CCCC' | 1           |                | AAAA%          | AAAA%          | AAAA%           |
//! | 'AAAA/BBBB/CCCC' | 2           |                |                | AAAA/BBBB%     | AAAA/BBBB%      |
//! | 'AAAA/BBBB/CCCC' | 3           |                |                |                | AAAA/BBBB/CCCC% |
//!
use std::cmp::Ordering;

use itertools::Itertools;
use tardis::basic::dto::TardisContext;
use tardis::basic::error::TardisError;
use tardis::basic::result::TardisResult;

use crate::rbum::rbum_enumeration::RbumScopeLevelKind;

pub fn get_pre_paths(scope_level: i8, own_paths: &str) -> Option<String> {
    let own_paths = own_paths.trim();
    let own_paths = own_paths.strip_suffix('/').unwrap_or(own_paths).to_string();
    if scope_level == 0 {
        return Some("%".to_string());
    }
    let split_items = if own_paths.is_empty() { vec![] } else { own_paths.split('/').collect::<Vec<_>>() };
    match split_items.len().cmp(&(scope_level as usize)) {
        Ordering::Less => {
            // unmatched characters
            None
        }
        _ => Some(format!("{}%", split_items.iter().take(scope_level as usize).join("/"))),
    }
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

pub fn degrade_own_paths(mut cxt: TardisContext, new_own_paths: &str) -> TardisResult<TardisContext> {
    if !new_own_paths.contains(&cxt.own_paths) {
        return Err(TardisError::Conflict("Not qualified for downgrade".to_string()));
    }
    cxt.own_paths = new_own_paths.to_string();
    Ok(cxt)
}
