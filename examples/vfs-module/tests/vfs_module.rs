//! Prouve que le cycle de vie d'un module stocké est le même sur Vfs mémoire et disque.

use std::path::Path;

use vfs_module_example::{codes, lifecycle_in_memory, lifecycle_on_disk, Lifecycle};

fn assert_expected(lc: &Lifecycle) {
    assert_eq!(lc.after_first_put.as_deref(), Some("0.1.0"));
    assert_eq!(lc.after_second_put.as_deref(), Some("0.2.0"));
    assert_eq!(lc.explicit_old_version.as_deref(), Some("0.1.0"));
    assert!(lc
        .listed
        .iter()
        .any(|(n, v)| n == "billing" && v == "0.1.0"));
    assert!(lc
        .listed
        .iter()
        .any(|(n, v)| n == "billing" && v == "0.2.0"));
    assert_eq!(lc.after_remove_new.as_deref(), Some("0.1.0"));
    assert_eq!(lc.final_error.as_deref(), Some(codes::VFS_NOT_FOUND));
}

#[tokio::test]
async fn in_memory_serves_the_highest_version_then_reports_not_found_after_removal() {
    let lc = lifecycle_in_memory().await;
    assert_expected(&lc);
}

#[tokio::test]
async fn the_same_lifecycle_holds_on_disk() {
    let dir = std::env::temp_dir().join(format!("vfs-module-test-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("temp dir");
    let lc = lifecycle_on_disk(Path::new(&dir)).await;
    let _ = std::fs::remove_dir_all(&dir);
    assert_expected(&lc);
}
