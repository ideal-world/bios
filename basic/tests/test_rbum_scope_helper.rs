use tardis::tokio;

use bios_basic::rbum::helper::rbum_scope_helper::get_match_paths;

#[tokio::test]
pub async fn test_get_match_paths() {
    assert_eq!(get_match_paths(0, ""), Some("%"));
    assert_eq!(get_match_paths(0, "aaaa"), Some("%"));

    assert_eq!(get_match_paths(1, ""), None);

    assert_eq!(get_match_paths(1, "aaaa"), Some("aaaa%"));
    assert_eq!(get_match_paths(1, "aaaa/bbbb"), Some("aaaa%"));
    assert_eq!(get_match_paths(1, "aaaa/bbbb/cccc"), Some("aaaa%"));

    assert_eq!(get_match_paths(2, "aaaa"), None);
    assert_eq!(get_match_paths(2, "aaaa/bbbb"), Some("aaaa/bbbb%"));
    assert_eq!(get_match_paths(2, "aaaa/bbbb/cccc"), Some("aaaa/bbbb%"));

    assert_eq!(get_match_paths(3, "aaaa"), None);
    assert_eq!(get_match_paths(3, "aaaa/bbbb"), Noneo);
    assert_eq!(get_match_paths(3, "aaaa/bbbb/cccc"), Some("aaaa/bbbb/cccc%"));
}
