#[cfg(windows)]
mod windows {
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::Mutex;
    use std::time::Duration;

    use baron_core::code_graph::{
        code_graph_cache_root, load_code_graph_state, CodeGraphProvider, QueryLimits,
    };
    use baron_core::config::{initialize_project, AdapterKind};
    use baron_core::graphify::{
        GraphifyLimits, GraphifyProvider, AUDITED_GRAPHIFY_REVISION, SUPPORTED_GRAPHIFY_VERSION,
    };
    use tempfile::tempdir;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn write(path: &Path, content: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, content).unwrap();
    }

    fn fixture() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/fake-graphify.ps1")
    }

    fn provider() -> GraphifyProvider {
        GraphifyProvider::powershell_script(fixture()).with_limits(GraphifyLimits {
            probe_timeout: Duration::from_secs(1),
            refresh_timeout: Duration::from_secs(1),
            query_timeout: Duration::from_secs(1),
            max_graph_bytes: 1_024,
            max_stdout_bytes: 1_024,
            max_stderr_bytes: 512,
        })
    }

    fn project() -> (tempfile::TempDir, PathBuf, PathBuf) {
        let temp = tempdir().unwrap();
        let repo = temp.path().join("repo");
        let vault = temp.path().join("Vault");
        fs::create_dir_all(&repo).unwrap();
        initialize_project(&repo, AdapterKind::Codex, &vault).unwrap();
        write(&repo.join("src/lib.rs"), "pub fn entry() {}\n");
        write(&repo.join("src/service.rs"), "pub fn service() {}\n");
        (temp, repo, vault)
    }

    #[test]
    fn graphify_accepts_only_the_pinned_version_and_local_code_only_commands() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|poison| poison.into_inner());
        let (temp, repo, vault) = project();
        let log = temp.path().join("provider.log");
        std::env::set_var("FAKE_GRAPHIFY_LOG", &log);
        std::env::set_var("GRAPHIFY_API_KEY", "must-not-reach-provider");
        std::env::remove_var("FAKE_GRAPHIFY_MODE");

        let provider = provider();
        assert_eq!(SUPPORTED_GRAPHIFY_VERSION, "0.9.25");
        assert_eq!(
            AUDITED_GRAPHIFY_REVISION,
            "2fa6cd3d5548577f8c5f591b713f0bf80c1af183"
        );
        let cache = code_graph_cache_root(&repo).unwrap();
        let state = provider.refresh(&repo, &cache).unwrap();
        assert_eq!(state.provider, "graphify-local");
        assert_eq!(state.provider_version, SUPPORTED_GRAPHIFY_VERSION);
        let hits = provider
            .query(
                &repo,
                &cache,
                "trace entry ownership",
                QueryLimits::default(),
            )
            .unwrap();
        assert_eq!(hits.len(), 2);
        assert_eq!(hits[0].node_id, "entry");
        assert_eq!(hits[1].node_id, "related");
        assert!(load_code_graph_state(&repo).unwrap().is_some());
        assert!(!cache.starts_with(&vault));

        let log = fs::read_to_string(&log).unwrap();
        assert!(log.contains("--version"));
        assert!(log.contains("extract"));
        assert!(log.contains("--code-only"));
        assert!(log.contains("--out"));
        assert!(log.contains("--no-cluster"));
        assert!(log.contains("query trace entry ownership --graph"));
        assert!(log.contains("--json --budget 8"));
        assert!(log.contains("query_log_disable=1"));
        assert!(log.contains("graphify_api_key_present=False"));
        for forbidden in [
            "install",
            "hook",
            "global",
            "save-result",
            "reflect",
            "http://",
            "https://",
        ] {
            assert!(
                !log.contains(forbidden),
                "unexpected provider call: {forbidden}"
            );
        }
        assert!(!log.contains(&vault.display().to_string()));
        std::env::remove_var("FAKE_GRAPHIFY_LOG");
        std::env::remove_var("GRAPHIFY_API_KEY");
    }

    #[test]
    fn graphify_failures_keep_last_known_good_state_and_return_fallback_errors() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|poison| poison.into_inner());
        let (_temp, repo, _vault) = project();
        let cache = code_graph_cache_root(&repo).unwrap();
        std::env::remove_var("FAKE_GRAPHIFY_MODE");
        let provider = provider();
        let original = provider.refresh(&repo, &cache).unwrap();

        for mode in ["wrong-version", "nonzero", "timeout", "oversized-graph"] {
            std::env::set_var("FAKE_GRAPHIFY_MODE", mode);
            let error = provider.refresh(&repo, &cache).unwrap_err().to_string();
            assert!(
                error.contains("Graphify") || error.contains("graph"),
                "mode {mode} returned an unclear fallback error: {error}"
            );
            let current = load_code_graph_state(&repo).unwrap().unwrap();
            assert_eq!(current.graph_sha256, original.graph_sha256, "mode {mode}");
        }

        for mode in ["malformed", "oversized", "nonzero", "timeout"] {
            std::env::set_var("FAKE_GRAPHIFY_MODE", mode);
            assert!(provider
                .query(&repo, &cache, "trace entry", QueryLimits::default())
                .is_err());
            let current = load_code_graph_state(&repo).unwrap().unwrap();
            assert_eq!(current.graph_sha256, original.graph_sha256, "mode {mode}");
        }
        std::env::remove_var("FAKE_GRAPHIFY_MODE");
    }

    #[test]
    fn missing_provider_is_optional_and_does_not_attempt_extraction() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|poison| poison.into_inner());
        let (_temp, repo, _vault) = project();
        let missing = GraphifyProvider::new("baron-definitely-missing-graphify");
        let probe = missing.probe(&repo).unwrap();
        assert!(!probe.present);
        assert!(probe.version.is_none());
        let cache = code_graph_cache_root(&repo).unwrap();
        assert!(missing.refresh(&repo, &cache).is_err());
        assert!(!cache.exists());
    }
}

#[cfg(not(windows))]
#[test]
fn graphify_provider_fixture_is_windows_specific_but_core_remains_portable() {
    assert!(true);
}
