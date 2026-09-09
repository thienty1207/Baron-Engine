//! Phase 15 documentation and generated-contract invariants.

use std::path::Path;

use baron_adapters::{core_managed_payloads, managed_payloads_for_adapter, AgentAdapter};

fn content<'a>(payloads: &'a [baron_adapters::ManagedAssetPayload], path: &str) -> &'a str {
    payloads
        .iter()
        .find(|payload| payload.relative_path == Path::new(path))
        .unwrap_or_else(|| panic!("missing generated payload {path}"))
        .content
        .as_str()
}

#[test]
fn generated_codex_and_claude_contracts_match_the_documented_lifecycle() {
    let codex = managed_payloads_for_adapter(AgentAdapter::Codex).unwrap();
    let claude = managed_payloads_for_adapter(AgentAdapter::Claude).unwrap();
    let codex_root = content(&codex, "AGENTS.md");
    let claude_root = content(&claude, "CLAUDE.md");
    for (name, body) in [("AGENTS.md", codex_root), ("CLAUDE.md", claude_root)] {
        assert!(
            body.len() < 12_000,
            "{name} must stay compact for host instruction budgets"
        );
        assert!(
            !body.contains("Phase 1"),
            "{name} must not embed phase history"
        );
        for required in [
            "user's current explicit instructions",
            "PrepareRequestV1",
            "Task State",
            "hooks",
            "fallback",
            "parent",
            "proof",
            "recovery",
            "selected",
            "Do not ask the user to run hidden Baron commands",
        ] {
            assert!(body.contains(required), "{name} is missing `{required}`");
        }
        assert!(!body.contains("active adapter is the runtime authority"));
    }

    let codex_bridge = content(&codex, ".agents/skills/baron-engine/SKILL.md");
    let claude_bridge = content(&claude, ".claude/skills/baron-engine/SKILL.md");
    let codex_metadata = content(&codex, ".agents/skills/baron-engine/agents/openai.yaml");
    assert!(codex_metadata.contains("allow_implicit_invocation: false"));
    assert!(claude_bridge.contains("disable-model-invocation: true"));
    for bridge in [codex_bridge, claude_bridge] {
        assert!(bridge.contains("canonical Baron Core"));
        assert!(bridge.contains("selected"));
        assert!(bridge.contains("parent"));
        assert!(bridge.contains("Do not preload or recursively read all"));
    }
}

#[test]
fn generated_layout_has_one_core_source_and_no_full_adapter_skill_tree() {
    let core = core_managed_payloads().unwrap();
    assert!(core
        .iter()
        .all(|payload| payload.relative_path.starts_with(Path::new(".baron/core/"))));
    assert!(core.iter().any(|payload| {
        payload.relative_path == Path::new(".baron/core/skills/superpowers/SKILL.md")
    }));

    for adapter in [AgentAdapter::Codex, AgentAdapter::Claude] {
        let payloads = managed_payloads_for_adapter(adapter).unwrap();
        assert!(!payloads.iter().any(|payload| {
            let path = payload.relative_path.to_string_lossy();
            path.starts_with(".codex/skills/")
                || (path.starts_with(".claude/skills/")
                    && path != ".claude/skills/INDEX.md"
                    && path != ".claude/skills/baron-engine/SKILL.md")
        }));
        let bridge_path = match adapter {
            AgentAdapter::Codex => ".agents/skills/baron-engine/SKILL.md",
            AgentAdapter::Claude => ".claude/skills/baron-engine/SKILL.md",
        };
        assert!(payloads
            .iter()
            .any(|payload| payload.relative_path == Path::new(bridge_path)));
    }
}
