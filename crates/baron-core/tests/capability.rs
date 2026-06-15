use std::fs;
use std::net::TcpListener;

use baron_core::capability::{
    check_capabilities, load_capability_state, load_registry, normalize_identifier,
    register_provider, remove_provider, CapabilityProvider, CheckOptions, Presence, ProviderKind,
    Requirement,
};
use baron_core::config::{initialize_project, AdapterKind};
use tempfile::tempdir;

fn provider(
    name: &str,
    capability: &str,
    kind: ProviderKind,
    requirement: Requirement,
) -> CapabilityProvider {
    CapabilityProvider {
        name: name.to_string(),
        capability: capability.to_string(),
        kind,
        requirement,
        command: None,
        scan_target: None,
        adapters: Vec::new(),
        description: "Provides focused verification evidence for Baron.".to_string(),
    }
}

#[test]
fn identifiers_are_normalized_to_stable_kebab_case() {
    assert_eq!(
        normalize_identifier(" Impact_Analysis  "),
        Some("impact-analysis".to_string())
    );
    assert_eq!(
        normalize_identifier("security--scan"),
        Some("security-scan".to_string())
    );
    assert_eq!(normalize_identifier("___"), None);
}

#[test]
fn registry_round_trip_supports_all_provider_kinds_and_rejects_duplicates() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    fs::create_dir_all(&repo).unwrap();
    let kinds = [
        ProviderKind::Cli,
        ProviderKind::Binary,
        ProviderKind::Mcp,
        ProviderKind::Skill,
        ProviderKind::Http,
        ProviderKind::AgentAdapter,
    ];

    for (index, kind) in kinds.into_iter().enumerate() {
        let mut item = provider(
            &format!("provider-{index}"),
            &format!("Capability {index}"),
            kind,
            Requirement::Optional,
        );
        match kind {
            ProviderKind::Cli | ProviderKind::Binary => {
                item.command = Some("git".to_string());
            }
            ProviderKind::Mcp | ProviderKind::Skill => {
                item.scan_target = Some(format!(".tools/provider-{index}"));
                item.adapters = vec![AdapterKind::Codex];
            }
            ProviderKind::Http => {
                item.scan_target = Some("http://127.0.0.1:9/health".to_string());
            }
            ProviderKind::AgentAdapter => {
                item.adapters = vec![AdapterKind::Claude];
            }
        }
        register_provider(&repo, item).unwrap();
    }

    let registry = load_registry(&repo).unwrap();
    assert_eq!(registry.schema_version, 1);
    assert_eq!(registry.providers.len(), 6);
    assert_eq!(registry.providers[0].capability, "capability-0");

    let duplicate = provider(
        "provider-0",
        "another-capability",
        ProviderKind::Cli,
        Requirement::Optional,
    );
    assert!(register_provider(&repo, duplicate).is_err());
}

#[test]
fn remove_targets_one_capability_provider_without_touching_others() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    fs::create_dir_all(&repo).unwrap();
    let mut first = provider(
        "cargo-test",
        "test-suite",
        ProviderKind::Cli,
        Requirement::Required,
    );
    first.command = Some("cargo".to_string());
    let mut second = provider(
        "security-skill",
        "security-scan",
        ProviderKind::Skill,
        Requirement::Optional,
    );
    second.scan_target = Some(".codex/skills/security".to_string());
    second.adapters = vec![AdapterKind::Codex];
    register_provider(&repo, first).unwrap();
    register_provider(&repo, second).unwrap();

    assert!(remove_provider(&repo, "test-suite", "cargo-test").unwrap());
    let registry = load_registry(&repo).unwrap();
    assert_eq!(registry.providers.len(), 1);
    assert_eq!(registry.providers[0].name, "security-skill");
}

#[test]
fn local_checks_report_presence_compatibility_and_persist_rebuildable_state() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(repo.join(".codex/skills/security")).unwrap();
    initialize_project(&repo, AdapterKind::Codex, &vault).unwrap();

    let mut cli = provider(
        "git-cli",
        "source-control",
        ProviderKind::Cli,
        Requirement::Required,
    );
    cli.command = Some("git".to_string());
    let mut skill = provider(
        "security-skill",
        "security-scan",
        ProviderKind::Skill,
        Requirement::Optional,
    );
    skill.scan_target = Some(".codex/skills/security".to_string());
    skill.adapters = vec![AdapterKind::Codex];
    let mut claude_only = provider(
        "claude-review",
        "code-review",
        ProviderKind::AgentAdapter,
        Requirement::Optional,
    );
    claude_only.adapters = vec![AdapterKind::Claude];
    let mut http = provider(
        "local-health",
        "deploy-verification",
        ProviderKind::Http,
        Requirement::Optional,
    );
    http.scan_target = Some("http://127.0.0.1:9/health".to_string());
    for item in [cli, skill, claude_only, http] {
        register_provider(&repo, item).unwrap();
    }

    let state = check_capabilities(
        &repo,
        CheckOptions {
            adapter: AdapterKind::Codex,
            capability: None,
            allow_network: false,
        },
    )
    .unwrap();

    let git = state
        .observations
        .iter()
        .find(|item| item.provider == "git-cli")
        .unwrap();
    assert_eq!(git.presence, Presence::Present);
    assert!(git.compatible);
    assert!(git.evidence.contains("resolved"));

    let skill = state
        .observations
        .iter()
        .find(|item| item.provider == "security-skill")
        .unwrap();
    assert_eq!(skill.presence, Presence::Present);
    assert!(skill.compatible);

    let claude = state
        .observations
        .iter()
        .find(|item| item.provider == "claude-review")
        .unwrap();
    assert_eq!(claude.presence, Presence::Unknown);
    assert!(!claude.compatible);

    let http = state
        .observations
        .iter()
        .find(|item| item.provider == "local-health")
        .unwrap();
    assert_eq!(http.presence, Presence::Unknown);
    assert!(http.evidence.contains("network probe disabled"));

    let cached = load_capability_state(&repo).unwrap().unwrap();
    assert_eq!(cached.observations, state.observations);
    assert!(repo.join(".baron/cache/capability-state.json").exists());
}

#[test]
fn missing_optional_provider_degrades_cleanly_but_required_gap_is_explicit() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    initialize_project(&repo, AdapterKind::Generic, &vault).unwrap();

    let mut optional = provider(
        "optional-linter",
        "lint",
        ProviderKind::Binary,
        Requirement::Optional,
    );
    optional.command = Some("definitely-missing-baron-tool".to_string());
    let mut required = provider(
        "required-deploy-check",
        "deploy-verification",
        ProviderKind::Cli,
        Requirement::Required,
    );
    required.command = Some("definitely-missing-baron-check".to_string());
    register_provider(&repo, optional).unwrap();
    register_provider(&repo, required).unwrap();

    let state = check_capabilities(
        &repo,
        CheckOptions {
            adapter: AdapterKind::Generic,
            capability: None,
            allow_network: false,
        },
    )
    .unwrap();

    assert_eq!(state.required_gaps, vec!["deploy-verification"]);
    assert_eq!(state.optional_gaps, vec!["lint"]);
}

#[test]
fn explicit_checks_cover_mcp_http_and_registered_agent_adapter_providers() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(repo.join(".tools/graph")).unwrap();
    initialize_project(&repo, AdapterKind::Codex, &vault).unwrap();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();

    let mut mcp = provider(
        "code-graph",
        "impact-analysis",
        ProviderKind::Mcp,
        Requirement::Optional,
    );
    mcp.scan_target = Some(".tools/graph".to_string());
    mcp.adapters = vec![AdapterKind::Codex];
    let mut http = provider(
        "health-endpoint",
        "deploy-verification",
        ProviderKind::Http,
        Requirement::Optional,
    );
    http.scan_target = Some(format!("http://{address}/health"));
    let mut adapter = provider(
        "codex-runtime",
        "agent-runtime",
        ProviderKind::AgentAdapter,
        Requirement::Optional,
    );
    adapter.adapters = vec![AdapterKind::Codex];
    for item in [mcp, http, adapter] {
        register_provider(&repo, item).unwrap();
    }

    let state = check_capabilities(
        &repo,
        CheckOptions {
            adapter: AdapterKind::Codex,
            capability: None,
            allow_network: true,
        },
    )
    .unwrap();

    for provider_name in ["code-graph", "health-endpoint", "codex-runtime"] {
        let observation = state
            .observations
            .iter()
            .find(|item| item.provider == provider_name)
            .unwrap();
        assert_eq!(observation.presence, Presence::Present);
        assert!(observation.compatible);
    }
}
