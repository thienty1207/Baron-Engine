use baron_core::operation::{
    operation_id_for_parts, task_id_for_task, LifecycleIdentity, SupportedAdapter,
    MAX_IDENTIFIER_CHARS,
};

#[test]
fn task_identity_ignores_session_request_and_line_ending_representation() {
    let first = task_id_for_task("project-1", "  Review API\r\ncontract  ").unwrap();
    let second = task_id_for_task("project-1", "Review API\ncontract").unwrap();
    assert_eq!(first, second);
}

#[test]
fn operation_identity_is_stable_only_for_the_same_complete_tuple() {
    let task = task_id_for_task("project-1", "Review API").unwrap();
    let operation_id = operation_id_for_parts(
        "project-1",
        &task,
        SupportedAdapter::Codex,
        "session-a",
        "request-a",
    );
    let first = LifecycleIdentity::new(
        "project-1",
        task.clone(),
        operation_id,
        SupportedAdapter::Codex,
        "session-a",
        "request-a",
    )
    .unwrap();
    let same = operation_id_for_parts(
        first.project_id(),
        first.task_id(),
        first.adapter(),
        first.session_id(),
        first.request_id(),
    );
    assert_eq!(same, first.operation_id());
    assert_ne!(
        operation_id_for_parts(
            "project-1",
            &task,
            SupportedAdapter::Codex,
            "session-a",
            "request-b",
        ),
        first.operation_id()
    );
    assert_ne!(
        operation_id_for_parts(
            "project-1",
            &task,
            SupportedAdapter::Claude,
            "session-a",
            "request-a",
        ),
        first.operation_id()
    );
}

#[test]
fn identity_validation_rejects_unsafe_and_overlong_authority_fields() {
    assert!(LifecycleIdentity::new(
        "project-1",
        "task-1",
        "operation-1",
        SupportedAdapter::Codex,
        "session\nunsafe",
        "request-1",
    )
    .is_err());
    assert!(LifecycleIdentity::new(
        "project-1",
        "task-1",
        "operation-1",
        SupportedAdapter::Codex,
        &"x".repeat(MAX_IDENTIFIER_CHARS + 1),
        "request-1",
    )
    .is_err());
}

#[test]
fn resolving_anonymous_identity_generates_distinct_operations() {
    let first = LifecycleIdentity::resolve(
        "project-1",
        "same task",
        SupportedAdapter::Codex,
        None,
        None,
    )
    .unwrap();
    let second = LifecycleIdentity::resolve(
        "project-1",
        "same task",
        SupportedAdapter::Codex,
        None,
        None,
    )
    .unwrap();
    assert_eq!(first.task_id(), second.task_id());
    assert_ne!(first.operation_id(), second.operation_id());
    assert_ne!(first.session_id(), second.session_id());
    assert_ne!(first.request_id(), second.request_id());
}
