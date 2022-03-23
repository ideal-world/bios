pub const RBUM_KIND_ID_LEN: usize = 6;
pub const RBUM_DOMAIN_ID_LEN: usize = 6;

pub const RBUM_REL_CATE_SYS_CODE_NODE_LEN: usize = 4;

pub const RBUM_SCOPE_L1_LEN: usize = 6;
pub const RBUM_SCOPE_L2_LEN: usize = 6;
pub const RBUM_SCOPE_L3_LEN: usize = 6;

pub fn get_pre_levels(scope_level: i32, scope_ids: &str) -> String {
    let len = match scope_level {
        0 => 0,
        1 => RBUM_SCOPE_L1_LEN,
        2 => RBUM_SCOPE_L1_LEN + RBUM_SCOPE_L2_LEN,
        _ => RBUM_SCOPE_L1_LEN + RBUM_SCOPE_L2_LEN + RBUM_SCOPE_L3_LEN,
    };
    if scope_ids.len() >= len {
        scope_ids[..len].to_string()
    } else {
        scope_ids.to_string()
    }
}
