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

use crate::rbum::dto::rbum_filer_dto::RbumBasicFilterReq;
use itertools::Itertools;
use tardis::basic::dto::TardisContext;
use tardis::basic::error::TardisError;
use tardis::basic::result::TardisResult;

use crate::rbum::rbum_enumeration::RbumScopeLevelKind;

pub fn get_pre_paths(scope_level: i8, own_paths: &str) -> Option<String> {
    let own_paths = own_paths.trim();
    let own_paths = own_paths.strip_suffix('/').unwrap_or(own_paths).to_string();
    if scope_level == 0 {
        return Some("".to_string());
    }
    let split_items = if own_paths.is_empty() { vec![] } else { own_paths.split('/').collect::<Vec<_>>() };
    match split_items.len().cmp(&(scope_level as usize)) {
        Ordering::Less => {
            // unmatched characters
            None
        }
        _ => Some(format!("{}", split_items.iter().take(scope_level as usize).join("/"))),
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

pub fn check_scope(record_own_paths: &str, record_scope_level: Option<i8>, filter: &RbumBasicFilterReq, cxt: &TardisContext) -> bool {
    let filter_own_paths = if let Some(own_paths) = &filter.own_paths { own_paths.as_str() } else { &cxt.own_paths };
    if record_own_paths == filter_own_paths || filter.with_sub_own_paths && record_own_paths.contains(filter_own_paths) {
        return true;
    }
    if filter.ignore_scope {
        return false;
    }
    if let Some(record_scope_level) = record_scope_level {
        if let Some(p1) = get_pre_paths(1, filter_own_paths) {
            if record_scope_level == 1 {
                return record_own_paths.is_empty() || record_own_paths.contains(&p1);
            }
            if let Some(p2) = get_pre_paths(2, filter_own_paths) {
                let node_len = p2.len() - p1.len() - 1;
                if record_scope_level == 2 {
                    return record_own_paths.is_empty() || record_own_paths.contains(&p2) || (record_own_paths.len() == node_len && record_own_paths.contains(&p1));
                }
                if let Some(p3) = get_pre_paths(3, filter_own_paths) {
                    if record_scope_level == 3 {
                        return record_own_paths.is_empty()
                            || record_own_paths.contains(&p3)
                            || (record_own_paths.len() == node_len && record_own_paths.contains(&p1))
                            || (record_own_paths.len() == node_len * 2 + 1 && record_own_paths.contains(&p2));
                    }
                }
            }
        }
    }
    false
}
