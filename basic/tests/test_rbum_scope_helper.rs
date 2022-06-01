use tardis::tokio;

use bios_basic::rbum::helper::rbum_scope_helper::get_pre_paths;

#[tokio::test]
pub async fn test_get_pre_paths() {
    assert_eq!(get_pre_paths(0, ""), Some("".to_string()));
    assert_eq!(get_pre_paths(0, "aaaa"), Some("".to_string()));

    assert_eq!(get_pre_paths(1, ""), None);

    assert_eq!(get_pre_paths(1, "aaaa"), Some("aaaa".to_string()));
    assert_eq!(get_pre_paths(1, "aaaa/bbbb"), Some("aaaa".to_string()));
    assert_eq!(get_pre_paths(1, "aaaa/bbbb/cccc"), Some("aaaa".to_string()));

    assert_eq!(get_pre_paths(2, "aaaa"), None);
    assert_eq!(get_pre_paths(2, "aaaa/bbbb"), Some("aaaa/bbbb".to_string()));
    assert_eq!(get_pre_paths(2, "aaaa/bbbb/cccc"), Some("aaaa/bbbb".to_string()));

    assert_eq!(get_pre_paths(3, "aaaa"), None);
    assert_eq!(get_pre_paths(3, "aaaa/bbbb"), None);
    assert_eq!(get_pre_paths(3, "aaaa/bbbb/cccc"), Some("aaaa/bbbb/cccc".to_string()));
}
