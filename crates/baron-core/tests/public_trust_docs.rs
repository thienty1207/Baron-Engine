use std::fs;
use std::path::{Path, PathBuf};

fn repo_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("baron-core lives under crates/baron-core")
}

fn read(relative: &str) -> String {
    fs::read_to_string(repo_root().join(relative)).expect(relative)
}

fn collect_public_text_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).expect("read public docs tree") {
        let entry = entry.expect("read public docs entry");
        let path = entry.path();
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name == ".git" || name == "target" {
            continue;
        }
        if path.is_dir() {
            collect_public_text_files(&path, out);
            continue;
        }
        if matches!(
            path.extension().and_then(|ext| ext.to_str()),
            Some("md" | "json" | "toml" | "rs")
        ) {
            out.push(path);
        }
    }
}

#[test]
fn readme_is_public_trust_landing_page_not_command_dump() {
    let readme = read("README.md");
    assert!(readme.contains("Current source version: `3.6.0`"));
    assert!(readme.contains("## Quick Start"));
    assert!(readme.contains("## Demo"));
    assert!(readme.contains("## Public Proof"));
    assert!(readme.contains("docs/demo/README.md"));
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
fn readme_keeps_research_attribution_and_automation_out_of_the_user_flow() {
    let readme = read("README.md").to_ascii_lowercase();

    for hidden in [
        "hallmark",
        "mattpocock",
        "matt pocock",
        "baron automation",
        "baron context",
        "baron control-plane",
    ] {
        assert!(
            !readme.contains(hidden),
            "README must keep `{hidden}` out of the normal user flow"
        );
    }
}

#[test]
fn public_demo_and_certification_docs_are_present() {
    let demo = read("docs/demo/README.md");
    assert!(demo.contains("10-year repo"));
    assert!(demo.contains("Codex"));
    assert!(demo.contains("Claude"));
    assert!(demo.contains("generic agent"));
    assert!(demo.contains("memory firewall"));
    assert!(demo.contains("proof gate"));
    assert!(demo.contains("session replay"));
    assert!(demo.contains("safe runtime backend"));

    let certification = read("docs/assessment/baron-3-public-certification.md");
    assert!(certification.contains("Baron 3.1.2 Public Trust"));
    assert!(certification.contains("cargo test --workspace --all-targets"));
    assert!(certification.contains("cargo clippy --workspace --all-targets -- -D warnings"));
    assert!(certification.contains("GitHub release latest"));
    assert!(certification.contains("shared Vault"));
    assert!(certification.contains("migration"));
}

#[test]
fn baron_3_3_certification_records_trust_state_and_release_evidence() {
    let certification = read("docs/assessment/baron-3.3.0-certification.md");

    for required in [
        "Request Authority",
        "Coherent State",
        "Completion Integrity",
        "cargo test --workspace --all-targets",
        "cargo clippy --workspace --all-targets -- -D warnings",
        "cargo build --release --locked -p baron-cli",
        "exact source revision",
        "Binary GitHub Release",
    ] {
        assert!(
            certification.contains(required),
            "Baron 3.3 certification is missing {required}"
        );
    }
}

#[test]
fn baron_3_4_safe_update_certification_states_its_release_boundary() {
    let certification = read("docs/assessment/baron-3.4.0-safe-update-certification.md");
    for required in [
        "Baron 3.4.0 Safe Update Certification",
        "baron update",
        "baron automation reconcile",
        "BASE`, `LOCAL`, `UPSTREAM`, and `RESOLVED`",
        "AI agents never run public `baron update`",
        "Git tag or GitHub Release",
    ] {
        assert!(
            certification.contains(required),
            "Baron 3.4 safe-update certification is missing {required}"
        );
    }
}

#[test]
fn baron_3_5_certification_records_local_skill_and_preservation_boundaries() {
    let certification = read("docs/assessment/baron-3.5.0-skill-intelligence-certification.md");
    for required in [
        "Baron 3.5 Skill Intelligence Certification",
        "aeb42fb354ff4efa36ab475773a082315a3af2ce",
        "ed37663cc5fbef691ddfecd080dff42f7e7e350d",
        "three adapters",
        "automation_reconcile_preserves_custom_routing_and_recovers_missing_domain_language",
        "No external installer, live instruction, or workflow skill was imported.",
        "Source `3.5.0` is not a Git tag or GitHub Release.",
    ] {
        assert!(
            certification.contains(required),
            "Baron 3.5 certification is missing {required}"
        );
    }
}

#[test]
fn baron_3_6_certification_records_optional_code_map_boundaries() {
    let certification = read("docs/assessment/baron-3.6.0-code-graph-certification.md");
    for required in [
        "Baron 3.6 Optional Code Map Certification",
        "0.9.25",
        "2fa6cd3d5548577f8c5f591b713f0bf80c1af183",
        "graphify --version",
        "code-only extraction",
        "same_name_projects_keep_graphs_local_and_out_of_vault_memory",
        "large_repository_survey_and_context_remain_bounded",
        "Source `3.6.0` is not a Git tag or GitHub Release.",
    ] {
        assert!(
            certification.contains(required),
            "Baron 3.6 certification is missing {required}"
        );
    }
}

#[test]
fn status_tracks_current_program() {
    let status_md = read("docs/BARON_STATUS.md");
    assert!(status_md.contains("Stable source release: `v3.6.0`"));
    assert!(status_md.contains("Phase 24 - Public Trust Release"));
    assert!(status_md.contains("Public Trust 3.1.2 final verification"));

    let status_json: serde_json::Value =
        serde_json::from_str(&read("docs/BARON_STATUS.json")).expect("valid status json");
    assert_eq!(status_json["stableRelease"], "3.6.0");
    assert_eq!(status_json["targetRelease"], "3.6.0");
    assert_eq!(status_json["programTargetRelease"], "3.6.0");
    assert_eq!(status_json["remainingPhaseCount"], 0);
    assert!(status_json["phases"]
        .as_array()
        .is_some_and(|phases| phases.iter().any(|phase| {
            phase["id"] == 35
                && phase["name"] == "Single Runtime Source And Managed Baseline"
                && phase["status"] == "completed"
        })));
    assert!(status_json["phases"]
        .as_array()
        .is_some_and(|phases| phases.iter().any(|phase| {
            phase["id"] == 38
                && phase["name"] == "Automation Contract And Baron 3.4 Certification"
                && phase["status"] == "completed"
        })));
    assert!(status_json["phases"]
        .as_array()
        .is_some_and(|phases| phases.iter().any(|phase| {
            phase["id"] == 45
                && phase["name"] == "Isolation Scale And Baron 3.6 Certification"
                && phase["status"] == "completed"
        })));
    assert!(status_json["currentPhase"]
        .as_str()
        .is_some_and(|phase| !phase.is_empty()));
    assert!(status_json["phases"]
        .as_array()
        .expect("phase list")
        .iter()
        .any(
            |phase| phase["id"] == 27 && phase["name"] == "Intent Clarity And Actionable Recovery"
        ));
}

#[test]
fn public_docs_do_not_reference_external_harness_repositories() {
    let root = repo_root();
    let external_harness_name = format!("{}-{}", "repository", "harness");
    let external_owner_name = format!("{}{}", "hoang", "nb24");
    let forbidden = [external_harness_name.as_str(), external_owner_name.as_str()];
    let mut files = Vec::new();
    collect_public_text_files(root, &mut files);

    let offenders: Vec<String> = files
        .into_iter()
        .filter_map(|path| {
            let body = fs::read_to_string(&path).ok()?;
            let matched = forbidden.iter().any(|needle| body.contains(needle));
            matched.then(|| path.strip_prefix(root).unwrap().display().to_string())
        })
        .collect();

    assert!(
        offenders.is_empty(),
        "public Baron files must not point readers at external harness repos: {offenders:#?}"
    );
    let path_offenders: Vec<String> = collect_paths(root)
        .into_iter()
        .filter_map(|path| {
            let relative = path.strip_prefix(root).ok()?.display().to_string();
            relative
                .contains(&external_harness_name)
                .then_some(relative)
        })
        .collect();
    assert!(
        path_offenders.is_empty(),
        "public Baron file paths must not point readers at external harness repos: {path_offenders:#?}"
    );
}

fn collect_paths(dir: &Path) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    for entry in fs::read_dir(dir).expect("read public docs tree") {
        let entry = entry.expect("read public docs entry");
        let path = entry.path();
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name == ".git" || name == "target" {
            continue;
        }
        if path.is_dir() {
            paths.extend(collect_paths(&path));
        } else {
            paths.push(path);
        }
    }
    paths
}
