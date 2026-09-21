use baron_core::operation::{
    operation_id_for_parts, task_id_for_task, AuthoritativeLifecycleIdentity, LifecycleIdentity,
    OperationContext, SupportedAdapter, MAX_IDENTIFIER_CHARS,
};
use baron_core::prepare::{operation_id_for_request, PrepareRequestV1};

#[test]
fn task_identity_ignores_session_request_and_line_ending_representation() {
    let first = task_id_for_task("project-1", "  Review API\r\ncontract  ").unwrap();
    let second = task_id_for_task("project-1", "Review API\ncontract").unwrap();
    assert_eq!(first, second);
}

#[test]
fn operation_identity_is_stable_only_for_the_same_complete_tuple() {
    let first = LifecycleIdentity::resolve(
        "project-1",
        "Review API",
        SupportedAdapter::Codex,
        Some("session-a"),
        Some("request-a"),
    )
    .unwrap();
    let task = first.task_id().to_string();
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
            SupportedAdapter::Codex,
            "session-b",
            "request-a",
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
fn checked_identity_reconstruction_rejects_forged_operation_id() {
    let task_id = task_id_for_task("project-1", "Review API").unwrap();
    let expected = operation_id_for_parts(
        "project-1",
        &task_id,
        SupportedAdapter::Codex,
        "session-a",
        "request-a",
    );

    let valid = LifecycleIdentity::from_parts_checked(
        "project-1",
        &task_id,
        &expected,
        SupportedAdapter::Codex,
        "session-a",
        "request-a",
    )
    .unwrap();
    assert_eq!(valid.operation_id(), expected);

    let error = LifecycleIdentity::from_parts_checked(
        "project-1",
        &task_id,
        "operation-FAKE",
        SupportedAdapter::Codex,
        "session-a",
        "request-a",
    )
    .unwrap_err();
    assert!(error.to_string().contains("does not match"));
}

#[test]
fn internally_consistent_fake_task_tuple_is_reconstruction_only() {
    let operation_id = operation_id_for_parts(
        "project-1",
        "task-fake",
        SupportedAdapter::Codex,
        "session-a",
        "request-a",
    );
    let reconstructed = LifecycleIdentity::from_parts_checked(
        "project-1",
        "task-fake",
        operation_id,
        SupportedAdapter::Codex,
        "session-a",
        "request-a",
    )
    .unwrap();

    assert_eq!(reconstructed.task_id(), "task-fake");
    assert!(reconstructed.validate_task("real task text").is_err());
    assert!(AuthoritativeLifecycleIdentity::from_identity_checked(
        &reconstructed,
        "real task text"
    )
    .is_err());
}

#[test]
fn canonical_task_resolution_produces_an_authority_capable_identity() {
    let resolved = LifecycleIdentity::resolve(
        "project-1",
        "real task text",
        SupportedAdapter::Codex,
        Some("session-a"),
        Some("request-a"),
    )
    .unwrap();
    let identity =
        AuthoritativeLifecycleIdentity::from_identity_checked(&resolved, "real task text").unwrap();

    assert_eq!(
        identity.task_id(),
        task_id_for_task("project-1", "real task text").unwrap()
    );
    assert_eq!(
        identity.operation_id(),
        operation_id_for_parts(
            identity.project_id(),
            identity.task_id(),
            identity.adapter(),
            identity.session_id(),
            identity.request_id(),
        )
    );
}

#[test]
fn forged_operation_context_is_rejected() {
    let task_id = task_id_for_task("project-1", "Review API").unwrap();
    let context = OperationContext::new(SupportedAdapter::Codex)
        .with_task_id(task_id)
        .with_operation_id("operation-FAKE")
        .with_session_id("session-a")
        .with_request_id("request-a");

    let error = context.lifecycle_identity("project-1").unwrap_err();
    assert!(error.to_string().contains("does not match"));
}

#[test]
fn operation_id_for_request_requires_present_non_blank_identity() {
    let request = |session_id: Option<&str>, request_id: Option<&str>| PrepareRequestV1 {
        schema_version: 1,
        task: "Review API".to_string(),
        session_id: session_id.map(str::to_string),
        request_id: request_id.map(str::to_string),
    };

    let valid = operation_id_for_request(
        "project-1",
        SupportedAdapter::Codex,
        &request(Some("session-a"), Some("request-a")),
    )
    .unwrap();
    assert_eq!(
        valid,
        operation_id_for_parts(
            "project-1",
            &task_id_for_task("project-1", "Review API").unwrap(),
            SupportedAdapter::Codex,
            "session-a",
            "request-a",
        )
    );

    for (session_id, request_id) in [
        (None, Some("request-a")),
        (Some("session-a"), None),
        (None, None),
        (Some("  "), Some("request-a")),
        (Some("session-a"), Some("\n")),
    ] {
        assert!(
            operation_id_for_request(
                "project-1",
                SupportedAdapter::Codex,
                &request(session_id, request_id),
            )
            .is_err(),
            "incomplete request identity must fail closed"
        );
    }
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
        "x".repeat(MAX_IDENTIFIER_CHARS + 1),
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

#[test]
fn resolving_blank_optional_ids_synthesizes_safe_values() {
    let identity = LifecycleIdentity::resolve(
        "project-1",
        "same task",
        SupportedAdapter::Claude,
        Some("  "),
        Some("\r\n"),
    )
    .unwrap();

    assert!(identity.session_id().starts_with("baron-session_id-"));
    assert!(identity.request_id().starts_with("baron-request_id-"));
    assert!(!identity.session_id().chars().any(char::is_control));
    assert!(!identity.request_id().chars().any(char::is_control));
}
