use std::fs;

use baron_adapters::{install_adapter, managed_payloads_for_adapter, AgentAdapter};
use tempfile::tempdir;

#[test]
fn codex_hook_projection_has_normalized_events_and_is_idempotent() {
    let temp = tempdir().unwrap();
    fs::create_dir_all(temp.path().join(".codex")).unwrap();
    fs::write(
        temp.path().join(".codex/hooks.json"),
        r#"{"hooks":{"SessionStart":[{"command":"third-party-start"}]},"third_party":true}"#,
    )
    .unwrap();
    install_adapter(temp.path(), AgentAdapter::Codex).unwrap();
    let first = fs::read_to_string(temp.path().join(".codex/hooks.json")).unwrap();
    install_adapter(temp.path(), AgentAdapter::Codex).unwrap();
    let second = fs::read_to_string(temp.path().join(".codex/hooks.json")).unwrap();
    assert_eq!(first, second);
    assert!(first.contains("third-party-start"));
    assert!(first.contains("UserPromptSubmit"));
    assert!(first.contains("PreCompact"));
    assert!(first.contains("automation hook user-prompt-submit"));
    assert!(first.contains("automation hook pre-compact"));
    assert!(!first.contains("control-plane prepare"));
}

#[test]
fn claude_hook_projection_preserves_settings_and_has_bounded_accelerators() {
    let temp = tempdir().unwrap();
    fs::create_dir_all(temp.path().join(".claude")).unwrap();
    fs::write(
        temp.path().join(".claude/settings.json"),
        r#"{"permissions":{"allow":["Read"]},"hooks":{"Stop":[{"command":"personal-stop"}]},"autoMemoryEnabled":true}"#,
    )
    .unwrap();
    install_adapter(temp.path(), AgentAdapter::Claude).unwrap();
    let settings = fs::read_to_string(temp.path().join(".claude/settings.json")).unwrap();
    let value: serde_json::Value = serde_json::from_str(&settings).unwrap();
    assert_eq!(value["autoMemoryEnabled"], true);
    assert_eq!(value["permissions"]["allow"][0], "Read");
    assert!(settings.contains("personal-stop"));
    assert!(settings.contains("SessionStart"));
    assert!(settings.contains("UserPromptSubmit"));
    assert!(settings.contains("PreCompact"));
    assert!(settings.contains("automation hook user-prompt-submit"));
    assert!(settings.contains("automation hook pre-compact"));
    assert!(!settings.contains("control-plane prepare"));
}

#[test]
fn hook_commands_use_stdin_transport_instead_of_prompt_interpolation() {
    for adapter in [AgentAdapter::Codex, AgentAdapter::Claude] {
        let payloads = managed_payloads_for_adapter(adapter).unwrap();
        let hooks = payloads
            .iter()
            .find(|payload| {
                payload
                    .relative_path
                    .to_string_lossy()
                    .ends_with("hooks.json")
                    || payload
                        .relative_path
                        .to_string_lossy()
                        .ends_with("settings.json")
            })
            .unwrap();
        assert!(hooks.content.contains("automation hook"));
        assert!(!hooks.content.contains("$task"));
        assert!(!hooks.content.contains("--task"));
    }
}
