//! Scope helper
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
use crate::rbum::rbum_config::RbumConfigApi;
use itertools::Itertools;
use tardis::basic::dto::TardisContext;
use tardis::basic::error::TardisError;
use tardis::basic::result::TardisResult;
use tardis::{TardisFuns, TardisFunsInst};

use crate::rbum::rbum_enumeration::RbumScopeLevelKind;

/// Get the previous path at the specified level from the own paths.
///
/// # Examples
///
/// ```
/// use bios_basic::rbum::helper::rbum_scope_helper::get_pre_paths;
/// assert_eq!(get_pre_paths(0, "a/b/c"), Some("".to_string()));
/// assert_eq!(get_pre_paths(0, "a/b/c/"), Some("".to_string()));
/// assert_eq!(get_pre_paths(1, "a/b/c"), Some("a".to_string()));
/// assert_eq!(get_pre_paths(2, "a/b/c"), Some("a/b".to_string()));
/// assert_eq!(get_pre_paths(3, "a/b/c"), Some("a/b/c".to_string()));
/// assert_eq!(get_pre_paths(4, "a/b/c"), None);
/// ```
pub fn get_pre_paths(scope_level: i16, own_paths: &str) -> Option<String> {
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
        _ => Some(split_items.iter().take(scope_level as usize).join("/")),
    }
}

/// Get the path entries at the specified level from the own paths.
///
/// # Examples
///
/// ```
/// use bios_basic::rbum::helper::rbum_scope_helper::get_path_item;
/// assert_eq!(get_path_item(0, "a/b/c"), None);
/// assert_eq!(get_path_item(0, "a/b/c/"), None);
/// assert_eq!(get_path_item(1, "a/b/c"), Some("a".to_string()));
/// assert_eq!(get_path_item(2, "a/b/c"), Some("b".to_string()));
/// assert_eq!(get_path_item(3, "a/b/c"), Some("c".to_string()));
/// assert_eq!(get_path_item(4, "a/b/c"), None);
/// ```
pub fn get_path_item(scope_level: i16, own_paths: &str) -> Option<String> {
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

/// Get the scope level from the own paths in context.
pub fn get_scope_level_by_context(ctx: &TardisContext) -> TardisResult<RbumScopeLevelKind> {
    let own_paths = ctx.own_paths.trim();
    let own_paths = own_paths.strip_suffix('/').unwrap_or(own_paths).to_string();
    if own_paths == *"" {
        return Ok(RbumScopeLevelKind::Root);
    }
    RbumScopeLevelKind::from_int(own_paths.split('/').count() as i16)
}

/// Get the max level id from the own paths in the context.
///
/// # Examples
/// ```
/// use bios_basic::rbum::helper::rbum_scope_helper::get_max_level_id_by_context;
/// use tardis::basic::dto::TardisContext;
/// let mut ctx = TardisContext::default();
/// ctx.own_paths = "".to_string();
/// assert_eq!(get_max_level_id_by_context(&ctx), None);
/// ctx.own_paths = "a".to_string();
/// assert_eq!(get_max_level_id_by_context(&ctx), Some("a".to_string()));
/// ctx.own_paths = "a/b/c".to_string();
/// assert_eq!(get_max_level_id_by_context(&ctx), Some("c".to_string()));
/// ctx.own_paths = "a/b/c/".to_string();
/// assert_eq!(get_max_level_id_by_context(&ctx), Some("c".to_string()));
/// ```
pub fn get_max_level_id_by_context(ctx: &TardisContext) -> Option<String> {
    let own_paths = ctx.own_paths.trim();
    if own_paths.is_empty() {
        return None;
    }
    let own_paths = own_paths.strip_suffix('/').unwrap_or(own_paths).to_string();
    own_paths.split('/').collect::<Vec<&str>>().last().map(|s| s.to_string())
}

/// Downgrade of own paths.
///
/// The new own paths must be a subpath of the own paths in the context.
///
/// # Examples
/// ```
/// use bios_basic::rbum::helper::rbum_scope_helper::degrade_own_paths;
/// use tardis::basic::dto::TardisContext;
/// let mut ctx = TardisContext::default();
/// ctx.own_paths = "a/b".to_string();
/// assert_eq!(degrade_own_paths(ctx.clone(), "a/b/c").unwrap().own_paths, "a/b/c");
/// ctx.own_paths = "".to_string();
/// assert_eq!(degrade_own_paths(ctx.clone(), "a/b").unwrap().own_paths, "a/b");
/// ctx.own_paths = "a".to_string();
/// assert!(degrade_own_paths(ctx.clone(), "b").is_err());
/// ctx.own_paths = "a/b".to_string();
/// assert!(degrade_own_paths(ctx.clone(), "a/c").is_err());
/// ```
pub fn degrade_own_paths(mut ctx: TardisContext, new_own_paths: &str) -> TardisResult<TardisContext> {
    if !new_own_paths.starts_with(&ctx.own_paths) {
        return Err(TardisError::conflict("not qualified for downgrade", "409-rbum-*-downgrade-error"));
    }
    ctx.own_paths = new_own_paths.to_string();
    Ok(ctx)
}

/// Check scope Legality.
///
/// Legality Rules:
/// 1. Determine the ``standard_own_paths`` : When ``filter.own_paths`` is empty, use ``ctx_own_paths`` as the standard own paths, otherwise use ``filter.own_paths`` as the standard own paths.
/// 1. If ``record_own_paths`` is equal to the ``standard_own_paths`` or if ``filter.with_sub_own_paths`` is true and ``record_own_paths`` is a sub-path of the ``standard_own_paths``, then return true.
/// 1. If ``filter.ignore_scope`` is true, it means only ``own_paths`` comparison is required, so directly return false.
/// 1.  If ``record_scope_level`` exists, then get the prefix path of the ``standard_own_paths`` based on the value of ``record_scope_level``, and this prefix path must be the same as or a sub-path of ``record_own_paths``.
///
/// # Examples
/// ```
/// use bios_basic::rbum::helper::rbum_scope_helper::check_scope;
/// use bios_basic::rbum::dto::rbum_filer_dto::RbumBasicFilterReq;
/// let mut filter = RbumBasicFilterReq::default();
/// filter.ignore_scope = false;
/// filter.with_sub_own_paths = true;
/// filter.own_paths = None;
/// assert!(check_scope("a/b", None, &filter, "a/b"));  // standard_own_paths == record_own_paths
/// assert!(check_scope("a/b/c", None, &filter, "a/b"));  // record_own_paths.starts_with(standard_own_paths) == true
/// assert!(!check_scope("a/b/c", None, &filter, "a/b/z"));  // record_own_paths.starts_with(standard_own_paths) == false
/// assert!(!check_scope("a", None, &filter, "a/b"));  // record_own_paths.starts_with(standard_own_paths) == false
/// // standard_sub_paths.starts_with(record_sub_paths)
/// assert!(check_scope("a/b", Some(0), &filter, "a/b/c"));  // "".starts_with("") == true
/// assert!(check_scope("c", Some(0), &filter, "a/b/c"));  // "".starts_with("") == true
/// assert!(check_scope("a/b", Some(1), &filter, "a/b/c"));  // "a".starts_with("a") == true
/// assert!(check_scope("", Some(1), &filter, "a/b/c"));  // "a".starts_with("") == true
/// assert!(!check_scope("x/b", Some(1), &filter, "a/b/c"));  // "a".starts_with("") == false
/// assert!(check_scope("a/b", Some(2), &filter, "a/b/c"));  // "a/b".starts_with("a/b") == true
/// assert!(check_scope("", Some(2), &filter, "a/b/c"));  // "a/b".starts_with("") == true
/// assert!(check_scope("a", Some(2), &filter, "a/b/c"));  // "a/b".starts_with("a") == true
/// assert!(!check_scope("a/x", Some(2), &filter, "a/b/c"));  // "a/b".starts_with("a/x") == false
/// assert!(check_scope("a/b/c", Some(3), &filter, "a/b/c"));  // "a/b/c".starts_with("a/b/c") == true
/// assert!(check_scope("", Some(3), &filter, "a/b/c"));  // "a/b/c".starts_with("") == true
/// assert!(check_scope("a", Some(3), &filter, "a/b/c"));  // "a/b/c".starts_with("a") == true
/// assert!(check_scope("a/b", Some(3), &filter, "a/b/c"));  // "a/b/c".starts_with("a/b") == true
/// assert!(!check_scope("a/b/x", Some(3), &filter, "a/b/c"));  // "a/b/c".starts_with("a/b/x") == false
/// ```
pub fn check_scope(record_own_paths: &str, record_scope_level: Option<i16>, filter: &RbumBasicFilterReq, ctx_own_paths: &str) -> bool {
    let standard_own_paths = if let Some(own_paths) = &filter.own_paths { own_paths.as_str() } else { ctx_own_paths };
    if record_own_paths == standard_own_paths || filter.with_sub_own_paths && record_own_paths.starts_with(standard_own_paths) {
        return true;
    }
    if filter.ignore_scope {
        return false;
    }
    if let Some(record_scope_level) = record_scope_level {
        if let Some(standard_sub_paths) = get_pre_paths(record_scope_level, standard_own_paths) {
            let record_sub_paths = if record_own_paths.len() <= standard_sub_paths.len() {
                record_own_paths
            } else {
                &record_own_paths[0..standard_sub_paths.len()]
            };
            return standard_sub_paths.starts_with(record_sub_paths);
        }
    }
    false
}

/// Fill the context.
///
/// This method will fetch the context from the request header (default: 'Bios-Ctx') and fill the current context.
///
/// Warning: This operation is unsafe, and it should only be used in scenarios where there is no security risk.
#[cfg(feature = "default")]
fn do_unsafe_fill_ctx<F>(request: &tardis::web::poem::Request, f: F, funs: &TardisFunsInst, ctx: &mut TardisContext) -> TardisResult<()>
where
    F: FnOnce(TardisContext, &mut TardisContext),
{
    let bios_ctx = if let Some(bios_ctx) = request.header(&funs.rbum_head_key_bios_ctx()).or_else(|| request.header(&funs.rbum_head_key_bios_ctx().to_lowercase())) {
        TardisFuns::json.str_to_obj::<TardisContext>(&TardisFuns::crypto.base64.decode_to_string(bios_ctx)?)?
    } else if ctx.owner.is_empty() && ctx.ak.is_empty() && ctx.own_paths.is_empty() && ctx.roles.is_empty() && ctx.groups.is_empty() {
        return Err(TardisError::unauthorized(
            &format!("[Basic] Request is not legal, missing header [{}]", funs.rbum_head_key_bios_ctx()),
            "404-rbum-req-ctx-not-exist",
        ));
    } else {
        return Ok(());
    };

    if bios_ctx.own_paths.starts_with(&ctx.own_paths) {
        f(bios_ctx, ctx);
        Ok(())
    } else {
        Err(TardisError::forbidden(
            &format!("[Basic] Request is not legal from head [{}]", funs.rbum_head_key_bios_ctx()),
            "401-rbum-req-ctx-permission-denied",
        ))
    }
}

/// Check ``owner`` field of the context and fill the context.
///
/// When using ``ak/sk`` authentication from an internal calling interface (mostly ``ci`` type interfaces),
/// there is no ``owner`` field,
/// so this method can be used to determine whether it comes from an internal calling interface.
///
/// This method will fetch the context from the request header (default: 'Bios-Ctx') and fill the current context.
///
/// Warning: This operation is unsafe, and it should only be used in scenarios where there is no security risk.
#[cfg(feature = "default")]
pub fn check_without_owner_and_unsafe_fill_ctx(request: &tardis::web::poem::Request, funs: &TardisFunsInst, ctx: &mut TardisContext) -> TardisResult<()> {
    if !ctx.owner.is_empty() {
        return Err(TardisError::forbidden("[Basic] Request context owner is not empty", "403-rbum-req-ctx-owner-is-not-empty"));
    }
    unsafe_fill_ctx(request, funs, ctx)
}

/// Fill the context.
///
/// This method will fetch the context from the request header (default: 'Bios-Ctx') and fill the current context.
///
/// Warning: This operation is unsafe, and it should only be used in scenarios where there is no security risk.
#[cfg(feature = "default")]
pub fn unsafe_fill_ctx(request: &tardis::web::poem::Request, funs: &TardisFunsInst, ctx: &mut TardisContext) -> TardisResult<()> {
    do_unsafe_fill_ctx(
        request,
        |bios_ctx, ctx| {
            let mut roles = bios_ctx.roles.clone();
            for role in bios_ctx.roles.clone() {
                if role.contains(':') {
                    let extend_role = role.split(':').collect::<Vec<_>>()[0];
                    roles.push(extend_role.to_string());
                }
            }
            ctx.owner = bios_ctx.owner.clone();
            ctx.roles = roles;
            ctx.groups = bios_ctx.groups;
            ctx.own_paths = bios_ctx.own_paths;
        },
        funs,
        ctx,
    )
}

/// Fill the ``owner`` field of the context.
///
/// This method will fetch the context from the request header (default: 'Bios-Ctx') and fill the current context.
///
/// Warning: This operation is unsafe, and it should only be used in scenarios where there is no security risk.
#[cfg(feature = "default")]
pub fn unsafe_fill_owner_only(request: &tardis::web::poem::Request, funs: &TardisFunsInst, ctx: &mut TardisContext) -> TardisResult<()> {
    do_unsafe_fill_ctx(
        request,
        |bios_ctx, ctx| {
            ctx.owner = bios_ctx.owner.clone();
        },
        funs,
        ctx,
    )
}

/// Fill the ``own_paths`` field of the context.
///
/// This method will fetch the context from the request header (default: 'Bios-Ctx') and fill the current context.
///
/// Warning: This operation is unsafe, and it should only be used in scenarios where there is no security risk.
#[cfg(feature = "default")]
pub fn unsafe_fill_own_paths_only(request: &tardis::web::poem::Request, funs: &TardisFunsInst, ctx: &mut TardisContext) -> TardisResult<()> {
    do_unsafe_fill_ctx(
        request,
        |bios_ctx, ctx| {
            ctx.own_paths = bios_ctx.own_paths;
        },
        funs,
        ctx,
    )
}

/// Fill the ``roles`` field of the context.
///
/// This method will fetch the context from the request header (default: 'Bios-Ctx') and fill the current context.
///
/// Warning: This operation is unsafe, and it should only be used in scenarios where there is no security risk.
#[cfg(feature = "default")]
pub fn unsafe_fill_roles_only(request: &tardis::web::poem::Request, funs: &TardisFunsInst, ctx: &mut TardisContext) -> TardisResult<()> {
    do_unsafe_fill_ctx(
        request,
        |bios_ctx, ctx| {
            let mut roles = bios_ctx.roles.clone();
            for role in bios_ctx.roles.clone() {
                if role.contains(':') {
                    let extend_role = role.split(':').collect::<Vec<_>>()[0];
                    roles.push(extend_role.to_string());
                }
            }
            ctx.roles = roles;
        },
        funs,
        ctx,
    )
}

/// Fill the ``group`` field of the context.
///
/// This method will fetch the context from the request header (default: 'Bios-Ctx') and fill the current context.
///
/// Warning: This operation is unsafe, and it should only be used in scenarios where there is no security risk.
#[cfg(feature = "default")]
pub fn unsafe_fill_groups_only(request: &tardis::web::poem::Request, funs: &TardisFunsInst, ctx: &mut TardisContext) -> TardisResult<()> {
    do_unsafe_fill_ctx(
        request,
        |bios_ctx, ctx| {
            ctx.groups = bios_ctx.groups;
        },
        funs,
        ctx,
    )
}
