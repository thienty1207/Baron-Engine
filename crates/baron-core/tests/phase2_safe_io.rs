//! Phase 2 Safe I/O and locking contract fixtures.
//!
//! The default cases are cross-platform. Symlink mechanics are guarded by the
//! platform they exercise; Windows junction coverage remains in the existing
//! adapter/core suites where the host can create one.

use std::fs;
use std::sync::{Arc, Barrier};
use std::time::Duration;

use baron_core::safe_io::{
    acquire_project_lock, acquire_project_lock_with_timeout, ensure_directory_chain,
    project_lock_path, read_bytes, read_text, replace_file,
};
use tempfile::tempdir;

#[test]
fn tri_state_read_fixture_is_explicit_and_preserve_first() {
    let temp = tempdir().unwrap();
    assert_eq!(read_bytes(temp.path().join("missing")).unwrap(), None);

    let path = temp.path().join("value");
    fs::write(&path, b"before").unwrap();
    assert_eq!(read_text(&path).unwrap().as_deref(), Some("before"));
    let before = fs::read(&path).unwrap();
    fs::write(&path, [0xff, 0xfe]).unwrap();
    assert!(read_text(&path).is_err());
    assert_eq!(fs::read(&path).unwrap(), [0xff, 0xfe]);
    fs::write(&path, before).unwrap();
    fs::create_dir(temp.path().join("directory")).unwrap();
    assert!(read_bytes(temp.path().join("directory")).is_err());
}

#[test]
fn atomic_replacement_is_noop_for_identical_bytes_and_uses_unique_staging() {
    let temp = tempdir().unwrap();
    ensure_directory_chain(temp.path().join("nested/dir")).unwrap();
    assert!(temp.path().join("nested/dir").is_dir());
    let path = temp.path().join("value.txt");
    fs::write(&path, b"same").unwrap();
    replace_file(&path, b"same").unwrap();
    assert_eq!(fs::read(&path).unwrap(), b"same");

    replace_file(&path, b"next").unwrap();
    assert_eq!(fs::read_to_string(&path).unwrap(), "next");
    let leftovers = fs::read_dir(temp.path())
        .unwrap()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_name().to_string_lossy().contains("baron-tmp-"))
        .count();
    assert_eq!(leftovers, 0);
}

#[test]
fn project_lock_serializes_writers_with_bounded_contention_and_reentrancy() {
    let temp = tempdir().unwrap();
    let owner = acquire_project_lock(temp.path()).unwrap();
    let nested = acquire_project_lock(temp.path()).unwrap();
    drop(nested);

    let barrier = Arc::new(Barrier::new(2));
    let ready = Arc::clone(&barrier);
    let repo = temp.path().to_path_buf();
    let contender = std::thread::spawn(move || {
        ready.wait();
        acquire_project_lock_with_timeout(&repo, Duration::from_millis(120))
            .unwrap_err()
            .to_string()
    });
    barrier.wait();
    let error = contender.join().unwrap();
    assert!(error.contains("Timed out waiting for Baron mutation lock"));

    drop(owner);
    acquire_project_lock_with_timeout(temp.path(), Duration::from_millis(120)).unwrap();
}

#[test]
fn stale_marker_is_recovered_only_after_the_os_lock_is_available() {
    let temp = tempdir().unwrap();
    let path = project_lock_path(temp.path()).unwrap();
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, "pid=999999 thread=dead acquired_unix_ms=0\n").unwrap();

    let lock = acquire_project_lock_with_timeout(temp.path(), Duration::from_millis(120)).unwrap();
    drop(lock);
    assert!(fs::read_to_string(path)
        .unwrap()
        .contains(&format!("pid={}", std::process::id())));
}

#[cfg(unix)]
#[test]
fn symlink_directory_escape_is_rejected_before_creation() {
    use std::os::unix::fs::symlink;

    let temp = tempdir().unwrap();
    let outside = temp.path().join("outside");
    let link = temp.path().join("link");
    fs::create_dir(&outside).unwrap();
    symlink(&outside, &link).unwrap();

    assert!(ensure_directory_chain(link.join("escape")).is_err());
    assert!(!outside.join("escape").exists());
}
