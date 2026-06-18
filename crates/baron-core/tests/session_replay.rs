use std::fs;
use std::path::Path;

use baron_core::session_replay::{
    index_session_replay, replay_session_context, search_session_replay,
};
use baron_core::vault::ensure_vault;
use tempfile::tempdir;

fn write(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, content).unwrap();
}

#[test]
fn indexes_imported_sessions_and_searches_exact_messages() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("tomoty");
    let vault = temp.path().join("vault");
    fs::create_dir_all(&repo).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();
    write(
        &context
            .project_root
            .join("Sessions/Imported/codex-login.md"),
        "---\n\
type: baron-imported-session\n\
adapter: codex\n\
source_hash: abc\n\
---\n\n\
# Imported Codex Session\n\n\
## Clean Conversation Memory\n\n\
### User\n\n\
We decided Gin login must use refresh token rotation and tenant-safe sessions.\n\n\
### Assistant\n\n\
Decision: implement Gin login with server-side refresh token rotation.\n\n\
### User\n\n\
Next action is add auth integration tests.\n",
    );

    let report = index_session_replay(&context).unwrap();
    assert_eq!(report.indexed_messages, 3);

    let hits = search_session_replay(&context, "Gin refresh token", 5).unwrap();
    assert!(!hits.is_empty());
    assert!(hits.iter().any(|hit| hit.project_id == context.project_id));
    assert!(hits
        .iter()
        .any(|hit| hit.text.contains("refresh token rotation")));
}

#[test]
fn replay_returns_bounded_surrounding_context() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("tomoty");
    let vault = temp.path().join("vault");
    fs::create_dir_all(&repo).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();
    write(
        &context
            .project_root
            .join("Sessions/Imported/codex-login.md"),
        "---\n\
type: baron-imported-session\n\
adapter: codex\n\
source_hash: abc\n\
---\n\n\
# Imported Codex Session\n\n\
## Clean Conversation Memory\n\n\
### User\n\n\
First message should stay outside a radius-one replay.\n\n\
### Assistant\n\n\
Decision: implement Gin login with server-side refresh token rotation.\n\n\
### User\n\n\
Next action is add auth integration tests.\n\n\
### Assistant\n\n\
Fourth message should stay outside a radius-one replay.\n",
    );

    index_session_replay(&context).unwrap();
    let hits = search_session_replay(&context, "auth integration tests", 5).unwrap();
    let target = hits
        .iter()
        .find(|hit| hit.text.contains("auth integration tests"))
        .unwrap();

    let replay = replay_session_context(&context, &target.message_id, 1).unwrap();
    let joined = replay
        .messages
        .iter()
        .map(|message| message.text.as_str())
        .collect::<Vec<_>>()
        .join("\n");

    assert!(joined.contains("refresh token rotation"));
    assert!(joined.contains("auth integration tests"));
    assert!(joined.contains("Fourth message"));
    assert!(!joined.contains("First message"));
}

#[test]
fn session_search_obeys_project_firewall() {
    let temp = tempdir().unwrap();
    let vault = temp.path().join("vault");
    let tomoty = temp.path().join("tomoty");
    let legacy = temp.path().join("legacy-crm");
    fs::create_dir_all(&tomoty).unwrap();
    fs::create_dir_all(&legacy).unwrap();
    let tomoty_context = ensure_vault(&vault, &tomoty).unwrap();
    let legacy_context = ensure_vault(&vault, &legacy).unwrap();

    write(
        &tomoty_context
            .project_root
            .join("Sessions/Imported/codex-current.md"),
        "# Imported Session\n\n## Clean Conversation Memory\n\n### User\n\nCurrent project login uses Gin refresh token rotation.\n",
    );
    write(
        &legacy_context
            .project_root
            .join("Sessions/Imported/codex-legacy.md"),
        "# Imported Session\n\n## Clean Conversation Memory\n\n### User\n\nLegacy CRM billing login uses PHP session cookies.\n",
    );

    index_session_replay(&tomoty_context).unwrap();
    index_session_replay(&legacy_context).unwrap();

    let hits = search_session_replay(&tomoty_context, "login", 10).unwrap();
    assert!(hits.iter().any(|hit| hit.text.contains("Gin refresh")));
    assert!(!hits.iter().any(|hit| hit.text.contains("PHP session")));
}
