use baron_core::work_shape::{decide_work_shape, DurabilityNeed, LifecycleDepth, WorkShape};

#[test]
fn work_shape_is_read_only_and_does_not_create_lifecycle_files() {
    let temp = tempfile::tempdir().unwrap();
    let decision = decide_work_shape(temp.path(), "inspect and report current status").unwrap();
    assert_eq!(decision.work_shape, WorkShape::ReadOnly);
    assert_eq!(decision.durability, DurabilityNeed::None);
    assert_eq!(decision.lifecycle, LifecycleDepth::ReadOnly);
    assert!(!temp.path().join("docs/baron").exists());
}

#[test]
fn multi_session_change_is_durable() {
    let temp = tempfile::tempdir().unwrap();
    let decision = decide_work_shape(temp.path(), "coordinate a multi-session migration").unwrap();
    assert_eq!(decision.work_shape, WorkShape::Durable);
    assert_eq!(decision.lifecycle, LifecycleDepth::Full);
}
