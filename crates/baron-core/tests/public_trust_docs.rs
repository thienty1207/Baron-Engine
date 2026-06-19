use std::fs;
use std::path::Path;

fn repo_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("baron-core lives under crates/baron-core")
}

fn read(relative: &str) -> String {
    fs::read_to_string(repo_root().join(relative)).expect(relative)
}

#[test]
fn readme_is_public_trust_landing_page_not_command_dump() {
    let readme = read("README.md");
    assert!(readme.contains("Current version: `3.1.0`"));
    assert!(readme.contains("## Quick Start"));
    assert!(readme.contains("## Demo"));
    assert!(readme.contains("## Public Proof"));
    assert!(readme.contains("docs/demo/README.md"));
    assert!(readme.contains("docs/assessment/baron-vs-repository-harness.md"));
    assert!(readme.contains("docs/assessment/baron-3-public-certification.md"));

    for command in [
        "baron --version",
        "baron setup --vault",
        "baron init --codex --fullstack",
        "baron update",
    ] {
        assert!(
            readme.contains(command),
            "README should expose normal-user command: {command}"
        );
    }

    for automation_command in [
        "baron memory",
        "baron harness",
        "baron proof",
        "baron trace",
        "baron control-plane",
        "baron autopilot",
        "baron runtime",
    ] {
        assert!(
            !readme.contains(automation_command),
            "README should keep automation command `{automation_command}` out of the main user flow"
        );
    }
}

#[test]
fn public_demo_benchmark_and_certification_docs_are_present() {
    let demo = read("docs/demo/README.md");
    assert!(demo.contains("10-year repo"));
    assert!(demo.contains("Codex"));
    assert!(demo.contains("Claude"));
    assert!(demo.contains("generic agent"));
    assert!(demo.contains("memory firewall"));
    assert!(demo.contains("proof gate"));
    assert!(demo.contains("session replay"));
    assert!(demo.contains("safe runtime backend"));

    let benchmark = read("docs/assessment/baron-vs-repository-harness.md");
    assert!(benchmark.contains("repository-harness"));
    assert!(benchmark.contains("repository-harness is simpler"));
    assert!(benchmark.contains("Baron is stronger"));
    assert!(benchmark.contains("memory firewall"));
    assert!(benchmark.contains("proof"));
    assert!(benchmark.contains("release safety"));

    let certification = read("docs/assessment/baron-3-public-certification.md");
    assert!(certification.contains("Baron 3.1.0 Public Trust"));
    assert!(certification.contains("cargo test --workspace --all-targets"));
    assert!(certification.contains("cargo clippy --workspace --all-targets -- -D warnings"));
    assert!(certification.contains("GitHub release latest"));
    assert!(certification.contains("shared Vault"));
    assert!(certification.contains("migration"));
}

#[test]
fn status_tracks_public_trust_phase() {
    let status_md = read("docs/BARON_STATUS.md");
    assert!(status_md.contains("Stable source release: `v3.1.0`"));
    assert!(status_md.contains("Phase 24 - Public Trust Release"));
    assert!(status_md.contains("Public Trust 3.1.0 final verification"));

    let status_json: serde_json::Value =
        serde_json::from_str(&read("docs/BARON_STATUS.json")).expect("valid status json");
    assert_eq!(status_json["stableRelease"], "3.1.0");
    assert_eq!(status_json["targetRelease"], "3.1.0");
    assert_eq!(status_json["currentPhase"], "phase-24-public-trust-release");
    assert_eq!(status_json["currentPhaseStatus"], "completed");
}
